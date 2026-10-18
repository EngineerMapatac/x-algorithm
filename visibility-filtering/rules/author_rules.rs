use crate::models::AuthorLabel;
use crate::rules::rule_spec::leaves::*;
use crate::rules::rule_spec::rule::Audience::{Everyone, NonAuthors, NonFollowers};
use crate::rules::rule_spec::rule::{all, Rule};
use crate::rules::rule_spec::*;

pub(super) const SUSPENDED_AUTHOR_DROP: Rule = Rule {
    id: RuleId::SuspendedAuthor,
    audience: NonAuthors,
    when: all![AuthorIsSuspended],
    action: drop_post(DropAs::AuthorIsSuspended),
};

pub(super) const DEACTIVATED_AUTHOR_DROP: Rule = Rule {
    id: RuleId::DeactivatedAuthor,
    audience: NonAuthors,
    when: all![AuthorIsDeactivated],
    action: drop_post(DropAs::AuthorIsDeactivated),
};

pub(super) const ERASED_AUTHOR_DROP_INACTIVE: Rule = Rule {
    id: RuleId::ErasedAuthor,
    audience: NonAuthors,
    when: all![AuthorIsErased],
    action: drop_post(DropAs::AuthorAccountIsInactive),
};

pub(super) const OFFBOARDED_AUTHOR_DROP_INACTIVE: Rule = Rule {
    id: RuleId::OffboardedAuthor,
    audience: NonAuthors,
    when: all![AuthorIsOffboarded],
    action: drop_post(DropAs::AuthorAccountIsInactive),
};

pub(super) const PROTECTED_AUTHOR_DROP: Rule = Rule {
    id: RuleId::ProtectedAuthor,
    audience: NonFollowers,
    when: all![AuthorIsProtected],
    action: drop_post(DropAs::AuthorIsProtected),
};

pub(super) const VIEWER_BLOCKS_AUTHOR_DROP: Rule = Rule {
    id: RuleId::ViewerBlocksAuthor,
    audience: Everyone,
    when: all![!ViewerLoggedOut, ViewerBlocksAuthor],
    action: drop_post(DropAs::ViewerBlocksAuthor),
};

pub(super) const VIEWER_MUTES_AUTHOR_DROP: Rule = Rule {
    id: RuleId::ViewerMutesAuthor,
    audience: Everyone,
    when: all![!ViewerLoggedOut, ViewerMutesAuthor],
    action: drop_post(DropAs::ViewerMutesAuthor),
};

pub(super) const VIEWER_MUTES_RETWEETS_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::ViewerMutesRetweets,
    audience: Everyone,
    when: all![
        !ViewerLoggedOut,
        TweetIsRetweet,
        ViewerMutesRetweetsFromAuthor
    ],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const NSFW_USER_AUTHOR_DROP_NSFW_MEDIA: Rule = Rule {
    id: RuleId::NsfwUserAuthor,
    audience: NonAuthors,
    when: all![AuthorIsNsfwUser],
    action: drop_post(DropAs::ContainNsfwMedia),
};

pub(super) const NSFW_ADMIN_AUTHOR_DROP_NSFW_MEDIA: Rule = Rule {
    id: RuleId::NsfwAdminAuthor,
    audience: NonAuthors,
    when: all![AuthorIsNsfwAdmin],
    action: drop_post(DropAs::ContainNsfwMedia),
};

pub(super) const NSFW_HIGH_RECALL_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::NsfwHighRecallUserLabel,
    audience: NonAuthors,
    when: all![AuthorHasLabel(AuthorLabel::NsfwHighRecall)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const NSFW_HIGH_PRECISION_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::NsfwHighPrecisionUserLabel,
    audience: NonAuthors,
    when: all![AuthorHasLabel(AuthorLabel::NsfwHighPrecision)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const NSFW_AVATAR_IMAGE_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::NsfwAvatarImageUserLabel,
    audience: NonAuthors,
    when: all![AuthorHasLabel(AuthorLabel::NsfwAvatarImage)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const NSFW_BANNER_IMAGE_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::NsfwBannerImageUserLabel,
    audience: NonAuthors,
    when: all![AuthorHasLabel(AuthorLabel::NsfwBannerImage)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const NSFW_NEAR_PERFECT_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::NsfwNearPerfectUserLabel,
    audience: NonAuthors,
    when: all![AuthorHasLabel(AuthorLabel::NsfwNearPerfect)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const SPAM_HIGH_RECALL_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::SpamHighRecallUserLabel,
    audience: NonAuthors,
    when: all![AuthorHasLabel(AuthorLabel::SpamHighRecall)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const COMPROMISED_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::CompromisedUserLabel,
    audience: NonAuthors,
    when: all![AuthorHasLabel(AuthorLabel::Compromised)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const READ_ONLY_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::ReadOnlyUserLabel,
    audience: NonAuthors,
    when: all![AuthorHasLabel(AuthorLabel::ReadOnly)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const IMPERSONATION_HIGH_PRECISION_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::ImpersonationHighPrecisionUserLabel,
    audience: NonAuthors,
    when: all![AuthorHasLabel(AuthorLabel::ImpersonationHighPrecision)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const ABUSIVE_HIGH_RECALL_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::AbusiveHighRecallUserLabel,
    audience: NonFollowers,
    when: all![AuthorHasLabel(AuthorLabel::AbusiveHighRecall)],
    action: drop_post(DropAs::UnspecifiedReason),
};

pub(super) const DO_NOT_AMPLIFY_USER_LABEL_DROP_UNSPECIFIED: Rule = Rule {
    id: RuleId::DoNotAmplifyUserLabel,
    audience: NonFollowers,
    when: all![AuthorHasLabel(AuthorLabel::DoNotAmplify)],
    action: drop_post(DropAs::UnspecifiedReason),
};
