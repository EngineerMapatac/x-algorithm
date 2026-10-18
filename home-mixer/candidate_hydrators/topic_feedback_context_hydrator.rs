use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use crate::params::EnableTopicFeedbackContext;
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::sync::Arc;
use thrift::protocol::{TInputProtocol, TOutputProtocol, TSerializable, TType};
use tonic::async_trait;
use tracing::warn;
use xai_candidate_pipeline::component_library::clients::StratoClient;
use xai_candidate_pipeline::hydrator::Hydrator;
use xai_recsys_proto::grok_topics::{topic_index, PARENT_TOPIC_IDS};
use xai_strato::strato_thrift::{strato_decode, StratoResult};

const FID_TOPICS: i16 = -9948;
const FID_ANNOTATIONS: i16 = 31524;
const TOP_TOPIC_COUNT: usize = 2;

pub struct TopicFeedbackContextHydrator {
    pub strato_client: Arc<dyn StratoClient + Send + Sync>,
}

impl TopicFeedbackContextHydrator {
    fn is_eligible_original_post(candidate: &PostCandidate) -> bool {
        candidate.in_reply_to_tweet_id.is_none()
            && candidate.retweeted_tweet_id.is_none()
            && candidate.ancestors.is_empty()
            && candidate.following_replied_user_ids.is_empty()
    }

    fn is_subtopic(topic_id: i64) -> bool {
        topic_index(topic_id).is_some() && !PARENT_TOPIC_IDS.contains(&topic_id)
    }

    fn decode_topics(bytes: &[u8]) -> (Vec<TopicAnnotation>, Vec<String>) {
        match strato_decode::<UnifiedPostAnnotationsValue>(bytes) {
            Ok(StratoResult::Ok {
                value: Some(tv), ..
            }) => {
                let annotations: Vec<TopicAnnotation> = tv
                    .annotations
                    .entities
                    .iter()
                    .enumerate()
                    .filter_map(|(i, entity)| {
                        let topic_id = entity.qualified_id.entity_id;
                        if topic_id == 0 || !Self::is_subtopic(topic_id) {
                            return None;
                        }
                        let topic_name = tv.topics.get(i)?.clone();
                        Some(TopicAnnotation {
                            topic_id,
                            topic_name,
                        })
                    })
                    .collect();

                (annotations, tv.topics)
            }
            Ok(StratoResult::Ok { value: None, .. }) | Ok(StratoResult::Err { .. }) => {
                (Vec::new(), Vec::new())
            }
            Err(e) => {
                warn!("TopicFeedbackContextHydrator: decode error: {}", e);
                (Vec::new(), Vec::new())
            }
        }
    }

    fn select_feedback_targets(
        candidate_topics: &[(usize, Vec<TopicAnnotation>)],
    ) -> HashMap<usize, (String, String)> {
        let mut frequency: HashMap<i64, usize> = HashMap::new();
        let mut name_by_id: HashMap<i64, String> = HashMap::new();
        for (_, topics) in candidate_topics {
            for topic in topics {
                *frequency.entry(topic.topic_id).or_default() += 1;
                name_by_id
                    .entry(topic.topic_id)
                    .or_insert_with(|| topic.topic_name.clone());
            }
        }

        let mut ranked: Vec<(i64, usize)> = frequency.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked.truncate(TOP_TOPIC_COUNT);

        let mut rng = rand::rng();
        let mut selected: HashMap<usize, (String, String)> = HashMap::new();
        let mut used_indices = std::collections::HashSet::new();

        for (topic_id, _) in ranked {
            let name = match name_by_id.get(&topic_id) {
                Some(n) => n.clone(),
                None => continue,
            };
            let candidates_for_topic: Vec<usize> = candidate_topics
                .iter()
                .filter(|(idx, topics)| {
                    !used_indices.contains(idx) && topics.iter().any(|t| t.topic_id == topic_id)
                })
                .map(|(idx, _)| *idx)
                .collect();
            let Some(&idx) = candidates_for_topic.choose(&mut rng) else {
                continue;
            };
            used_indices.insert(idx);
            selected.insert(idx, (name, topic_id.to_string()));
        }

        selected
    }
}

