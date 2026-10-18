use arrow::array::{Array, BooleanArray, FixedSizeListArray, Int64Array};
use arrow::ipc::reader::StreamReader;
use std::collections::HashSet;
use xai_recsys_proto::ActionName;

const COL_AUTHOR_ID: &str = "authorId";
const COL_ACTION_NAME_MULTI_HOT: &str = "actionNameMultiHot";

const POSITIVE_ENGAGEMENTS: [ActionName; 18] = [
    ActionName::ServerTweetFav,
    ActionName::ServerTweetReply,
    ActionName::ServerTweetQuote,
    ActionName::ServerTweetRetweet,
    ActionName::ClientTweetClick,
    ActionName::ClientQuotedTweetClick,
    ActionName::ClientTweetOpenLink,
    ActionName::ClientTweetClickProfile,
    ActionName::ClientTweetPhotoExpand,
    ActionName::ClientQuotedTweetPhotoExpand,
    ActionName::ClientTweetVideoOpen,
    ActionName::ClientTweetVideoQualityView,
    ActionName::ClientQuotedTweetVideoQualityView,
    ActionName::ClientTweetShare,
    ActionName::ClientTweetClickSendViaDirectMessage,
    ActionName::ClientTweetShareViaCopyLink,
    ActionName::ClientTweetBookmark,
    ActionName::ClientTweetFollowAuthor,
];

pub fn positively_engaged_author_ids(columnar_sequence: &bytes::Bytes) -> Option<HashSet<u64>> {
    let reader =
        StreamReader::try_new(std::io::Cursor::new(columnar_sequence.as_ref()), None).ok()?;
    let mut authors = HashSet::new();
    for batch in reader {
        let batch = batch.ok()?;
        let author_ids = batch
            .column_by_name(COL_AUTHOR_ID)?
            .as_any()
            .downcast_ref::<Int64Array>()?;
        let multi_hot = batch
            .column_by_name(COL_ACTION_NAME_MULTI_HOT)?
            .as_any()
            .downcast_ref::<FixedSizeListArray>()?;
        let hot_values = multi_hot.values().as_any().downcast_ref::<BooleanArray>()?;
        let vocab = multi_hot.value_length() as usize;
        for row in 0..batch.num_rows() {
            if author_ids.is_null(row) || author_ids.value(row) <= 0 {
                continue;
            }
            let base = row * vocab;
            let engaged = POSITIVE_ENGAGEMENTS.iter().any(|action| {
                let idx = *action as usize;
                idx < vocab && hot_values.value(base + idx)
            });
            if engaged {
                authors.insert(author_ids.value(row) as u64);
            }
        }
    }
    Some(authors)
}
