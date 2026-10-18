use xai_safety_label_store::types::{decode_lkey_bytes, SafetyLabelMap};
use xai_visibility_filtering_proto as vf_pb;
use xai_x_thrift::tweet_safety_label::SafetyLabel;

use super::proto;

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use tracing::debug;

#[derive(Debug, Clone)]
pub struct LkeyBytes(pub [u8; 4]);

#[derive(Debug, Clone)]
pub struct MvalBytes(pub Vec<u8>);

#[derive(Debug, Clone)]
pub struct RawSafetyLabel {
    pub lkey: LkeyBytes,
    pub mval: MvalBytes,
}

pub fn decode_mval_payload(bytes: &[u8]) -> Option<vf_pb::SafetyLabelMap> {
    xai_safety_label_store::mval_safety_label_map::decode_mval(bytes).map(proto::label_map_to_proto)
}

pub(crate) enum DecodeAttempt {
    Success(vf_pb::SafetyLabelMap),
    Panic,
}

pub(crate) fn decode_raw_labels(items: &[RawSafetyLabel]) -> DecodeAttempt {
    contain_decode_panic(|| decode_raw_labels_inner(items))
}

fn contain_decode_panic(decode: impl FnOnce() -> SafetyLabelMap) -> DecodeAttempt {
    match catch_unwind(AssertUnwindSafe(decode)) {
        Ok(map) => DecodeAttempt::Success(proto::label_map_to_proto(map)),
        Err(_) => DecodeAttempt::Panic,
    }
}

fn decode_raw_labels_inner(items: &[RawSafetyLabel]) -> SafetyLabelMap {
    let mut map = HashMap::with_capacity(items.len());
    for item in items {
        let label_type = decode_lkey_bytes(item.lkey.0);
        let label = match xai_x_thrift::deserialize_mval(&item.mval.0) {
            Ok(l) => l,
            Err(e) => {
                debug!(label_type = ?label_type, error = %e, "skipping label value with corrupt mval from manhattan");
                SafetyLabel::default()
            }
        };
        map.insert(label_type, label);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_safety_label_store::types::encode_lkey;
    use xai_x_thrift::tweet_safety_label::SafetyLabelType;

    fn minimal_mval() -> Vec<u8> {
        vec![0x0C, 0x00, 0x04, 0x00, 0x00]
    }

    fn mval_with_score() -> Vec<u8> {
        let mut buf = vec![0x0C, 0x00, 0x04];
        buf.extend_from_slice(&[0x04, 0x00, 0x01]);
        buf.extend_from_slice(&0.9f64.to_bits().to_be_bytes());
        buf.push(0x00);
        buf.push(0x00);
        buf
    }

    fn raw_label(lt: SafetyLabelType, mval: Vec<u8>) -> RawSafetyLabel {
        RawSafetyLabel {
            lkey: LkeyBytes(encode_lkey(lt)),
            mval: MvalBytes(mval),
        }
    }

    #[track_caller]
    fn decoded(items: &[RawSafetyLabel]) -> vf_pb::SafetyLabelMap {
        match decode_raw_labels(items) {
            DecodeAttempt::Success(map) => map,
            DecodeAttempt::Panic => panic!("expected Success, got Panic"),
        }
    }

    #[test]
    fn decode_raw_labels_contains_panics() {
        assert!(matches!(
            contain_decode_panic(|| panic!("test decode panic")),
            DecodeAttempt::Panic
        ));
    }

    #[test]
    fn decode_raw_labels_preserves_label_type_on_corrupt_mval() {
        let items = vec![
            raw_label(SafetyLabelType::SPAM, minimal_mval()),
            raw_label(SafetyLabelType::BOUNCE, vec![0xDE, 0xAD]),
            raw_label(SafetyLabelType::NSFW_HIGH_PRECISION, mval_with_score()),
        ];
        let map = decoded(&items);
        assert_eq!(map.labels.len(), 3);
        assert!(map.labels.contains_key(&i32::from(SafetyLabelType::SPAM)));
        assert!(map.labels.contains_key(&i32::from(SafetyLabelType::BOUNCE)));
        assert!(map
            .labels
            .contains_key(&i32::from(SafetyLabelType::NSFW_HIGH_PRECISION)));
        assert_eq!(
            map.labels[&i32::from(SafetyLabelType::BOUNCE)],
            vf_pb::SafetyLabel::default()
        );
    }
}
