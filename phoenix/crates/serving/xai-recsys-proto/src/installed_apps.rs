// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
pub const NUM_INSTALLED_APPS: usize = 64;

pub const NUM_INSTALLED_APPS_128: usize = 128;

pub const MAX_DEVICES_PER_USER: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum InstalledApp {
    Unknown = 0,
    Whatsapp = 1,
    Snapchat = 2,
    Spotify = 3,
    Threads = 4,
    Tesla = 5,
    Twitch = 6,
    Youtube = 7,
    Tiktok = 8,
    Discord = 9,
    Slack = 10,
    Amazon = 11,
    ChatGpt = 12,
    GoogleMaps = 13,
    Grok = 14,
    GrokImagine = 15,
    Duolingo = 16,
    Facebook = 17,
    Messenger = 18,
    FacebookAuth = 19,
    Github = 20,
    Gemini = 21,
    Google = 22,
    GoogleCalendar = 23,
    GoogleDocs = 24,
    Chrome = 25,
    Gmail = 26,
    Hulu = 27,
    Instagram = 28,
    InstagramStories = 29,
    Line = 30,
    Linkedin = 31,
    Outlook = 32,
    Netflix = 33,
    Paypal = 34,
    Pinterest = 35,
    Telegram = 36,
    Wechat = 37,
    Robinhood = 38,
    Phantom = 39,
    Coinbase = 40,
    Binance = 41,
    Bybit = 42,
    DuckDuckGo = 43,
    Yelp = 44,
    Temu = 45,
    Shopify = 46,
    Wayfair = 47,
    Wealthsimple = 48,
    YahooMail = 49,
    Starlink = 50,
    Shop = 51,
    Reddit = 52,
    Schwab = 53,
    Etrade = 54,
    Fidelity = 55,
    Ibkr = 56,
    Shein = 57,
    Zillow = 58,
    Claude = 59,
    KakaoTalk = 60,
    PayPay = 61,
    Mercari = 62,
    Roblox = 63,
    McDonalds = 64,
    Starbucks = 65,
    Minecraft = 66,
    Kick = 67,
    Qq = 68,
    Viber = 69,
    Rakuten = 70,
    RakutenPay = 71,
    Redfin = 72,
    ISpeed = 73,
    JpMatsui = 74,
    JpMonex = 75,
    JpTransit = 76,
    Kabu = 77,
    Sbi = 78,
    HotPepperBeauty = 79,
    Tabelog = 80,
    Navitime = 81,
    YahooTransit = 82,
    Yj = 83,
}

