use thrift::protocol::{TOutputProtocol, TSerializable};

pub use super::r#gen::hydra::{
    EngagementCount, MultiModalEmbeddings, PhoenixRankAllCandidateFlat, PhoenixRankAllObject,
};
pub use super::r#gen::scored_candidate::{PredictionScore, ScoredCandidate};

pub fn deserialize_binary<T: TSerializable>(data: &[u8]) -> anyhow::Result<T> {
    use thrift::protocol::TBinaryInputProtocol;
    use thrift::transport::TBufferChannel;

    let mut transport = TBufferChannel::with_capacity(data.len(), 0);
    transport.set_readable_bytes(data);
    let mut protocol = TBinaryInputProtocol::new(transport, false);
    let value = T::read_from_in_protocol(&mut protocol)?;
    Ok(value)
}

pub fn deserialize_compact<T: TSerializable>(data: &[u8]) -> anyhow::Result<T> {
    use thrift::protocol::TCompactInputProtocol;
    use thrift::transport::TBufferChannel;

    let mut transport = TBufferChannel::with_capacity(data.len(), 0);
    transport.set_readable_bytes(data);
    let mut protocol = TCompactInputProtocol::new(transport);
    let value = T::read_from_in_protocol(&mut protocol)?;
    Ok(value)
}

pub fn serialize_binary<T: TSerializable>(value: &T) -> anyhow::Result<Vec<u8>> {
    use thrift::protocol::TBinaryOutputProtocol;

    let mut buffer = Vec::new();
    let mut protocol = TBinaryOutputProtocol::new(&mut buffer, false);
    value.write_to_out_protocol(&mut protocol)?;
    protocol.flush()?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phoenix_rank_all_object_roundtrip() {
        let obj = PhoenixRankAllObject::new(
            Some(1234567890123456789_i64),
            Some(9876543210_i64),
            Some("1fav".to_string()),
            None::<Vec<i64>>,
        );

        let bytes = serialize_binary(&obj).unwrap();
        let decoded: PhoenixRankAllObject = deserialize_binary(&bytes).unwrap();

        assert_eq!(decoded.post_id, Some(1234567890123456789));
        assert_eq!(decoded.author_id, Some(9876543210));
        assert_eq!(decoded.index_name, Some("1fav".to_string()));
        assert!(decoded.topic_entity_ids.is_none());
    }

    #[test]
    fn phoenix_rank_all_object_with_topics() {
        let obj = PhoenixRankAllObject::new(
            Some(100_i64),
            Some(200_i64),
            Some("1fav_topic".to_string()),
            Some(vec![1925949634972626944_i64, 1925949452197478400_i64]),
        );

        let bytes = serialize_binary(&obj).unwrap();
        let decoded: PhoenixRankAllObject = deserialize_binary(&bytes).unwrap();

        let topics = decoded.topic_entity_ids.unwrap();
        assert_eq!(topics.len(), 2);
        assert_eq!(topics[0], 1925949634972626944);
    }

    #[test]
    fn candidate_flat_roundtrip() {
        let ec = EngagementCount::new(
            Some(10_i64),
            Some(5_i64),
            Some(100_i64),
            None::<i64>,
            None::<i64>,
            None::<i64>,
            None::<i64>,
            None::<i64>,
            None::<i64>,
        );
        let obj = PhoenixRankAllCandidateFlat::new(
            Some(111_i64),
            Some(222_i64),
            Some(true),
            Some(false),
            Some(50000_i64),
            Some(ec),
            Some(30000_i64),
            None::<MultiModalEmbeddings>,
            Some("metadata".to_string()),
        );

        let bytes = serialize_binary(&obj).unwrap();
        let decoded: PhoenixRankAllCandidateFlat = deserialize_binary(&bytes).unwrap();

        assert_eq!(decoded.post_id, Some(111));
        assert_eq!(decoded.author_id, Some(222));
        assert_eq!(decoded.has_video, Some(true));
        assert_eq!(decoded.has_image, Some(false));
        assert_eq!(decoded.author_followers_count, Some(50000));
        assert_eq!(decoded.video_duration_ms, Some(30000));
        assert_eq!(decoded.index_name, Some("metadata".to_string()));

        let ec = decoded.engagement_count.unwrap();
        assert_eq!(ec.retweet_count, Some(10));
        assert_eq!(ec.reply_count, Some(5));
        assert_eq!(ec.favorite_count, Some(100));
        assert_eq!(ec.quote_count, None);
    }

    #[test]
    fn scored_candidate_roundtrip() {
        let scores = vec![PredictionScore::new(
            Some("phoenix.HomeExperiment1Lap7.engagement".to_string()),
            Some(thrift::OrderedFloat::from(0.95)),
        )]
        .into_iter()
        .map(Box::new)
        .collect();

        let obj = ScoredCandidate::new(
            12345_i64,
            Some(67890_i64),
            Some(11111_i64),
            None::<thrift::OrderedFloat<f64>>,
            Some("RecTweet".to_string()),
            None::<i64>,
            None::<i64>,
            Some(scores),
        );

        let bytes = serialize_binary(&obj).unwrap();
        let decoded: ScoredCandidate = deserialize_binary(&bytes).unwrap();

        assert_eq!(decoded.tweet_id, 12345);
        assert_eq!(decoded.viewer_id, Some(67890));
        assert_eq!(decoded.author_id, Some(11111));
        assert_eq!(decoded.suggest_type, Some("RecTweet".to_string()));

        let scores = decoded.prediction_scores.unwrap();
        assert_eq!(scores.len(), 1);
    }
}
