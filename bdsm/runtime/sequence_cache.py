import logging
import time

log = logging.getLogger("sequence_cache")

try:
    import zstd as _zstd_mod

    def _zstd_decompress(data: bytes) -> bytes:
        return _zstd_mod.decompress(data)
except ImportError:
    import zstandard as _zstandard

    def _zstd_decompress(data: bytes) -> bytes:
        return _zstandard.ZstdDecompressor().decompress(data, max_output_size=64 * 1024 * 1024)


_ZSTD_MAGIC = b"\x28\xb5\x2f\xfd"
_ARROW_MAGIC = b"ARROW1"

_PROTO_FIELD_EXTRACTORS = {
    "action_name": lambda ua: int(ua.actionInfo.actionName),
    "engagement_time_ms": lambda ua: int(ua.actionInfo.engagementTimeMs),
    "tweet_id": lambda ua: int(ua.tweetInfo.tweetId),
    "author_id": lambda ua: int(ua.tweetInfo.authorId),
    "dwell_time": lambda ua: ua.actionInfo.userActionMeta.dwellTime,
    "product_surface": lambda ua: int(ua.actionInfo.userActionMeta.productSurface),
    "client_app_id": lambda ua: int(ua.actionInfo.userActionMeta.clientAppId),
    "is_author_followed_by_user": lambda ua: bool(ua.tweetInfo.isAuthorFollowedByUser),
    "favorite_count": lambda ua: int(ua.tweetInfo.favCount),
    "retweet_count": lambda ua: int(ua.tweetInfo.retweetCount),
    "reply_count": lambda ua: int(ua.tweetInfo.replyCount),
    "quote_count": lambda ua: int(ua.tweetInfo.quoteCount),
    "has_media": lambda ua: bool(ua.tweetInfo.tweetBoolFeatures.has_media),
    "promoted_id": lambda ua: int(ua.tweetInfo.promotedId),
    "post_language_code": lambda ua: int(ua.tweetInfo.languageCode),
    "quoting_author_id": lambda ua: int(ua.tweetInfo.quotingAuthorId),
    "quoted_author_id": lambda ua: int(ua.tweetInfo.quotedAuthorId),
    "in_reply_to_author_id": lambda ua: int(ua.tweetInfo.inReplyToAuthorId),
    "retweeting_author_id": lambda ua: int(ua.tweetInfo.retweetingAuthorId),
    "engaging_ip_country_code": lambda ua: int(ua.actionInfo.userActionMeta.engagingIpCountryCode),
    "served_time_ms": lambda ua: int(ua.actionInfo.userActionMeta.servedTimeMs),
    "search_query": lambda ua: str(ua.actionInfo.userActionMeta.searchQuery),
}


class SequenceCacheClient:
    def __init__(
        self,
        host: str = "localhost",
        port: int = 6379,
        timeout_ms: int = 150,
    ):
        import redis as redis_lib

        self._r = redis_lib.Redis(
            host=host,
            port=port,
            socket_timeout=timeout_ms / 1000.0,
            socket_connect_timeout=timeout_ms / 1000.0,
            decode_responses=False,
        )
        self._r.ping()
        log.info(f"SequenceCacheClient connected to {host}:{port} (timeout={timeout_ms}ms)")

    def get(self, user_id: int) -> bytes | None:
        return self._r.get(str(user_id))

    def set(self, user_id: int, value: bytes, ttl_secs: int) -> None:
        try:
            self._r.setex(str(user_id), ttl_secs, value)
        except Exception:
            pass


class RealtimeCacheClient:
    def __init__(
        self,
        host: str = "localhost",
        port: int = 6382,
        timeout_ms: int = 100,
    ):
        import redis as redis_lib

        self._r = redis_lib.Redis(
            host=host,
            port=port,
            socket_timeout=timeout_ms / 1000.0,
            socket_connect_timeout=timeout_ms / 1000.0,
            decode_responses=False,
        )
        self._r.ping()
        log.info(f"RealtimeCacheClient connected to {host}:{port} (timeout={timeout_ms}ms)")

    def read_range(
        self, user_id: int, min_time_ms: int, max_time_ms: int
    ) -> list[tuple[bytes, float]]:
        return self._r.zrangebyscore(f"ua_rt:{user_id}", min_time_ms, max_time_ms, withscores=True)


def _try_import_user_action_proto():
    try:
        from proto_gen.recsys_pb2 import UserAction

        return UserAction
    except ImportError:
        return None


_USER_ACTION_PROTO = _try_import_user_action_proto()
if _USER_ACTION_PROTO is None:
    log.warning(
        "proto_gen.recsys_pb2 unavailable — protobuf realtime members will "
        "be skipped (Arrow members still merge)"
    )


def _proto_member_to_row(member: bytes) -> dict | None:
    if _USER_ACTION_PROTO is None:
        return None
    try:
        ua = _USER_ACTION_PROTO()
        ua.ParseFromString(member)
    except Exception:
        return None
    if not ua.actionInfo.engagementTimeMs and not ua.actionInfo.actionName:
        return None
    row = {}
    for field, extract in _PROTO_FIELD_EXTRACTORS.items():
        try:
            row[field] = extract(ua)
        except Exception:
            pass
    return row


