from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import runtime_version as _runtime_version
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder

_runtime_version.ValidateProtobufRuntimeVersion(
    _runtime_version.Domain.PUBLIC, 6, 31, 1, "", "abuse_inference.proto"
)

_sym_db = _symbol_database.Default()


DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(
    b'\n\x15\x61\x62use_inference.proto\x12\x16xai.abuse.inference.v1"I\n\x0b\x41\x63tionCount\x12\x16\n\x0e\x61\x63tion_type_id\x18\x01 \x01(\x05\x12\r\n\x05\x63ount\x18\x02 \x01(\r\x12\x13\n\x0b\x61\x63tion_name\x18\x03 \x01(\t";\n\tFiredHead\x12\x0c\n\x04name\x18\x01 \x01(\t\x12\r\n\x05score\x18\x02 \x01(\x01\x12\x11\n\tthreshold\x18\x03 \x01(\x01"\xbf\x03\n\x0fSummaryCounters\x12\x13\n\x0breason_code\x18\x01 \x01(\t\x12\x15\n\rtotal_actions\x18\x02 \x01(\r\x12\x12\n\nserver_pct\x18\x03 \x01(\r\x12\x0f\n\x07renders\x18\x04 \x01(\r\x12\x0e\n\x06\x64wells\x18\x05 \x01(\r\x12\x14\n\x0c\x61vg_dwell_ms\x18\x06 \x01(\r\x12\x1d\n\x15\x64istinct_action_types\x18\x07 \x01(\r\x12\x18\n\x10\x64istinct_targets\x18\x08 \x01(\r\x12\x16\n\x0e\x64istinct_posts\x18\t \x01(\r\x12\x13\n\x0b\x61pi_follows\x18\n \x01(\r\x12\x13\n\x0b\x61pi_replies\x18\x0b \x01(\r\x12\x10\n\x08\x61pi_favs\x18\x0c \x01(\r\x12\x14\n\x0c\x61pi_retweets\x18\r \x01(\r\x12\x15\n\rclient_clicks\x18\x0e \x01(\r\x12\x1b\n\x13\x63lient_photo_expand\x18\x0f \x01(\r\x12\x16\n\x0e\x63lient_follows\x18\x10 \x01(\r\x12\x0e\n\x06labels\x18\x11 \x03(\t\x12\x36\n\x0b\x66ired_heads\x18\x12 \x03(\x0b\x32!.xai.abuse.inference.v1.FiredHead"\x80\x03\n\x13SequenceUpdateEvent\x12\x0f\n\x07user_id\x18\x01 \x01(\x03\x12\x13\n\x0bwrite_ts_ms\x18\x02 \x01(\x03\x12\x1b\n\x13latest_action_ts_ms\x18\x03 \x01(\x03\x12\x18\n\x10sequence_version\x18\x04 \x01(\x03\x12\x1c\n\x14sequence_fingerprint\x18\x05 \x01(\t\x12\x14\n\x0csequence_len\x18\x06 \x01(\r\x12\x1a\n\x12\x64\x65lta_action_count\x18\x07 \x01(\r\x12\x19\n\x11reply_count_delta\x18\x08 \x01(\r\x12\x1a\n\x12\x66ollow_count_delta\x18\t \x01(\r\x12\x1a\n\x12report_count_delta\x18\n \x01(\r\x12\x1a\n\x12render_count_delta\x18\x0b \x01(\r\x12\x19\n\x11\x64well_count_delta\x18\x0c \x01(\r\x12\x18\n\x10prior_score_tier\x18\r \x01(\t\x12\x18\n\x10producer_version\x18\x0e \x01(\t"\xec\x01\n\x0cScoreRequest\x12\x0f\n\x07user_id\x18\x01 \x01(\x03\x12\x1c\n\x14target_model_version\x18\x02 \x01(\t\x12\x16\n\x0equeue_priority\x18\x03 \x01(\t\x12\x16\n\x0e\x65nqueue_reason\x18\x04 \x01(\t\x12\x15\n\renqueue_ts_ms\x18\x05 \x01(\x03\x12\x18\n\x10sequence_version\x18\x06 \x01(\x03\x12\x1c\n\x14sequence_fingerprint\x18\x07 \x01(\t\x12\x17\n\x0ftrigger_sources\x18\x08 \x03(\t\x12\x15\n\rforce_rescore\x18\t \x01(\x08"\xb4\x01\n\rDecodedAction\x12\x0b\n\x03idx\x18\x01 \x01(\r\x12\x13\n\x0b\x61\x63tion_type\x18\x02 \x01(\t\x12\x10\n\x08tweet_id\x18\x03 \x01(\x03\x12\x11\n\tauthor_id\x18\x04 \x01(\x03\x12\x1a\n\x12\x65ngagement_time_ms\x18\x05 \x01(\x03\x12\x10\n\x08\x64well_ms\x18\x06 \x01(\x01\x12\x17\n\x0fproduct_surface\x18\x07 \x01(\t\x12\x15\n\rclient_app_id\x18\x08 \x01(\x03"\xf4\x04\n\x0bScoreResult\x12\x0f\n\x07user_id\x18\x01 \x01(\x03\x12\x15\n\rmodel_version\x18\x02 \x01(\t\x12\x13\n\x0bscore_ts_ms\x18\x03 \x01(\x03\x12\x14\n\x0cscore_status\x18\x04 \x01(\t\x12\x16\n\x0equeue_priority\x18\x05 \x01(\t\x12\x16\n\x0e\x65nqueue_reason\x18\x06 \x01(\t\x12\x1b\n\x13latest_action_ts_ms\x18\x07 \x01(\x03\x12\x1f\n\x17scored_sequence_version\x18\x08 \x01(\x03\x12#\n\x1bscored_sequence_fingerprint\x18\t \x01(\t\x12\x14\n\x0csequence_len\x18\n \x01(\r\x12\x17\n\x0fraw_head_scores\x18\x0b \x03(\x01\x12\x38\n\x07summary\x18\x0c \x01(\x0b\x32\'.xai.abuse.inference.v1.SummaryCounters\x12\x43\n\x16\x61\x63tion_counts_non_zero\x18\r \x03(\x0b\x32#.xai.abuse.inference.v1.ActionCount\x12\x19\n\x11scorer_latency_ms\x18\x0e \x01(\r\x12!\n\x19manhattan_read_latency_ms\x18\x0f \x01(\r\x12 \n\x18\x66\x65\x61ture_build_latency_ms\x18\x10 \x01(\r\x12\x17\n\x0ftrigger_sources\x18\x11 \x03(\t\x12>\n\x0f\x64\x65\x63oded_actions\x18\x12 \x03(\x0b\x32%.xai.abuse.inference.v1.DecodedAction\x12\x18\n\x10\x65nforcement_note\x18\x13 \x01(\t"\xc0\x03\n\x14TriggerActivityState\x12\x0f\n\x07user_id\x18\x01 \x01(\x03\x12\x1c\n\x14\x61\x63tive_model_version\x18\x02 \x01(\t\x12&\n\x1elast_seen_sequence_fingerprint\x18\x03 \x01(\t\x12\x1d\n\x15last_seen_write_ts_ms\x18\x04 \x01(\x03\x12%\n\x1dlast_seen_latest_action_ts_ms\x18\x05 \x01(\x03\x12\x1e\n\x16last_seen_sequence_len\x18\x06 \x01(\r\x12!\n\x19last_enqueued_fingerprint\x18\x07 \x01(\t\x12\x1a\n\x12last_enqueue_ts_ms\x18\x08 \x01(\x03\x12\x1b\n\x13last_enqueue_reason\x18\t \x01(\t\x12\x1b\n\x13last_queue_priority\x18\n \x01(\t\x12\x1f\n\x17last_scored_fingerprint\x18\x0b \x01(\t\x12\x18\n\x10last_score_ts_ms\x18\x0c \x01(\x03\x12\x16\n\x0elast_risk_tier\x18\r \x01(\t\x12\x1f\n\x17last_state_update_ts_ms\x18\x0e \x01(\x03"\xe1\x01\n\x17RiskTierTransitionEvent\x12\x0f\n\x07user_id\x18\x01 \x01(\x03\x12\x15\n\rmodel_version\x18\x02 \x01(\t\x12\x1a\n\x12previous_risk_tier\x18\x03 \x01(\t\x12\x15\n\rnew_risk_tier\x18\x04 \x01(\t\x12\x16\n\x0e\x61ggregate_risk\x18\x05 \x01(\x01\x12#\n\x1bscored_sequence_fingerprint\x18\x06 \x01(\t\x12\x13\n\x0bscore_ts_ms\x18\x07 \x01(\x03\x12\x19\n\x11transition_reason\x18\x08 \x01(\tb\x06proto3'
)

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, "abuse_inference_pb2", _globals)
if not _descriptor._USE_C_DESCRIPTORS:
    DESCRIPTOR._loaded_options = None
    _globals["_ACTIONCOUNT"]._serialized_start = 49
    _globals["_ACTIONCOUNT"]._serialized_end = 122
    _globals["_FIREDHEAD"]._serialized_start = 124
    _globals["_FIREDHEAD"]._serialized_end = 183
    _globals["_SUMMARYCOUNTERS"]._serialized_start = 186
    _globals["_SUMMARYCOUNTERS"]._serialized_end = 633
    _globals["_SEQUENCEUPDATEEVENT"]._serialized_start = 636
    _globals["_SEQUENCEUPDATEEVENT"]._serialized_end = 1020
    _globals["_SCOREREQUEST"]._serialized_start = 1023
    _globals["_SCOREREQUEST"]._serialized_end = 1259
    _globals["_DECODEDACTION"]._serialized_start = 1262
    _globals["_DECODEDACTION"]._serialized_end = 1442
    _globals["_SCORERESULT"]._serialized_start = 1445
    _globals["_SCORERESULT"]._serialized_end = 2073
    _globals["_TRIGGERACTIVITYSTATE"]._serialized_start = 2076
    _globals["_TRIGGERACTIVITYSTATE"]._serialized_end = 2524
    _globals["_RISKTIERTRANSITIONEVENT"]._serialized_start = 2527
    _globals["_RISKTIERTRANSITIONEVENT"]._serialized_end = 2752
