namespace java com.twitter.cortex_media_annotator.thriftjava
namespace py com.twitter.cortex_media_annotator.annotate

include "com/twitter/mediaservices/commons/MediaCommon.thrift"
include "com/twitter/mediaservices/commons/ServerCommon.thrift"
include "com/twitter/ml/api/data.thrift"

enum MediaModel {
  XX_NSFW,
  FINGERPRINTING,
  CATEGORY_CLASSIFICATION,
  HATEFUL_SYMBOL_RECOGNITION,
  XX_NSFW_V2,
  FINGERPRINTING_V2,
  VIOLENCE_AND_GORE,
  CATEGORY_CLASSIFICATION_EMBEDDINGS,
  RESERVED_9,
  RESERVED_10,
  RESERVED_11,
  RESERVED_12,
  RESERVED_13,
  RESERVED_14,
  RESERVED_15,
  RESERVED_16,
  RESERVED_17,
  RESERVED_18,
  RESERVED_19,
  RESERVED_20
}(persisted='true')

struct MediaEntity {
  1: optional MediaCommon.MediaCategory category

  2: optional string blobstore_path

  3: optional string media_id(personalDataType = 'MediaId')

  4: optional binary media_data(personalDataType = 'MediaFile')

  5: optional string original_blobstore_path

  6: optional string broadcast_id(personalDataType = 'BroadcastId')
}(persisted='true', hasPersonalData = 'true')

struct AnnotationRequest {
  1: MediaEntity entity
  2: set<MediaModel> models
}(persisted='true')

struct AnnotationResponse {
  1: map<MediaModel, data.DataRecord> datarecord
  2: optional map<MediaModel, list<data.DataRecord>> annotations
}(persisted='true')

enum BadRequestCode {
  MEDIA_NOT_FOUND,
  INVALID_INPUT,
  FILE_TOO_LARGE,
  MEDIA_NOT_SUPPORTED,
  RESERVED_2,
  RESERVED_3,
  RESERVED_4,
  RESERVED_5,
  RESERVED_6,
  RESERVED_7,
  RESERVED_8,
  RESERVED_9
}

exception BadRequest {
  1: string message
  2: optional BadRequestCode code
}

service CortexMediaAnnotator {
  AnnotationResponse getAnnotations(1: AnnotationRequest request)
    throws (1: ServerCommon.ServerError serverError, 2: BadRequest badRequest)
}