impl InstalledApp {
    #[inline(always)]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "whatsapp" => Some(Self::Whatsapp),
            "snapchat" => Some(Self::Snapchat),
            "spotify" => Some(Self::Spotify),
            "barcelona" => Some(Self::Threads),
            "tesla" => Some(Self::Tesla),
            "twitch" => Some(Self::Twitch),
            "youtube" => Some(Self::Youtube),
            "tiktok" => Some(Self::Tiktok),
            "discord" => Some(Self::Discord),
            "slack" => Some(Self::Slack),
            "com.amazon.mobile.shopping" | "amazon" => Some(Self::Amazon),
            "chatgpt" => Some(Self::ChatGpt),
            "comgooglemaps" => Some(Self::GoogleMaps),
            "xai-grok" => Some(Self::Grok),
            "grok-imagine" => Some(Self::GrokImagine),
            "duolingo" => Some(Self::Duolingo),
            "fb" => Some(Self::Facebook),
            "fbauth" => Some(Self::FacebookAuth),
            "fb-messenger" => Some(Self::Messenger),
            "github" => Some(Self::Github),
            "gemini" => Some(Self::Gemini),
            "google" => Some(Self::Google),
            "googlecalendar" => Some(Self::GoogleCalendar),
            "googledocs" => Some(Self::GoogleDocs),
            "googlechrome" => Some(Self::Chrome),
            "googlegmail" => Some(Self::Gmail),
            "hulu" => Some(Self::Hulu),
            "instagram" => Some(Self::Instagram),
            "instagram-stories" => Some(Self::InstagramStories),
            "line" => Some(Self::Line),
            "linkedin" => Some(Self::Linkedin),
            "ms-outlook" => Some(Self::Outlook),
            "nflx" => Some(Self::Netflix),
            "paypal" => Some(Self::Paypal),
            "pinterest" => Some(Self::Pinterest),
            "tg" => Some(Self::Telegram),
            "weixin" => Some(Self::Wechat),
            "robinhood" => Some(Self::Robinhood),
            "phantom" => Some(Self::Phantom),
            "coinbase" => Some(Self::Coinbase),
            "bnc" => Some(Self::Binance),
            "bybitapp" => Some(Self::Bybit),
            "ddgQuickLink" => Some(Self::DuckDuckGo),
            "yelp" => Some(Self::Yelp),
            "temu" => Some(Self::Temu),
            "shopify" => Some(Self::Shopify),
            "wayfairapp" | "wayfair" => Some(Self::Wayfair),
            "wstrade" => Some(Self::Wealthsimple),
            "ymail" => Some(Self::YahooMail),
            "com.starlink.mobile" | "starlink" => Some(Self::Starlink),
            "shopapp" | "shop" => Some(Self::Shop),
            "reddit" => Some(Self::Reddit),
            "schwab" => Some(Self::Schwab),
            "etrade" => Some(Self::Etrade),
            "Fidelity" | "fidelity" => Some(Self::Fidelity),
            "ibkr" => Some(Self::Ibkr),
            "sheinlink" | "shein" => Some(Self::Shein),
            "zillow" => Some(Self::Zillow),
            "claude" => Some(Self::Claude),
            "kakaotalk" => Some(Self::KakaoTalk),
            "paypay" => Some(Self::PayPay),
            "mercari" => Some(Self::Mercari),
            "roblox" => Some(Self::Roblox),
            "mcdonalds" => Some(Self::McDonalds),
            "starbucks" => Some(Self::Starbucks),
            "minecraft" => Some(Self::Minecraft),
            "kick" => Some(Self::Kick),
            "qq" => Some(Self::Qq),
            "viber" => Some(Self::Viber),
            "rakuten" => Some(Self::Rakuten),
            "rakutenpay" => Some(Self::RakutenPay),
            "redfin" => Some(Self::Redfin),
            "ispeed" => Some(Self::ISpeed),
            "jpMatsui" => Some(Self::JpMatsui),
            "jpMonex" => Some(Self::JpMonex),
            "jpTransit" => Some(Self::JpTransit),
            "kabu" => Some(Self::Kabu),
            "sbi" => Some(Self::Sbi),
            "hotPepperBeauty" => Some(Self::HotPepperBeauty),
            "tabelog" => Some(Self::Tabelog),
            "navitime" => Some(Self::Navitime),
            "yahoo-transit" => Some(Self::YahooTransit),
            "yj" => Some(Self::Yj),
            "twitter" => None,
            _ => None,
        }
    }

    pub fn build_multi_hot(slugs: &[String], num_slots: usize) -> Vec<bool> {
        let len = num_slots.min(NUM_INSTALLED_APPS_128);
        let mut multi_hot = vec![false; len];
        for slug in slugs {
            if let Some(app) = Self::from_slug(slug) {
                let i = app.index();
                if i < len {
                    multi_hot[i] = true;
                }
            }
        }
        multi_hot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_at_index_64_ignored_in_64_bit_vector() {
        let slugs = vec!["mcdonalds".to_string()];
        let hot = InstalledApp::build_multi_hot(&slugs, NUM_INSTALLED_APPS);
        assert_eq!(hot.len(), NUM_INSTALLED_APPS);
        assert!(!hot.iter().any(|&b| b));
        let hot128 = InstalledApp::build_multi_hot(&slugs, NUM_INSTALLED_APPS_128);
        assert!(hot128[InstalledApp::McDonalds.index()]);
    }

    #[test]
    fn app_at_index_59_included_in_64_bit_vector() {
        let slugs = vec!["claude".to_string()];
        let hot = InstalledApp::build_multi_hot(&slugs, NUM_INSTALLED_APPS);
        assert!(hot[InstalledApp::Claude.index()]);
    }
}
