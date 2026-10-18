use super::feedback::{feedback_url, FEEDBACK_TTL_MS};
use xai_urt_thrift::entry::{TimelineEntry, TimelineEntryContent};
use xai_urt_thrift::feedback::{FeedbackEngagementType, FeedbackEntity, FeedbackMetadata};
use xai_urt_thrift::item::{TimelineItem, TimelineItemContent};
use xai_urt_thrift::metadata::{Callback, ClientEventInfo};
use xai_urt_thrift::prompt::{Prompt, PromptContent};
use xai_urt_thrift::relevance_prompt::{RelevancePrompt, RelevancePromptDisplayType};

const ENTRY_ID: &str = "relevanceprompt-feed-survey-prompt";
const COMPONENT: &str = "for_you_relevance_prompt";
const ELEMENT: &str = "relevance_prompt";

struct SurveyStrings {
    title: &'static str,
    confirmation: &'static str,
    is_relevant: &'static str,
    neutral: &'static str,
    not_relevant: &'static str,
}

const ENGLISH: SurveyStrings = SurveyStrings {
    title: "Is your feed interesting?",
    confirmation: "Thank you for your feedback!",
    is_relevant: "Yes",
    neutral: "Not sure",
    not_relevant: "No",
};

const JAPANESE: SurveyStrings = SurveyStrings {
    title: "本日のタイムラインはいかがですか？",
    confirmation: "フィードバックありがとうございます",
    is_relevant: "良い",
    neutral: "普通",
    not_relevant: "悪い",
};

pub(super) fn marshal_feed_survey(
    sort_index: i64,
    language_code: &str,
    served_id: u64,
) -> TimelineEntry {
    let strings = survey_strings(language_code);

    let relevance = RelevancePrompt {
        title: strings.title.to_string(),
        confirmation: strings.confirmation.to_string(),
        is_relevant_text: strings.is_relevant.to_string(),
        not_relevant_text: strings.not_relevant.to_string(),
        is_relevant_callback: survey_callback("Relevant", served_id),
        not_relevant_callback: survey_callback("NotRelevant", served_id),
        reactive_triggers: None,
        display_type: Some(RelevancePromptDisplayType::COMPACT),
        is_relevant_follow_up: None,
        not_relevant_follow_up: None,
        subtitle: None,
        neutral_text: Some(strings.neutral.to_string()),
        neutral_callback: Some(survey_callback("Neutral", served_id)),
    };

    let prompt = Prompt {
        content: PromptContent::RelevancePrompt(relevance),
        client_event_info: Some(ClientEventInfo {
            component: Some(COMPONENT.to_string()),
            element: Some(ELEMENT.to_string()),
            details: None,
            action: None,
            entity_token: None,
        }),
        impression_callbacks: None,
    };

    TimelineEntry {
        entry_id: ENTRY_ID.to_string(),
        sort_index,
        content: TimelineEntryContent::Item(TimelineItem {
            content: TimelineItemContent::Prompt(prompt),
            client_event_info: None,
            feedback_info: None,
            prompt: None,
            reactive_triggers: None,
        }),
        expiry_time: None,
    }
}

fn survey_strings(language_code: &str) -> &'static SurveyStrings {
    let primary = language_code
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if primary == "ja" {
        &JAPANESE
    } else {
        &ENGLISH
    }
}

fn survey_callback(feedback_type: &str, served_id: u64) -> Callback {
    let metadata = FeedbackMetadata {
        injection_type: None,
        engagement_type: Some(FeedbackEngagementType::RELEVANCE_PROMPT),
        entity_ids: Some(vec![FeedbackEntity::FeedbackId(served_id as i64)]),
        ttl_ms: Some(FEEDBACK_TTL_MS),
    };
    Callback {
        endpoint: feedback_url(feedback_type, &metadata),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relevance_prompt(entry: &TimelineEntry) -> &RelevancePrompt {
        match &entry.content {
            TimelineEntryContent::Item(TimelineItem {
                content:
                    TimelineItemContent::Prompt(Prompt {
                        content: PromptContent::RelevancePrompt(p),
                        ..
                    }),
                ..
            }) => p,
            other => panic!("expected relevance prompt, got {other:?}"),
        }
    }

    #[test]
    fn marshals_english_prompt_by_default() {
        let entry = marshal_feed_survey(0, "en", 123);
        assert_eq!(entry.entry_id, ENTRY_ID);
        let p = relevance_prompt(&entry);
        assert_eq!(p.title, "Is your feed interesting?");
        assert_eq!(p.is_relevant_text, "Yes");
        assert_eq!(p.neutral_text.as_deref(), Some("Not sure"));
        assert_eq!(p.not_relevant_text, "No");
        assert_eq!(p.display_type, Some(RelevancePromptDisplayType::COMPACT));
    }

    #[test]
    fn marshals_japanese_prompt() {
        for lang in ["ja", "ja-JP"] {
            let entry = marshal_feed_survey(0, lang, 123);
            let p = relevance_prompt(&entry);
            assert_eq!(p.title, "本日のタイムラインはいかがですか？");
            assert_eq!(p.is_relevant_text, "良い");
            assert_eq!(p.neutral_text.as_deref(), Some("普通"));
            assert_eq!(p.not_relevant_text, "悪い");
        }
    }

    #[test]
    fn callbacks_carry_feedback_type_and_metadata() {
        let entry = marshal_feed_survey(0, "en", 123);
        let p = relevance_prompt(&entry);
        assert!(p
            .is_relevant_callback
            .endpoint
            .starts_with("/2/timeline/feedback.json?feedback_type=Relevant&action_metadata="));
        assert!(p
            .not_relevant_callback
            .endpoint
            .starts_with("/2/timeline/feedback.json?feedback_type=NotRelevant&action_metadata="));
        assert!(p
            .neutral_callback
            .as_ref()
            .unwrap()
            .endpoint
            .starts_with("/2/timeline/feedback.json?feedback_type=Neutral&action_metadata="));
    }
}
