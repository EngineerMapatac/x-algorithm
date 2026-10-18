use anyhow::{Context, Result};
use lazy_static::lazy_static;
use xai_manhattan::{Item, ManhattanClient, Tenant};
use xai_uas_thrift::deserialize_from_bytes_binary;
use xai_uas_thrift::user_action_sequence::CompressionType as TCompressionType;
use xai_uas_thrift::user_action_sequence::UserActionSequence as TUserActionSequence;
use xai_uas_thrift::user_action_sequence::UserActionSequenceDataContainer as TUserActionSequenceDataContainer;

lazy_static! {
    pub static ref MH_ENDPOINT: String =
        "http://manhattan.service.capi-prod-dp.kube.atla.twitter.biz:80".to_string();
    pub static ref UAS_TENANT: Tenant = Tenant {
        cluster: "nash".to_string(),
        app_id: "user_actions".to_string(),
        dataset: "user_action_sequence_xai".to_string(),
    };
    pub static ref TIMEOUT_MS: u64 = 500;
}

pub trait UserActionSequenceOps {
    fn get_by_user_id(&self, user_id: i64) -> impl Future<Output = Result<TUserActionSequence>>;
}

pub struct UserActionSequenceFetcher {
    mh_client: ManhattanClient,
}

impl UserActionSequenceFetcher {
    pub fn new() -> Result<Self> {
        let mh_client = ManhattanClient::new_with_addr_timeout(&MH_ENDPOINT, *TIMEOUT_MS);
        let mh_client = mh_client.context("Failed to connect to Manhattan")?;
        Ok(Self { mh_client })
    }
}

impl UserActionSequenceOps for UserActionSequenceFetcher {
    async fn get_by_user_id(&self, user_id: i64) -> Result<TUserActionSequence> {
        let pkey: Vec<Vec<u8>> = vec![user_id.to_be_bytes().to_vec()];
        let lkey: Vec<Vec<u8>> = vec![];
        let item: Option<Item> = self
            .mh_client
            .clone()
            .get(UAS_TENANT.clone(), pkey, lkey)
            .await
            .context("Failed to fetch data from Manhattan")?;
        match item {
            Some(item) => {
                let value = item.value();
                let value = value.as_bytes();

                let uas = deserialize_from_bytes_binary::<TUserActionSequence>(value)
                    .context("Failed to deserialize user action sequence")?;

                Ok(maybe_decompress(uas)?)
            }
            None => anyhow::bail!("No user action sequence found for user_id {}", user_id),
        }
    }
}

fn maybe_decompress(uas: TUserActionSequence) -> Result<TUserActionSequence> {
    if let Some(TUserActionSequenceDataContainer::CompressedUserActions(compressed)) =
        uas.user_actions_data.as_deref()
        && compressed.compression_type == Some(TCompressionType::ZSTD)
        && let Some(data) = &compressed.data
    {
        let decompressed = zstd::decode_all(&data[..])?;
        return deserialize_from_bytes_binary::<TUserActionSequence>(&decompressed)
            .context("Failed to deserialize decompressed user action sequence");
    }
    Ok(uas)
}

#[cfg(test)]
mod tests {
    use crate::clients::uas_fetcher::*;
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_mh_connection() {
        if let Ok(uas_fetcher) = UserActionSequenceFetcher::new() {
            let result = uas_fetcher.get_by_user_id(12);
            println!("Result: {:?}", result.await);
        }
    }
}
