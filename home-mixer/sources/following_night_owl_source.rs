use crate::clients::night_owl_client::{NightOwlClient, REQUEST_TIMEOUT_MS};
use crate::models::candidate::{PostCandidate, RetrievalSource};
use crate::models::query::ScoredPostsQuery;
use crate::params::FOLLOWING_POST_FETCH_SIZE;
use night_owl_proto as night_owl;
use night_owl_proto::pagination::Kind as PaginationKind;
use night_owl_proto::{FreshSince, FromSize, Pagination, SearchAfter};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use tonic::async_trait;
use xai_candidate_pipeline::component_library::utils::time_from_id;
use xai_candidate_pipeline::source::Source;
use xai_home_mixer_proto::ServedType;
use xai_urt_thrift::operation::CursorType;

const BASE_FILTERS: &str = "filter:follows include:retweets include:protected include:spam";
const MAX_RESULTS: u32 = FOLLOWING_POST_FETCH_SIZE as u32;
const COLLECTOR_TIMEOUT_MS: u32 = 300;
const RECALL_MAX_AGE_SECS: i64 = 365 * 24 * 60 * 60;

pub struct FollowingNightOwlSource {
    pub client: Arc<dyn NightOwlClient>,
}

#[async_trait]
impl Source<ScoredPostsQuery, PostCandidate> for FollowingNightOwlSource {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        !query.user_features.followed_user_ids.is_empty()
    }

    async fn source(&self, query: &ScoredPostsQuery) -> Result<Vec<PostCandidate>, String> {
        let request = build_request(query)?;

        let response = self
            .client
            .search(request, REQUEST_TIMEOUT_MS)
            .await
            .map_err(|e| format!("FollowingNightOwlSource: {e}"))?;

        let mut posts: Vec<PostCandidate> = response
            .hits
            .into_iter()
            .map(hit_to_post_candidate)
            .collect();

        posts.sort_by_key(|b| std::cmp::Reverse(b.tweet_id));

        Ok(posts)
    }
}

fn build_request(query: &ScoredPostsQuery) -> Result<night_owl::SearchRequest, String> {
    let mut from_user_ids: Vec<i64> =
        Vec::with_capacity(query.user_features.followed_user_ids.len() + 1);
    from_user_ids.push(query.user_id as i64);
    from_user_ids.extend(query.user_features.followed_user_ids.iter().copied());

    let (old_query, pagination) = pagination_for_cursor(query)?;

    Ok(night_owl::SearchRequest {
        old_query,
        target: night_owl::SearchTarget::Realtime as i32,
        pagination: Some(pagination),
        ranking: Some(night_owl::Ranking {
            kind: Some(night_owl::ranking::Kind::Recency(
                night_owl::RecencyRanking {
                    field: String::new(),
                },
            )),
        }),
        collector: Some(night_owl::Collector {
            timeout_ms: COLLECTOR_TIMEOUT_MS,
            track_total_hits: false,
            explain: false,
            profile: false,
            ..Default::default()
        }),
        return_full_source: true,
        include_debug: false,
        enable_archive: true,
        from_user_ids: from_user_ids.clone(),
        reply_to_user_ids: from_user_ids,
        ..Default::default()
    })
}

fn base_query(request_time_ms: i64) -> String {
    let now_secs = request_time_ms / 1000;
    if now_secs <= 0 {
        return BASE_FILTERS.to_string();
    }
    format!(
        "{BASE_FILTERS} since_time:{}",
        (now_secs - RECALL_MAX_AGE_SECS).max(0)
    )
}

fn pagination_for_cursor(query: &ScoredPostsQuery) -> Result<(String, Pagination), String> {
    let base = base_query(query.request_time_ms);

    let Some(cursor) = query.cursor.as_ref() else {
        return Ok((base, from_size_pagination()));
    };

    let cursor_type = cursor.cursor_type.as_ref();
    let id = cursor.id.filter(|&i| i > 0).map(|i| i as u64);

    match (cursor_type, id) {
        (Some(ct), Some(since_id)) if *ct == CursorType::TOP => {
            Ok((base, fresh_since_pagination(since_id)))
        }
        (Some(ct), Some(max_id)) if *ct == CursorType::BOTTOM => {
            Ok((base, search_after_pagination(max_id)))
        }
        (Some(ct), Some(max_id)) if *ct == CursorType::GAP => {
            let Some(since_id) = cursor.gap_boundary_id.filter(|&i| i > 0).map(|i| i as u64) else {
                return Err(format!(
                    "FollowingNightOwlSource: invalid gap cursor (missing gap_boundary_id): {cursor:?}"
                ));
            };
            let old_query = format!("{base} max_id:{}", max_id.saturating_sub(1));
            Ok((old_query, fresh_since_pagination(since_id)))
        }
        (Some(ct), None) if *ct == CursorType::GAP => Err(format!(
            "FollowingNightOwlSource: invalid gap cursor (missing id): {cursor:?}"
        )),
        _ => Ok((base, from_size_pagination())),
    }
}

