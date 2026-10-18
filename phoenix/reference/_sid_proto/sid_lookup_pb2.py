# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import runtime_version as _runtime_version
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder

_runtime_version.ValidateProtobufRuntimeVersion(
    _runtime_version.Domain.PUBLIC, 6, 31, 1, "", "sid_lookup.proto"
)

_sym_db = _symbol_database.Default()


DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(
    b'\n\x10sid_lookup.proto\x12\nsid_lookup"%\n\x11LookupSidsRequest\x12\x10\n\x08post_ids\x18\x01 \x03(\x03";\n\x12LookupSidsResponse\x12%\n\x07results\x18\x01 \x03(\x0b\x32\x14.sid_lookup.PostSids"\x19\n\x08PostSids\x12\r\n\x05\x63odes\x18\x01 \x03(\x05\x32_\n\x10SidLookupService\x12K\n\nLookupSids\x12\x1d.sid_lookup.LookupSidsRequest\x1a\x1e.sid_lookup.LookupSidsResponseb\x06proto3'
)

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, "sid_lookup_pb2", _globals)
if not _descriptor._USE_C_DESCRIPTORS:
    DESCRIPTOR._loaded_options = None
    _globals["_LOOKUPSIDSREQUEST"]._serialized_start = 32
    _globals["_LOOKUPSIDSREQUEST"]._serialized_end = 69
    _globals["_LOOKUPSIDSRESPONSE"]._serialized_start = 71
    _globals["_LOOKUPSIDSRESPONSE"]._serialized_end = 130
    _globals["_POSTSIDS"]._serialized_start = 132
    _globals["_POSTSIDS"]._serialized_end = 157
    _globals["_SIDLOOKUPSERVICE"]._serialized_start = 159
    _globals["_SIDLOOKUPSERVICE"]._serialized_end = 254
