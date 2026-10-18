use super::{Feature, Leaf, Meaning};
use crate::hydration::Hydrators;
use crate::models::VerifyBlurSupport;
use crate::params::CountryList;
use crate::rules::context::At;
use crate::rules::rule_spec::rule::RuleCond;
use crate::rules::rule_spec::Predicate;

pub(in crate::rules) enum Request {}

impl Feature for Request {
    const REQUIRED: Hydrators = Hydrators::empty();
    type Value<'a> = ();

    #[inline]
    fn read<'a>(_: At<'_, 'a>) {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum RequestLeaf {
    ViewerLoggedOut,
    RequestCountryIn(CountryList),
    RequestCountryIs(&'static str),
    ClientVerifyBlurSupportIs(VerifyBlurSupport),
    ClientHasModernBlur,
    ClientHasStaleTweetLimits,
    ClientHasCommunityViewerRemovedLimits,
    ClientBlursGoreIgnoringSettings,
    ClientHasFosnrRules,
    ClientNeedsFosnrFallbackDrops,
}

impl Leaf for RequestLeaf {
    type Feature = Request;

    #[inline]
    fn meaning<'a, M: Meaning<'a, Request>>(self, m: M) -> M::Out {
        match self {
            Self::ViewerLoggedOut => m.run(|(), facts| facts.viewer_id().is_none()),
            Self::RequestCountryIn(list) => m.run_with(list, |(), facts| {
                facts
                    .request_country()
                    .is_some_and(|country| facts.in_country_list(list, country))
            }),
            Self::RequestCountryIs(country) => m.run_with(country, |(), facts| {
                facts.request_country() == Some(country)
            }),
            Self::ClientVerifyBlurSupportIs(support) => m.run_with(support, |(), facts| {
                facts.client_capability().verify_blur_support == Some(support)
            }),
            Self::ClientHasModernBlur => m.run(|(), facts| facts.client_capability().modern_blur),
            Self::ClientHasStaleTweetLimits => {
                m.run(|(), facts| facts.client_capability().stale_tweet_limits)
            }
            Self::ClientHasCommunityViewerRemovedLimits => {
                m.run(|(), facts| facts.client_capability().community_viewer_removed_limits)
            }
            Self::ClientBlursGoreIgnoringSettings => {
                m.run(|(), facts| facts.client_capability().gore_blur_ignores_settings)
            }
            Self::ClientHasFosnrRules => m.run(|(), facts| facts.client_capability().fosnr_rules),
            Self::ClientNeedsFosnrFallbackDrops => {
                m.run(|(), facts| facts.client_capability().fosnr_fallback_drops)
            }
        }
    }
}

impl RequestLeaf {
    pub(in crate::rules) const fn leaf(self) -> RuleCond {
        RuleCond::Leaf(Predicate::Request(self))
    }
}
