use serde::{Deserialize, Serialize};
use xai_twittercontext_proto::TwitterContextViewer;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Viewer {
    LoggedIn {
        id: u64,
        profile: ViewerProfile,
        has_age_verified_18_label: bool,
    },
    #[default]
    LoggedOut,
}

impl Viewer {
    pub fn user_id(&self) -> Option<u64> {
        match self {
            Viewer::LoggedIn { id, .. } => Some(*id),
            Viewer::LoggedOut => None,
        }
    }
}

pub const ADULT_AGE_YEARS: i32 = 18;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewerAge {
    Known(i32),
    NotStated,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViewerProfile {
    pub allows_sensitive_media: bool,
    pub viewer_age: ViewerAge,
    pub has_verified_badge: bool,
    pub has_idv_premium: bool,
    pub is_read_only: bool,
    pub account_country_code: Option<String>,
}

impl ViewerProfile {
    pub fn is_underage(&self) -> bool {
        matches!(self.viewer_age, ViewerAge::Known(age) if age < ADULT_AGE_YEARS)
    }

    pub fn has_no_stated_age(&self) -> bool {
        self.viewer_age == ViewerAge::NotStated
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, strum::VariantArray,
)]
pub enum VerifyBlurSupport {
    #[default]
    Unsupported,
    IosNeedsUpdate,
    AndroidNeedsUpdate,
    Supported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientCapability {
    pub verify_blur_support: Option<VerifyBlurSupport>,
    pub modern_blur: bool,
    pub stale_tweet_limits: bool,
    pub community_viewer_removed_limits: bool,
    pub gore_blur_ignores_settings: bool,
    pub fosnr_rules: bool,
    pub fosnr_fallback_drops: bool,
}

impl Default for ClientCapability {
    fn default() -> Self {
        Self {
            verify_blur_support: Some(VerifyBlurSupport::default()),
            modern_blur: false,
            stale_tweet_limits: true,
            community_viewer_removed_limits: false,
            gore_blur_ignores_settings: false,
            fosnr_rules: true,
            fosnr_fallback_drops: false,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
pub enum Client {
    #[default]
    Web,
    IosCurrent,
    IosOutdated,
    AndroidCurrent,
    AndroidOutdated,
    AndroidWithoutFosnr,
    MacApp,
}

impl Client {
    const fn app(self) -> (i64, Option<&'static str>) {
        match self {
            Self::Web => (3_033_300, None),
            Self::IosCurrent => (
                129_032,
                Some("Twitter-iPhone/11.11.5 iOS/17.0 (Apple;iPhone15,2;;;;;1;2022)"),
            ),
            Self::IosOutdated => (
                129_032,
                Some("Twitter-iPhone/11.11.4 iOS/17.0 (Apple;iPhone15,2;;;;;1;2022)"),
            ),
            Self::AndroidCurrent => (
                258_901,
                Some(
                    "TwitterAndroid/11.11.0-release.00 (311110000-r-0) Pixel 7/14 \
                     (Google;panther;google;panther;0;;1;2022)",
                ),
            ),
            Self::AndroidOutdated => (
                258_901,
                Some(
                    "TwitterAndroid/11.10.9-release.00 (311109000-r-0) Pixel 7/14 \
                     (Google;panther;google;panther;0;;1;2022)",
                ),
            ),
            Self::AndroidWithoutFosnr => (
                258_901,
                Some(
                    "TwitterAndroid/9.82.0-release.00 (29820000-r-0) Pixel 7/14 \
                     (Google;panther;google;panther;0;;1;2022)",
                ),
            ),
            Self::MacApp => (
                557_701,
                Some("Twitter-Mac/11.11.5 macOS/14.0 (Apple;Mac14,2)"),
            ),
        }
    }

    pub(crate) fn context(
        self,
        viewer_id: Option<u64>,
        country_code: Option<&str>,
    ) -> TwitterContextViewer {
        let (client_application_id, user_agent) = self.app();
        TwitterContextViewer {
            user_id: viewer_id.map_or(0, u64::cast_signed),
            client_application_id,
            user_agent: user_agent.unwrap_or_default().to_string(),
            request_country_code: country_code.unwrap_or_default().to_string(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ViewerFeatures {
    pub viewer: Viewer,
    pub country_code: Option<String>,
    pub client_capability: ClientCapability,
}

impl ViewerFeatures {
    pub fn from_request(
        viewer: Viewer,
        country_code: Option<String>,
        client_capability: ClientCapability,
    ) -> Self {
        Self {
            viewer,
            country_code: country_code.map(|c| c.to_ascii_lowercase()),
            client_capability,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stated_age_below_adult_is_underage_and_only_a_missing_age_is_unstated() {
        for (age, underage, no_stated_age) in [
            (ViewerAge::Known(ADULT_AGE_YEARS - 1), true, false),
            (ViewerAge::Known(ADULT_AGE_YEARS), false, false),
            (ViewerAge::NotStated, false, true),
            (ViewerAge::Unknown, false, false),
        ] {
            let profile = ViewerProfile {
                viewer_age: age,
                ..ViewerProfile::default()
            };
            assert_eq!(profile.is_underage(), underage, "{age:?}");
            assert_eq!(profile.has_no_stated_age(), no_stated_age, "{age:?}");
        }
    }
}
