use std::collections::{HashMap, HashSet};
use xai_home_mixer_proto::feed_item::Item;
use xai_home_mixer_proto::FeedItem;
use xai_home_mixer_proto::ScoredPost;

pub fn select_conversation_modules(items: Vec<FeedItem>) -> (Vec<FeedItem>, Vec<FeedItem>) {
    let mut focal_by_root: HashMap<u64, u64> = HashMap::new();
    let mut reply_ancestors: HashMap<u64, Vec<u64>> = HashMap::new();

    for item in &items {
        let Some(Item::Post(post)) = &item.item else {
            continue;
        };
        if !is_reply(post) {
            continue;
        }
        let root = post.ancestors.iter().copied().min().unwrap();
        focal_by_root
            .entry(root)
            .and_modify(|focal| *focal = (*focal).max(post.tweet_id))
            .or_insert(post.tweet_id);
        reply_ancestors.insert(post.tweet_id, post.ancestors.clone());
    }

    let focals: HashSet<u64> = focal_by_root.values().copied().collect();
    let mut drop_ids: HashSet<u64> = HashSet::new();
    for item in &items {
        let Some(Item::Post(post)) = &item.item else {
            continue;
        };
        if is_reply(post) && !focals.contains(&post.tweet_id) {
            drop_ids.insert(post.tweet_id);
        }
    }
    for &focal_id in &focals {
        if let Some(ancestors) = reply_ancestors.get(&focal_id) {
            drop_ids.extend(ancestors.iter().copied().filter(|id| !focals.contains(id)));
        }
    }

    let module_tweet_ids: HashSet<u64> = drop_ids.union(&focals).copied().collect();
    for item in &items {
        let Some(Item::Post(post)) = &item.item else {
            continue;
        };
        if post.retweeted_tweet_id != 0 && module_tweet_ids.contains(&post.retweeted_tweet_id) {
            drop_ids.insert(post.tweet_id);
        }
    }

    items.into_iter().partition(|item| {
        as_post(item).is_none_or(|post| {
            focals.contains(&post.tweet_id) || !drop_ids.contains(&post.tweet_id)
        })
    })
}

fn is_reply(post: &ScoredPost) -> bool {
    post.in_reply_to_tweet_id != 0 && !post.ancestors.is_empty()
}

fn as_post(item: &FeedItem) -> Option<&ScoredPost> {
    match &item.item {
        Some(Item::Post(post)) => Some(post),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(tweet_id: u64, in_reply_to: u64, ancestors: &[u64]) -> FeedItem {
        FeedItem {
            position: 0,
            item: Some(Item::Post(ScoredPost {
                tweet_id,
                in_reply_to_tweet_id: in_reply_to,
                ancestors: ancestors.to_vec(),
                ..Default::default()
            })),
        }
    }

    fn retweet(tweet_id: u64, source: u64) -> FeedItem {
        FeedItem {
            position: 0,
            item: Some(Item::Post(ScoredPost {
                tweet_id,
                retweeted_tweet_id: source,
                ..Default::default()
            })),
        }
    }

    fn ids(items: &[FeedItem]) -> Vec<u64> {
        items
            .iter()
            .filter_map(|item| as_post(item).map(|p| p.tweet_id))
            .collect()
    }

    #[test]
    fn drops_retweet_of_absorbed_root() {
        let items = vec![retweet(50, 10), post(30, 10, &[10]), post(10, 0, &[])];
        let (kept, removed) = select_conversation_modules(items);
        assert_eq!(ids(&kept), vec![30]);
        assert_eq!(ids(&removed), vec![50, 10]);
    }

    #[test]
    fn keeps_unrelated_retweet() {
        let items = vec![retweet(50, 99), post(30, 10, &[10])];
        let (kept, _) = select_conversation_modules(items);
        assert_eq!(ids(&kept), vec![50, 30]);
    }

    #[test]
    fn does_not_absorb_focal_with_shallow_ancestors() {
        let items = vec![post(30, 20, &[20]), post(20, 10, &[10]), post(10, 0, &[])];
        let (kept, removed) = select_conversation_modules(items);
        assert_eq!(ids(&kept), vec![30, 20]);
        assert_eq!(ids(&removed), vec![10]);
    }
}
