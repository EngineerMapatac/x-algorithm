use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use xai_recsys_proto::AdIndexInfo;
use xai_urt_thrift::metadata::AdindexDetails;
use xai_x_thrift::controller_data as cd;

pub(super) fn post_controller_data(
    tweet_type_metrics: &[u8],
    served_time_ms: u64,
    position: i32,
    prediction_request_id: u64,
) -> Option<String> {
    let tweet_types_list_bytes: Option<Vec<i8>> = if tweet_type_metrics.is_empty() {
        None
    } else {
        Some(tweet_type_metrics.iter().map(|&b| b as i8).collect())
    };

    let v1 = cd::HomeTweetsControllerDataV1 {
        tweet_types_bitmap: 0,
        tweet_types_bitmap_continued1: None,
        trace_id: None,
        served_time: if served_time_ms != 0 {
            Some(served_time_ms as i64)
        } else {
            None
        },
        injected_position: Some(position),
        tweet_types_list_bytes,
        inferred_topic_ids: None,
        served_id: None,
        request_join_id: None,
        prediction_id: if prediction_request_id != 0 {
            Some(prediction_request_id as i64)
        } else {
            None
        },
    };

    serialize_b64(&cd::ControllerData::V2(cd::ControllerDataV2::HomeTweets(
        cd::HomeTweetsControllerData::V1(v1),
    )))
}

pub(super) fn ad_item_controller_data() -> Option<String> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    post_controller_data(&[], now_ms, 0, 0)
}

pub(super) fn ad_index_controller_data(ad: &AdIndexInfo) -> Option<AdindexDetails> {
    let proto_cd = ad.controller_data.as_ref()?;
    let val = cd::ControllerData::V2(cd::ControllerDataV2::AdindexControllerData(
        cd::AdindexControllerData::V1(cd::AdindexControllerDataV1 {
            candidate_source: Some(proto_cd.candidate_source),
            exp_bucket: Some(proto_cd.exp_bucket.clone()),
            placeholder: Some(proto_cd.placeholder.clone()),
            line_item_id: Some(ad.line_item_id),
        }),
    ));
    Some(AdindexDetails {
        serialized_controller_data: serialize_b64(&val),
    })
}

pub(super) fn wtf_controller_data(tracking_token: &str) -> Option<String> {
    let bytes = BASE64_STANDARD.decode(tracking_token).ok()?;
    let token: cd::HermitTrackingToken = xai_x_thrift::deserialize_binary(&bytes).ok()?;
    serialize_b64(&token.controller_data?)
}

fn serialize_b64(val: &cd::ControllerData) -> Option<String> {
    let bytes = xai_x_thrift::serialize_binary(val).ok()?;
    Some(BASE64_STANDARD.encode(&bytes))
}
