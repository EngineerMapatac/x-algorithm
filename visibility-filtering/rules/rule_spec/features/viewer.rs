use super::{Feature, Leaf, Meaning};
use crate::hydration::{Hydrator, Hydrators};
use crate::models::{Viewer, ViewerProfile};
use crate::params::CountryList;
use crate::rules::context::At;
use crate::rules::rule_spec::rule::RuleCond;
use crate::rules::rule_spec::{account_or_request_country_in, Predicate};

pub(in crate::rules) enum ViewerAccount {}

impl Feature for ViewerAccount {
    const REQUIRED: Hydrators = Hydrators::of(Hydrator::ViewerProfile);
    type Value<'a> = Option<&'a ViewerProfile>;

    #[inline]
    fn read<'a>(at: At<'_, 'a>) -> Option<&'a ViewerProfile> {
        match &at.viewer().viewer {
            Viewer::LoggedIn { profile, .. } => Some(profile),
            Viewer::LoggedOut => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ViewerAccountLeaf {
    ViewerUnderage,
    ViewerNoStatedAge,
    ViewerAllowsSensitiveMedia,
    ViewerAccountOrRequestCountryIn(CountryList),
    ViewerHasVerifiedBadge,
    ViewerReadOnly,
}

impl Leaf for ViewerAccountLeaf {
    type Feature = ViewerAccount;

    #[inline]
    fn meaning<'a, M: Meaning<'a, ViewerAccount>>(self, m: M) -> M::Out {
        match self {
            Self::ViewerUnderage => {
                m.run(|profile, _| profile.is_some_and(ViewerProfile::is_underage))
            }
            Self::ViewerNoStatedAge => {
                m.run(|profile, _| profile.is_some_and(ViewerProfile::has_no_stated_age))
            }
            Self::ViewerAllowsSensitiveMedia => {
                m.run(|profile, _| profile.is_some_and(|profile| profile.allows_sensitive_media))
            }
            Self::ViewerAccountOrRequestCountryIn(list) => m.run_with(list, |profile, facts| {
                account_or_request_country_in(profile, facts.request_country(), |country| {
                    facts.in_country_list(list, country)
                })
            }),
            Self::ViewerHasVerifiedBadge => {
                m.run(|profile, _| profile.is_some_and(|profile| profile.has_verified_badge))
            }
            Self::ViewerReadOnly => {
                m.run(|profile, _| profile.is_some_and(|profile| profile.is_read_only))
            }
        }
    }
}

impl ViewerAccountLeaf {
    pub(in crate::rules) const fn leaf(self) -> RuleCond {
        RuleCond::Leaf(Predicate::ViewerAccount(self))
    }
}

pub(in crate::rules) enum ViewerLabels {}

impl Feature for ViewerLabels {
    const REQUIRED: Hydrators = Hydrators::of(Hydrator::ViewerLabels);
    type Value<'a> = bool;

    #[inline]
    fn read<'a>(at: At<'_, 'a>) -> bool {
        match &at.viewer().viewer {
            Viewer::LoggedIn {
                has_age_verified_18_label,
                ..
            } => *has_age_verified_18_label,
            Viewer::LoggedOut => false,
        }
    }
}

pub(in crate::rules) enum ViewerCountry {}

impl Feature for ViewerCountry {
    const REQUIRED: Hydrators = Hydrators::of(Hydrator::ViewerCountry);
    type Value<'a> = Option<&'a str>;

    #[inline]
    fn read<'a>(at: At<'_, 'a>) -> Option<&'a str> {
        let features = at.post().conversation_control.as_ref();
        features.and_then(|features| features.viewer_country.as_deref())
    }
}