#[async_trait]
impl Hydrator<ScoredPostsQuery, PostCandidate> for TopicFeedbackContextHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableTopicFeedbackContext)
            && !query.is_topic_request()
            && !query.in_network_only
    }

    async fn hydrate(
        &self,
        _query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let eligible: Vec<(usize, u64)> = candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| Self::is_eligible_original_post(c))
            .map(|(i, c)| (i, c.tweet_id))
            .collect();

        let mut grok_topics_by_idx: HashMap<usize, Vec<String>> = HashMap::new();
        let selected = if eligible.is_empty() {
            HashMap::new()
        } else {
            let tweet_ids: Vec<u64> = eligible.iter().map(|(_, id)| *id).collect();
            let results = self
                .strato_client
                .batch_get_unified_post_annotations(&tweet_ids)
                .await;

            let mut candidate_topics: Vec<(usize, Vec<TopicAnnotation>)> = Vec::new();
            for ((idx, _), result) in eligible.into_iter().zip(results) {
                match result {
                    Ok(bytes) if !bytes.is_empty() => {
                        let (annotations, topic_names) = Self::decode_topics(&bytes);
                        if !topic_names.is_empty() {
                            grok_topics_by_idx.insert(idx, topic_names);
                        }
                        if !annotations.is_empty() {
                            candidate_topics.push((idx, annotations));
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!("TopicFeedbackContextHydrator: strato fetch error: {}", e);
                    }
                }
            }
            Self::select_feedback_targets(&candidate_topics)
        };

        candidates
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let (topic, topic_id) = match selected.get(&i) {
                    Some((t, id)) => (Some(t.clone()), Some(id.clone())),
                    None => (None, None),
                };
                Ok(PostCandidate {
                    topic_feedback_topic: topic,
                    topic_feedback_topic_id: topic_id,
                    grok_topics: grok_topics_by_idx.get(&i).cloned(),
                    ..Default::default()
                })
            })
            .collect()
    }

    fn update(&self, candidate: &mut PostCandidate, hydrated: PostCandidate) {
        candidate.topic_feedback_topic = hydrated.topic_feedback_topic;
        candidate.topic_feedback_topic_id = hydrated.topic_feedback_topic_id;
        candidate.grok_topics = hydrated.grok_topics;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TopicAnnotation {
    topic_id: i64,
    topic_name: String,
}

#[derive(Debug, Default, Clone)]
struct QualifiedId {
    entity_id: i64,
}

impl TSerializable for QualifiedId {
    fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Self> {
        i_prot.read_struct_begin()?;
        let mut q = QualifiedId::default();
        loop {
            let field = i_prot.read_field_begin()?;
            if field.field_type == TType::Stop {
                break;
            }
            match field.id {
                Some(2) => q.entity_id = i_prot.read_i64()?,
                _ => i_prot.skip(field.field_type)?,
            }
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        Ok(q)
    }

    fn write_to_out_protocol(&self, _o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        unimplemented!("decode-only")
    }
}

#[derive(Debug, Default, Clone)]
struct EntityWithMetadata {
    qualified_id: QualifiedId,
}

impl TSerializable for EntityWithMetadata {
    fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Self> {
        i_prot.read_struct_begin()?;
        let mut e = EntityWithMetadata::default();
        loop {
            let field = i_prot.read_field_begin()?;
            if field.field_type == TType::Stop {
                break;
            }
            match field.id {
                Some(1) => e.qualified_id = QualifiedId::read_from_in_protocol(i_prot)?,
                _ => i_prot.skip(field.field_type)?,
            }
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        Ok(e)
    }

    fn write_to_out_protocol(&self, _o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        unimplemented!("decode-only")
    }
}

#[derive(Debug, Default, Clone)]
struct UnifiedPostAnnotations {
    entities: Vec<EntityWithMetadata>,
}

impl TSerializable for UnifiedPostAnnotations {
    fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Self> {
        i_prot.read_struct_begin()?;
        let mut a = UnifiedPostAnnotations::default();
        loop {
            let field = i_prot.read_field_begin()?;
            if field.field_type == TType::Stop {
                break;
            }
            match field.id {
                Some(2) if field.field_type == TType::List => {
                    let list = i_prot.read_list_begin()?;
                    a.entities = Vec::with_capacity(list.size.max(0) as usize);
                    for _ in 0..list.size {
                        a.entities
                            .push(EntityWithMetadata::read_from_in_protocol(i_prot)?);
                    }
                    i_prot.read_list_end()?;
                }
                _ => i_prot.skip(field.field_type)?,
            }
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        Ok(a)
    }

    fn write_to_out_protocol(&self, _o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        unimplemented!("decode-only")
    }
}

#[derive(Debug, Default, Clone)]
struct UnifiedPostAnnotationsValue {
    topics: Vec<String>,
    annotations: UnifiedPostAnnotations,
}

impl TSerializable for UnifiedPostAnnotationsValue {
    fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Self> {
        i_prot.read_struct_begin()?;
        let mut tv = UnifiedPostAnnotationsValue::default();
        loop {
            let field = i_prot.read_field_begin()?;
            if field.field_type == TType::Stop {
                break;
            }
            match field.id {
                Some(FID_TOPICS) if field.field_type == TType::List => {
                    let list = i_prot.read_list_begin()?;
                    tv.topics = Vec::with_capacity(list.size.max(0) as usize);
                    for _ in 0..list.size {
                        tv.topics.push(i_prot.read_string()?);
                    }
                    i_prot.read_list_end()?;
                }
                Some(FID_ANNOTATIONS) if field.field_type == TType::Struct => {
                    tv.annotations = UnifiedPostAnnotations::read_from_in_protocol(i_prot)?;
                }
                _ => i_prot.skip(field.field_type)?,
            }
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        Ok(tv)
    }

    fn write_to_out_protocol(&self, _o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        unimplemented!("decode-only")
    }
}
