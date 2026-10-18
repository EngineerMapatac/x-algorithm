use super::leaves::*;
use super::rule::{all, any};
use super::*;

pub(in crate::rules) const HAS_NSFW_MEDIA_LABEL: RuleCond = RuleCond::Named(&NamedCond {
    name: "has_nsfw_media_label",
    condition: any![
        TweetHasLabel(SafetyLabelType::NSFW_HIGH_PRECISION),
        TweetHasLabel(SafetyLabelType::NSFW_HIGH_RECALL),
        TweetHasLabel(SafetyLabelType::GORE_AND_VIOLENCE_HIGH_PRECISION)
    ],
});

pub(in crate::rules) const IS_NSFW_FLAGGED: RuleCond = RuleCond::Named(&NamedCond {
    name: "is_nsfw_flagged",
    condition: any![
        AuthorIsNsfwUser,
        AuthorIsNsfwAdmin,
        TweetNsfwUserFlag,
        TweetNsfwAdminFlag
    ],
});

pub(in crate::rules) const HAS_NSFW_TEXT_OR_CARD_LABEL: RuleCond = RuleCond::Named(&NamedCond {
    name: "has_nsfw_text_or_card_label",
    condition: any![
        TweetHasLabel(SafetyLabelType::NSFW_TEXT),
        TweetHasLabel(SafetyLabelType::NSFW_CARD_IMAGE)
    ],
});

pub(in crate::rules) const IS_NSFW_ADMIN: RuleCond = RuleCond::Named(&NamedCond {
    name: "is_nsfw_admin",
    condition: any![AuthorIsNsfwAdmin, TweetNsfwAdminFlag],
});

pub(in crate::rules) const IS_NSFW_USER: RuleCond = RuleCond::Named(&NamedCond {
    name: "is_nsfw_user",
    condition: any![AuthorIsNsfwUser, TweetNsfwUserFlag],
});

pub(in crate::rules) const TWEET_IS_NSFW_FLAGGED: RuleCond = RuleCond::Named(&NamedCond {
    name: "tweet_is_nsfw_flagged",
    condition: any![TweetNsfwAdminFlag, TweetNsfwUserFlag],
});

pub(in crate::rules) const IS_NSFW_SENSITIVE_TWEET: RuleCond = RuleCond::Named(&NamedCond {
    name: "is_nsfw_sensitive_tweet",
    condition: any![
        TweetHasLabel(SafetyLabelType::NSFW_HIGH_PRECISION),
        TweetHasLabel(SafetyLabelType::NSFW_HIGH_RECALL),
        TweetHasLabel(SafetyLabelType::NSFW_TEXT),
        TweetHasLabel(SafetyLabelType::NSFW_TEXT_HIGH_PRECISION),
        TweetHasLabel(SafetyLabelType::NSFW_VIDEO),
        TweetNsfwAdminFlag,
        TweetNsfwUserFlag
    ],
});

pub(in crate::rules) const IS_NSFW_SENSITIVE_AUTHOR: RuleCond = RuleCond::Named(&NamedCond {
    name: "is_nsfw_sensitive_author",
    condition: any![
        AuthorHasLabel(AuthorLabel::NsfwAvatarImage),
        AuthorHasLabel(AuthorLabel::NsfwBannerImage),
        AuthorHasLabel(AuthorLabel::NsfwHighPrecision),
        AuthorHasLabel(AuthorLabel::NsfwHighRecall),
        AuthorHasLabel(AuthorLabel::NsfwNearPerfect),
        AuthorIsNsfwAdmin,
        AuthorIsNsfwUser
    ],
});

pub(in crate::rules) const HAS_FOSNR_LABEL: RuleCond = RuleCond::Named(&NamedCond {
    name: "has_fosnr_label",
    condition: any![
        TweetHasLabel(SafetyLabelType::FOSNR_HATEFUL_CONDUCT),
        TweetHasLabel(SafetyLabelType::FOSNR_VIOLENT_SPEECH),
        TweetHasLabel(SafetyLabelType::FOSNR_ABUSE),
        TweetHasLabel(SafetyLabelType::FOSNR_CIVIC_INTEGRITY),
        TweetHasLabel(SafetyLabelType::FOSNR_ABUSE_INSULTS)
    ],
});

pub(in crate::rules) const VIEWER_IS_NOT_AGE_VERIFIED_IN_AGE_VERIFICATION_COUNTRIES: RuleCond =
    RuleCond::Named(&NamedCond {
        name: "viewer_is_not_age_verified_in_age_verification_countries",
        condition: all![
            RequestCountryIn(CountryList::AgeVerification),
            !ViewerAgeVerified
        ],
    });

pub(in crate::rules) const VIEWER_IS_NOT_AGE_VERIFIED_IN_TOMBSTONE_COUNTRIES: RuleCond =
    RuleCond::Named(&NamedCond {
        name: "viewer_is_not_age_verified_in_tombstone_countries",
        condition: all![RequestCountryIn(CountryList::Tombstone), !ViewerAgeVerified],
    });

pub(in crate::rules) const VIEWER_IS_NEITHER_ROOT_AUTHOR_NOR_INVITED: RuleCond =
    RuleCond::Named(&NamedCond {
        name: "viewer_is_neither_root_author_nor_invited",
        condition: RuleCond::Not(&any![
            ViewerIsConversationRootAuthor,
            ViewerIsInvitedToConversation
        ]),
    });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::SafetyLevel;
    use strum::VariantArray;

    #[test]
    fn no_two_named_conditions_read_the_same_leaves() {
        let mut named: Vec<&'static NamedCond> = Vec::new();
        for &level in SafetyLevel::VARIANTS {
            for rule in level.spec().rules {
                for root in [rule.audience.condition(), rule.when] {
                    root.named(&mut named);
                }
            }
        }
        let leaf_set = |definition: &NamedCond| {
            let mut leaves = Vec::new();
            definition.condition.leaves(&mut leaves);
            let mut leaves: Vec<String> = leaves.iter().map(|leaf| format!("{leaf:?}")).collect();
            leaves.sort();
            leaves
        };
        for (index, definition) in named.iter().enumerate() {
            for other in &named[..index] {
                if other.name == definition.name {
                    continue;
                }
                assert_ne!(
                    leaf_set(definition),
                    leaf_set(other),
                    "{} and {} read the same leaves; keep one",
                    other.name,
                    definition.name
                );
            }
        }
    }
}