fn from_size_pagination() -> Pagination {
    Pagination {
        kind: Some(PaginationKind::FromSize(FromSize {
            from: 0,
            size: MAX_RESULTS,
        })),
    }
}

fn fresh_since_pagination(since_id: u64) -> Pagination {
    Pagination {
        kind: Some(PaginationKind::FreshSince(FreshSince {
            size: MAX_RESULTS,
            cursor: pagination_cursor_bytes(since_id),
        })),
    }
}

fn search_after_pagination(max_id: u64) -> Pagination {
    Pagination {
        kind: Some(PaginationKind::SearchAfter(SearchAfter {
            size: MAX_RESULTS,
            cursor: pagination_cursor_bytes(max_id),
            ..Default::default()
        })),
    }
}

fn pagination_cursor_bytes(tweet_id: u64) -> bytes::Bytes {
    let created_at_ms = time_from_id(tweet_id)
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64;
    bytes::Bytes::from(format!(r#"[{created_at_ms},"{tweet_id}"]"#))
}

fn hit_to_post_candidate(hit: night_owl::SearchHit) -> PostCandidate {
    let doc = hit.doc.as_ref();
    let author_id = doc
        .and_then(|d| d.author_id)
        .and_then(|id| u64::try_from(id).ok())
        .unwrap_or(0);
    let in_reply_to_tweet_id = doc
        .and_then(|d| d.in_reply_to_status_id)
        .and_then(|id| u64::try_from(id).ok())
        .unwrap_or(0);
    let retweeted_tweet_id = doc
        .and_then(|d| d.retweet_of_status_id)
        .and_then(|id| u64::try_from(id).ok())
        .unwrap_or(0);
    let retweeted_user_id = doc
        .and_then(|d| d.retweeted_user_id)
        .and_then(|id| u64::try_from(id).ok())
        .unwrap_or(0);
    let quoted_tweet_id = doc
        .and_then(|d| d.quoted_tweet_id)
        .and_then(|id| u64::try_from(id).ok())
        .unwrap_or(0);
    let quoted_user_id = doc
        .and_then(|d| d.quoted_user_id)
        .and_then(|id| u64::try_from(id).ok())
        .unwrap_or(0);
    let conversation_id = doc
        .and_then(|d| d.conversation_id)
        .and_then(|id| u64::try_from(id).ok());

    let mut ancestors = Vec::new();
    if in_reply_to_tweet_id != 0 {
        ancestors.push(in_reply_to_tweet_id);
        if let Some(root) = conversation_id.filter(|&root| root != in_reply_to_tweet_id) {
            ancestors.push(root);
        }
    }

    PostCandidate {
        tweet_id: hit.doc_id,
        author_id,
        retweeted_tweet_id: (retweeted_tweet_id != 0).then_some(retweeted_tweet_id),
        retweeted_user_id: (retweeted_user_id != 0).then_some(retweeted_user_id),
        in_reply_to_tweet_id: (in_reply_to_tweet_id != 0).then_some(in_reply_to_tweet_id),
        quoted_tweet_id: (quoted_tweet_id != 0).then_some(quoted_tweet_id),
        quoted_user_id: (quoted_user_id != 0).then_some(quoted_user_id),
        score: Some(0.0),
        in_network: Some(true),
        ancestors,
        tweet_text: doc
            .map(|d| d.text.clone().unwrap_or_default())
            .unwrap_or_default(),
        served_type: Some(ServedType::FollowingInNetwork),
        retrieval_sources: vec![RetrievalSource::from_served_type(
            ServedType::FollowingInNetwork,
        )],
        ..Default::default()
    }
}
