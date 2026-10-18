# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import base64
import logging
import pathlib

from xai_checkpointing.dek import publish_tree_dek

rank_logger = logging.getLogger("rank")


def at_dir(kvstore: dict, path: pathlib.Path | str) -> dict:
    inner = kvstore.get("base")
    root = inner.get("path", "/") if isinstance(inner, dict) else "/"
    rel = pathlib.Path(path).relative_to(root)
    scoped = dict(kvstore)
    if rel != pathlib.Path("."):
        scoped["path"] = scoped.get("path", "") + rel.as_posix() + "/"
    return scoped


def use_encrypted_kvstore(ocdbt_config: dict, kvstore: dict) -> dict:
    local = ocdbt_config["base"]
    assert local["driver"] == "file", local
    return {**ocdbt_config, "base": at_dir(kvstore, local["path"])}


def _envelope_spec(
    checkpoint_root: pathlib.Path | str,
    dek_b64: str,
    wrapped_dek: str,
    chunk_size: int,
    encryption_context: dict,
) -> dict:
    base = {
        "driver": "xai_encrypted",
        "base": {"driver": "file", "path": pathlib.Path(checkpoint_root).as_posix() + "/"},
        "dek_b64": dek_b64,
        "wrapped_dek": wrapped_dek,
        "chunk_size": chunk_size,
    }
    if encryption_context:
        base["encryption_context"] = encryption_context
    return base


def encrypted_kvstore(path: pathlib.Path, kms_client, encryption_chunk_size: int) -> dict:
    path.mkdir(parents=True, exist_ok=True)
    raw, wrapped, context, entry = publish_tree_dek(path, kms_client)
    rank_logger.info(
        "KMS unwrap OK for %s (key_id=%s)",
        path,
        entry.get("key_id"),
    )
    return _envelope_spec(
        path, base64.b64encode(raw).decode(), wrapped, encryption_chunk_size, context
    )
