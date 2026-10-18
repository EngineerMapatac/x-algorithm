use std::fmt::Debug;
use xai_x_thrift::action::Action;
use xai_x_thrift::get_tweet_fields::TweetFieldsResultState;
use xai_x_thrift::safety_result::FilteredReason;

pub(crate) const NONE: &str = "none";

#[derive(Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Class {
    Allow,
    BareDrop,
    Drop,
    Suppressed,
    NotFound,
    Failed,
    Missing,
}

#[derive(Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Failure {
    NoValue,
    StratoError,
    RateLimited,
    DecodeError,
    RpcError,
    OverCapacity,
    TweetypieFailed,
}

pub(crate) type Label = (String, String);

pub(crate) fn label(class: Class, reason: impl Into<String>) -> Label {
    (<&str>::from(class).to_string(), reason.into())
}

pub(crate) fn failed(failure: Failure) -> Label {
    label(Class::Failed, <&str>::from(failure))
}

fn ident(s: &str) -> (&str, &str) {
    let end = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    s.split_at(end)
}

fn snake(ident: &str) -> String {
    let mut out = String::with_capacity(ident.len() + 4);
    for (i, c) in ident.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

pub(crate) fn debug_label(value: &impl Debug) -> Label {
    let debug = format!("{value:?}");
    let class = snake(ident(&debug).0);
    let Some((_, mut rest)) = debug.split_once("reason: Some(") else {
        return (class, NONE.to_string());
    };
    let reason = loop {
        let (name, after) = ident(rest);
        if let Some(inner) = after.strip_prefix(" {")
            && let Some((_, nested)) = inner.split_once("Some(")
        {
            rest = nested;
            continue;
        }
        let number = after
            .strip_prefix('(')
            .and_then(|r| r.split(')').next())
            .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit() || c == '-'));
        break match (name, number) {
            ("", _) => "unknown".to_string(),
            (name, Some(n)) => format!("{}_{n}", snake(name)),
            (name, None) => snake(name),
        };
    };
    (class, reason)
}

pub(crate) fn state_label(state: &TweetFieldsResultState) -> Label {
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "FilteredReason is a generated Thrift union; every variant but SafetyResult labels by its Debug name, including any the IDL adds"
    )]
    let reason = |reason: &FilteredReason| match reason {
        FilteredReason::SafetyResult(result) => debug_label(result).1,
        other => debug_label(other).0,
    };
    match state {
        TweetFieldsResultState::Found(found) => match &found.suppress_reason {
            None => label(Class::Allow, NONE),
            Some(suppress @ FilteredReason::SafetyResult(result)) => {
                (debug_label(&result.action).0, reason(suppress))
            }
            Some(suppress) => label(Class::Suppressed, reason(suppress)),
        },
        TweetFieldsResultState::Filtered(filtered) => {
            #[expect(
                clippy::wildcard_enum_match_arm,
                reason = "FilteredReason is a generated Thrift union; every variant but SafetyResult is a plain drop, including any the IDL adds"
            )]
            let drop_label = match &filtered.reason {
                FilteredReason::SafetyResult(result)
                    if result.reason.is_none()
                        && matches!(&result.action, Action::Drop(drop) if drop.reason.is_none()) =>
                {
                    label(Class::BareDrop, NONE)
                }
                filtered @ FilteredReason::SafetyResult(result) => {
                    (debug_label(&result.action).0, reason(filtered))
                }
                filtered => label(Class::Drop, reason(filtered)),
            };
            drop_label
        }
        TweetFieldsResultState::NotFound(not_found) => label(
            Class::NotFound,
            not_found
                .filtered_reason
                .as_ref()
                .map_or_else(|| NONE.to_string(), reason),
        ),
        TweetFieldsResultState::Failed(result) if result.over_capacity => {
            failed(Failure::OverCapacity)
        }
        TweetFieldsResultState::Failed(_) => failed(Failure::TweetypieFailed),
    }
}
