use base64::{Engine as _, engine::general_purpose::STANDARD};
use prost::Message;
use tonic::metadata::MetadataMap;

const STRATO_CONTEXT_KEY: &str = "stratocontext";
const STRATO_CONTEXT_BIN_KEY: &str = "stratocontext-bin";

#[derive(Clone, PartialEq, prost::Message)]
pub struct StratoContext {
            #[prost(string, tag = "8")]
    pub ad_id: String,
    #[prost(bool, tag = "11")]
    pub is_polling: bool,
        #[prost(string, tag = "12")]
    pub mobile_device_id: String,
}

pub fn parse(metadata: &MetadataMap) -> Option<StratoContext> {
    if let Some(value) = metadata.get(STRATO_CONTEXT_KEY)
        && let Ok(s) = value.to_str()
        && let Ok(bytes) = STANDARD.decode(s.trim())
        && let Ok(ctx) = StratoContext::decode(bytes.as_slice())
    {
        return Some(ctx);
    }
    if let Some(value) = metadata.get_bin(STRATO_CONTEXT_BIN_KEY)
        && let Ok(bytes) = value.to_bytes()
        && let Ok(ctx) = StratoContext::decode(bytes.as_ref())
    {
        return Some(ctx);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;

    fn encode_ascii(ctx: &StratoContext) -> MetadataMap {
        let mut map = MetadataMap::new();
        let encoded = STANDARD.encode(ctx.encode_to_vec());
        map.insert(STRATO_CONTEXT_KEY, encoded.parse().unwrap());
        map
    }

    fn polling(ctx: StratoContext) -> StratoContext {
        StratoContext {
            is_polling: true,
            ..ctx
        }
    }

    #[test]
    fn polling_true() {
        let ctx = parse(&encode_ascii(&polling(StratoContext::default()))).unwrap();
        assert!(ctx.is_polling);
    }

    #[test]
    fn polling_false() {
        let ctx = parse(&encode_ascii(&StratoContext::default())).unwrap();
        assert!(!ctx.is_polling);
    }

    #[test]
    fn binary_metadata() {
        let mut map = MetadataMap::new();
        map.insert_bin(
            STRATO_CONTEXT_BIN_KEY,
            MetadataValue::from_bytes(&polling(StratoContext::default()).encode_to_vec()),
        );
        assert!(parse(&map).unwrap().is_polling);
    }

    #[test]
    fn missing_context() {
        assert!(parse(&MetadataMap::new()).is_none());
    }

    #[test]
    fn decodes_device_ids() {
        let ctx = parse(&encode_ascii(&StratoContext {
            ad_id: "ad-1".into(),
            is_polling: false,
            mobile_device_id: "dev-1".into(),
        }))
        .unwrap();
        assert_eq!(ctx.ad_id, "ad-1");
        assert_eq!(ctx.mobile_device_id, "dev-1");
    }
}
