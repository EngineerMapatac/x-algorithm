namespace java com.twitter.cortex_media_annotator.thriftjava

include "com/twitter/cortex_media_annotator/annotate.thrift"

struct AnnotateRequestResponse {
  1: annotate.AnnotationRequest request
  2: annotate.AnnotationResponse response
}(persisted='true')
