use crate::metrics::Rpc;
use crate::rules::levels;
use crate::rules::rule_spec::rule::Rule;
use strum::VariantArray;
use xai_visibility_filtering_proto as vf_pb;
use xai_x_thrift::safety_level::SafetyLevel as ThriftLevel;

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum SafetyLevel {
    FilterAll,
    TimelineHome,
    TimelineHomeRecommendations,
    TimelineHomeHydration,
    ImmersiveExpandedRecommendations,
}

pub(crate) struct LevelSpec {
    proto: Option<vf_pb::SafetyLevel>,
    thrift: ThriftLevel,
    on_evaluate_tweets: bool,
    pub(super) rules: &'static [&'static Rule],
    pub(crate) unspecified_drop_is_exact: bool,
}

impl SafetyLevel {
    pub(crate) const fn spec(self) -> LevelSpec {
        match self {
            SafetyLevel::FilterAll => LevelSpec {
                proto: Some(vf_pb::SafetyLevel::FilterAll),
                thrift: ThriftLevel::FILTER_ALL,
                on_evaluate_tweets: true,
                rules: levels::FILTER_ALL,
                unspecified_drop_is_exact: true,
            },
            SafetyLevel::TimelineHome => LevelSpec {
                proto: Some(vf_pb::SafetyLevel::TimelineHome),
                thrift: ThriftLevel::TIMELINE_HOME,
                on_evaluate_tweets: true,
                rules: levels::TIMELINE_HOME,
                unspecified_drop_is_exact: false,
            },
            SafetyLevel::TimelineHomeRecommendations => LevelSpec {
                proto: Some(vf_pb::SafetyLevel::TimelineHomeRecommendations),
                thrift: ThriftLevel::TIMELINE_HOME_RECOMMENDATIONS,
                on_evaluate_tweets: true,
                rules: levels::TIMELINE_HOME_RECOMMENDATIONS,
                unspecified_drop_is_exact: false,
            },
            SafetyLevel::TimelineHomeHydration => LevelSpec {
                proto: None,
                thrift: ThriftLevel::TIMELINE_HOME_HYDRATION,
                on_evaluate_tweets: true,
                rules: levels::TIMELINE_HOME_HYDRATION,
                unspecified_drop_is_exact: false,
            },
            SafetyLevel::ImmersiveExpandedRecommendations => LevelSpec {
                proto: Some(vf_pb::SafetyLevel::ImmersiveExpandedRecommendations),
                thrift: ThriftLevel::IMMERSIVE_EXPANDED_RECOMMENDATIONS,
                on_evaluate_tweets: false,
                rules: levels::IMMERSIVE_EXPANDED_RECOMMENDATIONS,
                unspecified_drop_is_exact: false,
            },
        }
    }

    pub(crate) fn from_proto(level: vf_pb::SafetyLevel) -> Option<Self> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|variant| variant.spec().proto == Some(level))
    }

    pub(crate) fn from_thrift(level: ThriftLevel) -> Option<Self> {
        Self::VARIANTS.iter().copied().find(|variant| {
            let spec = variant.spec();
            spec.on_evaluate_tweets && spec.thrift == level
        })
    }

    pub fn thrift_number(self) -> i32 {
        self.spec().thrift.0
    }

    pub(crate) fn is_carried_by(self, rpc: Rpc) -> bool {
        match rpc {
            Rpc::FilterTweets => self.spec().proto.is_some(),
            Rpc::EvaluateTweets => self.spec().on_evaluate_tweets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_filter_tweets_level_reaches_its_level() {
        let reached: Vec<_> = (0..=255)
            .filter_map(|number| vf_pb::SafetyLevel::try_from(number).ok())
            .map(|proto| (proto, SafetyLevel::from_proto(proto)))
            .collect();
        assert_eq!(
            reached,
            [
                (vf_pb::SafetyLevel::FilterAll, Some(SafetyLevel::FilterAll)),
                (
                    vf_pb::SafetyLevel::TimelineHome,
                    Some(SafetyLevel::TimelineHome)
                ),
                (
                    vf_pb::SafetyLevel::TimelineHomeRecommendations,
                    Some(SafetyLevel::TimelineHomeRecommendations)
                ),
                (
                    vf_pb::SafetyLevel::ImmersiveExpandedRecommendations,
                    Some(SafetyLevel::ImmersiveExpandedRecommendations)
                ),
            ]
        );
    }

    #[test]
    fn evaluate_tweets_serves_these_thrift_levels() {
        let served: Vec<_> = ThriftLevel::ENUM_VALUES
            .iter()
            .filter_map(|&thrift| SafetyLevel::from_thrift(thrift).map(|level| (thrift, level)))
            .collect();
        assert_eq!(
            served,
            [
                (ThriftLevel::TIMELINE_HOME, SafetyLevel::TimelineHome),
                (ThriftLevel::FILTER_ALL, SafetyLevel::FilterAll),
                (
                    ThriftLevel::TIMELINE_HOME_RECOMMENDATIONS,
                    SafetyLevel::TimelineHomeRecommendations
                ),
                (
                    ThriftLevel::TIMELINE_HOME_HYDRATION,
                    SafetyLevel::TimelineHomeHydration
                ),
            ]
        );
    }

    #[test]
    fn every_level_reports_the_number_the_thrift_enum_gives_it() {
        let numbers: Vec<_> = SafetyLevel::VARIANTS
            .iter()
            .map(|level| level.thrift_number())
            .collect();
        assert_eq!(numbers, [16, 8, 61, 82, 247]);
    }

    #[test]
    fn each_level_has_rules_an_ingress_and_wire_ids_of_its_own() {
        for &level in SafetyLevel::VARIANTS {
            let spec = level.spec();
            assert!(!spec.rules.is_empty(), "{level:?} wires no rule");
            assert!(
                spec.proto.is_some() || spec.on_evaluate_tweets,
                "{level:?} has no ingress"
            );
            for &other in SafetyLevel::VARIANTS
                .iter()
                .filter(|&&other| other != level)
            {
                let other_spec = other.spec();
                assert!(
                    spec.proto.is_none() || spec.proto != other_spec.proto,
                    "{level:?} and {other:?} share a proto level"
                );
                assert!(
                    spec.thrift != other_spec.thrift,
                    "{level:?} and {other:?} share a Thrift level"
                );
            }
        }
    }
}