def _arrow_member_to_rows(member: bytes) -> list[dict]:
    import pyarrow as pa

    try:
        raw = _zstd_decompress(member) if member[:4] == _ZSTD_MAGIC else member
        table = pa.ipc.RecordBatchFileReader(pa.py_buffer(raw)).read_all()
        actions = table.column("user_actions").combine_chunks().values
        return actions.to_pylist()
    except Exception:
        return []


def _member_to_rows(member: bytes, score_ms: float) -> list[dict]:
    if not member:
        return []
    if member[:4] == _ZSTD_MAGIC or member[:6] == _ARROW_MAGIC:
        rows = _arrow_member_to_rows(member)
    else:
        row = _proto_member_to_row(member)
        rows = [row] if row is not None else []
    for r in rows:
        r.setdefault("action_added_to_sequence_epoch_ms", int(score_ms))
    return rows


def _coerce_row_to_struct_type(row: dict, struct_type) -> dict:
    import pyarrow.types as pt

    out = {}
    for i in range(struct_type.num_fields):
        f = struct_type.field(i)
        if f.name not in row:
            continue
        v = row[f.name]
        if v is None:
            continue
        try:
            if pt.is_integer(f.type):
                out[f.name] = int(v)
            elif pt.is_floating(f.type):
                out[f.name] = float(v)
            elif pt.is_boolean(f.type):
                out[f.name] = bool(v)
            elif pt.is_string(f.type) or pt.is_large_string(f.type):
                out[f.name] = str(v)
            else:
                out[f.name] = v
        except (TypeError, ValueError):
            continue
    return out


def _dedup_key(row: dict) -> tuple:
    return (
        int(row.get("engagement_time_ms") or 0),
        int(row.get("action_name") or 0),
        int(row.get("tweet_id") or 0),
        int(row.get("author_id") or 0),
    )


def merge_realtime_into_sequence(
    raw_bytes: bytes,
    rt_client: RealtimeCacheClient,
    user_id: int,
    headroom_ms: int = 60_000,
) -> tuple[bytes, int]:
    import numpy as np
    import pyarrow as pa
    import pyarrow.compute as pc
    import zstandard as _zst

    decompressed = _zstd_decompress(raw_bytes)
    table = pa.ipc.RecordBatchFileReader(pa.py_buffer(decompressed)).read_all()
    actions_list = table.column("user_actions").combine_chunks()
    base_struct = actions_list.values
    struct_type = base_struct.type

    last_seq_time = 0
    if "last_sequence_time" in table.column_names:
        v = table.column("last_sequence_time")[0].as_py()
        last_seq_time = int(v) if v else 0
    if last_seq_time == 0 and len(base_struct) > 0:
        try:
            et = base_struct.field("engagement_time_ms")
            m = pc.max(et).as_py()
            last_seq_time = int(m) if m else 0
        except KeyError:
            pass

    now_ms = int(time.time() * 1000)
    members = rt_client.read_range(user_id, max(0, last_seq_time - headroom_ms), now_ms)
    if not members:
        return raw_bytes, 0

    rows = []
    for member, score in members:
        rows.extend(_member_to_rows(member, score))
    if not rows:
        return raw_bytes, 0

    try:
        et_np = base_struct.field("engagement_time_ms").to_numpy(zero_copy_only=False)
        overlap_idx = np.where(et_np >= (last_seq_time - headroom_ms))[0]
        existing_keys = set()
        if len(overlap_idx) > 0:
            an = base_struct.field("action_name").to_numpy(zero_copy_only=False)
            ti = base_struct.field("tweet_id").to_numpy(zero_copy_only=False)
            ai = base_struct.field("author_id").to_numpy(zero_copy_only=False)
            for i in overlap_idx:
                existing_keys.add((int(et_np[i]), int(an[i]), int(ti[i]), int(ai[i])))
    except KeyError:
        existing_keys = set()

    new_rows, seen = [], set()
    for row in rows:
        k = _dedup_key(row)
        if k in existing_keys or k in seen:
            continue
        seen.add(k)
        new_rows.append(_coerce_row_to_struct_type(row, struct_type))
    if not new_rows:
        return raw_bytes, 0

    rt_struct = pa.array(new_rows, type=struct_type)
    merged = pa.concat_arrays([base_struct, rt_struct])
    order = pc.sort_indices(merged.field("engagement_time_ms"))
    merged = merged.take(order)

    new_list = pa.ListArray.from_arrays(pa.array([0, len(merged)], type=pa.int32()), merged)
    col_idx = table.column_names.index("user_actions")
    new_table = table.set_column(col_idx, table.schema.field(col_idx).with_nullable(True), new_list)

    sink = pa.BufferOutputStream()
    with pa.ipc.new_file(sink, new_table.schema) as writer:
        writer.write_table(new_table)
    out = _zst.ZstdCompressor(level=1).compress(sink.getvalue().to_pybytes())
    return out, len(new_rows)
