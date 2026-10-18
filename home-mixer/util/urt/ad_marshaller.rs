use super::client_event::ad_client_event_info;
use crate::models::query::RequestType;

const ENTRY_NAMESPACE_PROMOTED_TWEET: &str = "promoted-tweet";
use super::post_marshaller::{make_tweet, make_tweet_item};
use std::collections::{BTreeMap, BTreeSet};
use xai_recsys_proto::{AdIndexInfo, HeaderImagePromptAdInfo};
use xai_urt_thrift::ad::RtbImageAd;
use xai_urt_thrift::api_media::{ApiMediaKey, TweetImageKey};
use xai_urt_thrift::contextual_refs::{ContextualTweetRef, SafetyLevel, TweetHydrationContext};
use xai_urt_thrift::entry::{TimelineEntry, TimelineEntryContent};
use xai_urt_thrift::item::{TimelineItem, TimelineItemContent};
use xai_urt_thrift::message::{
    HeaderImagePrompt, MessageAction, MessageContent, MessageImage, MessagePrompt,
    MessageTextAction,
};
use xai_urt_thrift::metadata::{Callback, ClientEventInfo, ImageVariant, Url, UrlType};
use xai_urt_thrift::promoted::{
    AdMetadataContainer, BoostCta, BoostCtaActionText, CallToAction, ClickTrackingInfo, DspId,
    DynamicPrerollType, MediaInfo, Preroll, PrerollMetadata, PromotedMetadata, RTBAdMetadata,
    SkAdNetworkData, UrlOverrideType, VideoVariant,
};
use xai_urt_thrift::richtext::RichText;

const CLIENT_TWEET_OPEN_LINK: usize = 39;
const LIKELY_TO_OPEN_LINK_THRESHOLD: f64 = 0.1;

const ENTRY_NAMESPACE_RTB_IMAGE_AD: &str = "rtb-image-ad";
const ENTRY_NAMESPACE_HEADER_IMAGE_PROMPT_AD: &str = "header-image-prompt-ad";
const GOOGLE_SDK_NATIVE_AD: &str = "GoogleSDKNativeAd";
const EMPTY_LANDING_URL: &str = "emptyLandingUrl";
const DSP_IMPRESSION_PREAMBLE: &str = "IS1:";
const DEFAULT_DSP_CREATIVE_ID: &str = "default-dsp-creative-id";

pub(super) fn marshal_ad(
    ad: &AdIndexInfo,
    sort_index: i64,
    request_type: RequestType,
) -> TimelineEntry {
    let impression_string = format!("{:x}", ad.impression_id);

    if let Some(info) = &ad.header_image_prompt_ad_info {
        return marshal_header_image_prompt_ad(
            ad,
            info,
            sort_index,
            request_type,
            &impression_string,
        );
    }

    if ad.post_id == 0 && ad.rtb_ad_metadata.is_some() {
        return marshal_ssp_ad(ad, sort_index, &impression_string);
    }

    let entry_id = format!(
        "{}-{}-{}",
        ENTRY_NAMESPACE_PROMOTED_TWEET, ad.post_id, impression_string
    );

    let url_params: BTreeMap<String, String> = ad
        .url_params
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let url_override = if ad.click_tracking_url_override.is_empty() {
        None
    } else {
        Some(ad.click_tracking_url_override.clone())
    };
    let url_override_type = match ad.click_tracking_url_override_type {
        1 => Some(UrlOverrideType::DCM),
        _ => None,
    };

    let has_preroll = ad.preroll_info.is_some();
    let is_sponsorship = ad.preroll_info.as_ref().is_some_and(|pi| pi.is_sponsorship);

    let click_tracking_info = ClickTrackingInfo {
        url_params: if url_params.is_empty() {
            None
        } else {
            Some(url_params)
        },
        url_override,
        url_override_type,
    };

    let ad_metadata_container = AdMetadataContainer {
        remove_promoted_attribution_for_preroll: if has_preroll {
            Some(!is_sponsorship)
        } else {
            None
        },
        video_analytics_scribe_passthrough: None,
        preroll: None,
        sponsorship_candidate: None,
        sponsorship_organization: None,
        sponsorship_organization_website: None,
        sponsorship_type: None,
        disclaimer_type: None,
        sk_ad_network_data_list: convert_skan_list(&ad.sk_ad_network_data),
        sk_ad_network_data: convert_skan_cta(&ad.sk_ad_network_data),
        unified_card_override: if ad.unified_card_override.is_empty() {
            None
        } else {
            Some(ad.unified_card_override.clone())
        },
        render_legacy_website_card: Some(false),
        render_sales_cta_website_card: Some(ad.render_sales_cta_website_card),
        render_custom_cta_website_card: None,
        is_quick_promote: Some(ad.is_quick_promote),
        destination_url_params: if ad.destination_url_params.is_empty() {
            None
        } else {
            Some(
                ad.destination_url_params
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )
        },
        is_external_browser: Some(ad.is_external_browser),
        model_scores: None,
        likely_to_open_link: Some(likely_to_open_link(ad)),
        promoted_by_id: ad.promoted_by.filter(|&id| id != 0),
        overlay_media: if ad.overlay_media_id == 0 {
            None
        } else {
            Some(ApiMediaKey::TweetImage(TweetImageKey::new(
                ad.overlay_media_id,
                0,
            )))
        },
        show_overlay_on_first_card: Some(ad.overlay_on_first_card),
        boost_cta: ad.boost_cta.as_ref().and_then(|cta| {
            let boosted_by = cta.boosted_by.filter(|&id| id != 0)?;
            Some(BoostCta::new(
                cta.action_url.clone(),
                boosted_by,
                cta.action_text.map(BoostCtaActionText),
            ))
        }),
        allow_spine_clicks: Some(ad.allow_spine_clicks),
    };

    let promoted_metadata = PromotedMetadata {
        advertiser_id: ad.author_id,
        impression_id: Some(impression_string.clone()),
        disclosure_type: None,
        experiment_values: None,
        promoted_trend_id: None,
        promoted_trend_name: None,
        promoted_trend_query_term: None,
        ad_metadata_container: Some(ad_metadata_container),
        promoted_trend_description: None,
        impression_string: Some(impression_string),
        click_tracking_info: Some(click_tracking_info),
        advertiser_name: None,
        rtb_ad_metadata: rtb_ad_metadata(ad),
    };

    let preroll_metadata = ad.preroll_info.as_ref().map(|pi| {
        let call_to_action = if pi.cta_type.is_empty() {
            None
        } else {
            Some(CallToAction {
                call_to_action_type: Some(pi.cta_type.clone()),
                url: Some(pi.cta_url.clone()),
                type_: None,
            })
        };

        let video_variants: Vec<VideoVariant> = pi
            .video_variants
            .iter()
            .map(|v| VideoVariant {
                url: Some(v.url.clone()),
                content_type: Some(v.content_type.clone()),
                bitrate: Some(v.bitrate),
            })
            .collect();

        let media_info = MediaInfo {
            uuid: Some(pi.creative_media_id.clone()),
            publisher_id: Some(ad.author_id),
            call_to_action,
            duration_millis: Some(pi.duration_millis),
            video_variants: Some(video_variants),
            advertiser_name: Some(pi.advertiser_name.clone()),
            render_ad_by_advertiser_name: Some(!pi.is_sponsorship),
            advertiser_profile_image_url: Some(pi.advertiser_profile_image_url.clone()),
        };

        PrerollMetadata {
            preroll: Some(Preroll {
                preroll_id: Some(pi.creative_media_id.clone()),
                dynamic_preroll_type: Some(DynamicPrerollType::AMPLIFY),
                media_info: Some(media_info),
            }),
            video_analytics_scribe_passthrough: None,
            promoted_content: None,
        }
    });

    let mut tweet = make_tweet(ad.post_id as u64);
    tweet.promoted_metadata = Some(promoted_metadata);
    tweet.preroll_metadata = preroll_metadata;
    tweet.contextual_tweet_ref = Some(contextual_ref(ad.post_id));

    TimelineEntry {
        entry_id,
        sort_index,
        content: TimelineEntryContent::Item(make_tweet_item(
            tweet,
            Some(ad_client_event_info(ad, request_type)),
            None,
        )),
        expiry_time: None,
    }
}

fn marshal_header_image_prompt_ad(
    ad: &AdIndexInfo,
    info: &HeaderImagePromptAdInfo,
    sort_index: i64,
    request_type: RequestType,
    impression_string: &str,
) -> TimelineEntry {
    let header_image = MessageImage {
        image_variants: BTreeSet::from([ImageVariant {
            url: info.image_url.clone(),
            width: info.image_width,
            height: info.image_height,
            palette: None,
        }]),
        background_color: None,
    };

    let plain_rich_text = |text: &str| RichText {
        text: text.to_string(),
        entities: vec![],
        rtl: None,
        alignment: None,
    };

    let button_action =
        |text: &str, url: &str, cta_action: &str, callbacks: &[String]| MessageTextAction {
            text: text.to_string(),
            action: MessageAction {
                dismiss_on_click: info.dismiss_on_click,
                url: Some(url.to_string()),
                client_event_info: Some(ClientEventInfo {
                    component: None,
                    element: None,
                    details: None,
                    action: Some(cta_action.to_string()),
                    entity_token: None,
                }),
                on_click_callbacks: to_callbacks(callbacks),
                on_click_reactive_trigger: None,
            },
        };

    let primary_button_action = button_action(
        &info.button_text,
        &info.landing_url,
        "primary_cta",
        &info.click_callback_endpoints,
    );
    let secondary_button_action = (!info.secondary_button_text.is_empty()).then(|| {
        button_action(
            &info.secondary_button_text,
            &info.secondary_landing_url,
            "secondary_cta",
            &[],
        )
    });

    let body_text = (!info.body_text.is_empty()).then(|| info.body_text.clone());
    let content = MessageContent::HeaderImagePrompt(HeaderImagePrompt {
        header_image,
        header_text: Some(info.header_text.clone()),
        body_text: body_text.clone(),
        primary_button_action: Some(primary_button_action),
        secondary_button_action,
        action: None,
        header_rich_text: Some(plain_rich_text(&info.header_text)),
        body_rich_text: body_text.as_deref().map(plain_rich_text),
    });

    let message_prompt = MessagePrompt {
        content,
        impression_callbacks: to_callbacks(&info.impression_callback_endpoints),
    };

    let mut client_event_info = ad_client_event_info(ad, request_type);
    client_event_info.element = None;

    TimelineEntry {
        entry_id: format!("{ENTRY_NAMESPACE_HEADER_IMAGE_PROMPT_AD}-{impression_string}"),
        sort_index,
        content: TimelineEntryContent::Item(TimelineItem {
            content: TimelineItemContent::Message(message_prompt),
            client_event_info: Some(client_event_info),
            feedback_info: None,
            prompt: None,
            reactive_triggers: None,
        }),
        expiry_time: None,
    }
}

fn to_callbacks(endpoints: &[String]) -> Option<Vec<Callback>> {
    if endpoints.is_empty() {
        return None;
    }
    Some(
        endpoints
            .iter()
            .map(|e| Callback {
                endpoint: e.clone(),
            })
            .collect(),
    )
}

fn rtb_ad_metadata(ad: &AdIndexInfo) -> Option<Box<RTBAdMetadata>> {
    ad.rtb_ad_metadata.as_ref().map(|m| {
        Box::new(RTBAdMetadata {
            dsp_id: Some(DspId::from(m.dsp_id)),
            response_content: (!m.response_content.is_empty()).then(|| m.response_content.clone()),
        })
    })
}

fn marshal_ssp_ad(ad: &AdIndexInfo, sort_index: i64, impression_string: &str) -> TimelineEntry {
    let dsp_impression_string = format!("{DSP_IMPRESSION_PREAMBLE}{impression_string}");
    let promoted_metadata = PromotedMetadata {
        advertiser_id: ad.author_id,
        impression_id: Some(dsp_impression_string.clone()),
        disclosure_type: None,
        experiment_values: None,
        promoted_trend_id: None,
        promoted_trend_name: None,
        promoted_trend_query_term: None,
        ad_metadata_container: None,
        promoted_trend_description: None,
        impression_string: Some(dsp_impression_string),
        click_tracking_info: None,
        advertiser_name: None,
        rtb_ad_metadata: rtb_ad_metadata(ad),
    };

    let rtb_image_ad = RtbImageAd {
        promoted_metadata,
        text: None,
        title: GOOGLE_SDK_NATIVE_AD.to_string(),
        landing_url: Url {
            url_type: UrlType::EXTERNAL_URL,
            url: EMPTY_LANDING_URL.to_string(),
            urt_endpoint_options: None,
        },
        vanity_url: Some(GOOGLE_SDK_NATIVE_AD.to_string()),
        image: Some(ImageVariant {
            url: GOOGLE_SDK_NATIVE_AD.to_string(),
            width: 0,
            height: 0,
            palette: None,
        }),
        creative_id: Some(DEFAULT_DSP_CREATIVE_ID.to_string()),
        advertiser_name: None,
        advertiser_avatar_image_url: None,
        bundle: None,
    };

    TimelineEntry {
        entry_id: format!("{ENTRY_NAMESPACE_RTB_IMAGE_AD}-{impression_string}"),
        sort_index,
        content: TimelineEntryContent::Item(TimelineItem {
            content: TimelineItemContent::RtbImageAd(rtb_image_ad),
            client_event_info: None,
            feedback_info: None,
            prompt: None,
            reactive_triggers: None,
        }),
        expiry_time: None,
    }
}

fn likely_to_open_link(ad: &AdIndexInfo) -> bool {
    ad.action_probabilities
        .get(CLIENT_TWEET_OPEN_LINK)
        .is_some_and(|&score| (score as f64) > LIKELY_TO_OPEN_LINK_THRESHOLD)
}

fn contextual_ref(post_id: i64) -> ContextualTweetRef {
    ContextualTweetRef {
        id: post_id,
        hydration_context: Some(Box::new(TweetHydrationContext {
            safety_level_override: Some(SafetyLevel::TIMELINE_HOME_PROMOTED_HYDRATION),
            outer_tweet_context: None,
        })),
    }
}

fn convert_skan(data: &xai_recsys_proto::SkAdNetworkData) -> SkAdNetworkData {
    SkAdNetworkData {
        version: Some(data.version.clone()),
        src_app_id: Some(data.src_app_id.clone()),
        dst_app_id: Some(data.dst_app_id.clone()),
        ad_network_id: Some(data.ad_network_id.clone()),
        campaign_id: Some(data.campaign_id),
        impression_time_in_millis: Some(data.impression_time_millis),
        nonce: Some(data.nonce.clone()),
        signature: Some(data.signature.clone()),
        fidelity_type: Some(data.fidelity_type),
    }
}

fn convert_skan_list(data: &[xai_recsys_proto::SkAdNetworkData]) -> Option<Vec<SkAdNetworkData>> {
    if data.is_empty() {
        return None;
    }
    Some(data.iter().map(convert_skan).collect())
}

fn convert_skan_cta(data: &[xai_recsys_proto::SkAdNetworkData]) -> Option<SkAdNetworkData> {
    data.iter().find(|d| d.fidelity_type == 1).map(convert_skan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_recsys_proto::RtbAdMetadata as PbRtbAdMetadata;
    use xai_urt_thrift::item::TimelineItemContent;

    fn promoted_metadata_for(ad: AdIndexInfo) -> PromotedMetadata {
        let entry = marshal_ad(&ad, 0, RequestType::ForYou);
        let item = match entry.content {
            TimelineEntryContent::Item(item) => item,
            _ => panic!("expected an item entry"),
        };
        let tweet = match item.content {
            TimelineItemContent::Tweet(tweet) => tweet,
            _ => panic!("expected a tweet item"),
        };
        tweet.promoted_metadata.expect("promoted_metadata present")
    }

    #[test]
    fn header_image_prompt_ad_marshals_to_message_prompt_entry() {
        let ad = AdIndexInfo {
            impression_id: 0xfa4ead01,
            header_image_prompt_ad_info: Some(HeaderImagePromptAdInfo {
                image_url: "https://example.com/ad.jpg".to_string(),
                image_width: 1199,
                image_height: 571,
                header_text: "Header".to_string(),
                body_text: "Body".to_string(),
                button_text: "Shop now".to_string(),
                landing_url: "https://example.com/land".to_string(),
                dismiss_on_click: true,
                impression_callback_endpoints: vec!["https://example.com/imp".to_string()],
                click_callback_endpoints: vec!["https://example.com/click".to_string()],
                secondary_button_text: String::new(),
                secondary_landing_url: String::new(),
            }),
            rtb_ad_metadata: Some(PbRtbAdMetadata {
                dsp_id: 0,
                response_content: "x".to_string(),
            }),
            ..Default::default()
        };

        let entry = marshal_ad(&ad, 5, RequestType::ForYou);
        assert_eq!(entry.entry_id, "header-image-prompt-ad-fa4ead01");
        assert_eq!(entry.sort_index, 5);

        let item = match entry.content {
            TimelineEntryContent::Item(item) => item,
            _ => panic!("expected an item entry"),
        };
        assert!(item.client_event_info.is_some());
        assert!(item.client_event_info.as_ref().unwrap().element.is_none());

        let prompt = match item.content {
            TimelineItemContent::Message(m) => m,
            other => panic!("expected a message prompt item, got {other:?}"),
        };
        assert_eq!(
            prompt.impression_callbacks.as_ref().unwrap()[0].endpoint,
            "https://example.com/imp"
        );

        let header = match prompt.content {
            MessageContent::HeaderImagePrompt(h) => h,
            other => panic!("expected header image prompt content, got {other:?}"),
        };
        let variant = header.header_image.image_variants.first().unwrap();
        assert_eq!(variant.url, "https://example.com/ad.jpg");
        assert_eq!((variant.width, variant.height), (1199, 571));
        assert_eq!(header.header_text.as_deref(), Some("Header"));
        assert_eq!(header.body_text.as_deref(), Some("Body"));

        let cta = header.primary_button_action.unwrap();
        assert_eq!(cta.text, "Shop now");
        assert_eq!(cta.action.url.as_deref(), Some("https://example.com/land"));
        assert_eq!(
            cta.action
                .client_event_info
                .as_ref()
                .and_then(|c| c.action.as_deref()),
            Some("primary_cta")
        );
        assert!(header.secondary_button_action.is_none());
        assert!(cta.action.dismiss_on_click);
        assert_eq!(
            cta.action.on_click_callbacks.unwrap()[0].endpoint,
            "https://example.com/click"
        );
    }

    #[test]
    fn header_image_prompt_ad_emits_secondary_button_when_set() {
        let ad = AdIndexInfo {
            impression_id: 0xfa4ead01,
            header_image_prompt_ad_info: Some(HeaderImagePromptAdInfo {
                image_url: "https://example.com/ad.jpg".to_string(),
                image_width: 1199,
                image_height: 571,
                header_text: "Header".to_string(),
                body_text: "Body".to_string(),
                button_text: "Shop now".to_string(),
                landing_url: "https://example.com/land".to_string(),
                dismiss_on_click: true,
                impression_callback_endpoints: vec![],
                click_callback_endpoints: vec![],
                secondary_button_text: "Learn more".to_string(),
                secondary_landing_url: "https://example.com".to_string(),
            }),
            ..Default::default()
        };

        let entry = marshal_ad(&ad, 0, RequestType::ForYou);
        let TimelineEntryContent::Item(item) = entry.content else {
            panic!("expected an item entry");
        };
        let TimelineItemContent::Message(prompt) = item.content else {
            panic!("expected a message prompt item");
        };
        let MessageContent::HeaderImagePrompt(header) = prompt.content else {
            panic!("expected header image prompt content");
        };

        let secondary = header.secondary_button_action.unwrap();
        assert_eq!(secondary.text, "Learn more");
        assert_eq!(secondary.action.url.as_deref(), Some("https://example.com"));
        assert_eq!(
            secondary
                .action
                .client_event_info
                .as_ref()
                .and_then(|c| c.action.as_deref()),
            Some("secondary_cta")
        );
    }

    #[test]
    fn header_image_prompt_ad_info_absent_stays_a_tweet_item() {
        let ad = AdIndexInfo {
            post_id: 42,
            impression_id: 0xabc,
            ..Default::default()
        };
        let entry = marshal_ad(&ad, 0, RequestType::ForYou);
        assert_eq!(entry.entry_id, "promoted-tweet-42-abc");
        let item = match entry.content {
            TimelineEntryContent::Item(item) => item,
            _ => panic!("expected an item entry"),
        };
        assert!(matches!(item.content, TimelineItemContent::Tweet(_)));
    }

    #[test]
    fn rtb_ad_metadata_present_maps_to_urt() {
        let ad = AdIndexInfo {
            post_id: 42,
            rtb_ad_metadata: Some(PbRtbAdMetadata {
                dsp_id: 0,
                response_content: "{\"ad_networks\":[{\"network\":\"google\"}]}".to_string(),
            }),
            ..Default::default()
        };
        let rtb = promoted_metadata_for(ad)
            .rtb_ad_metadata
            .expect("rtb_ad_metadata should be set");
        assert_eq!(rtb.dsp_id, Some(DspId::GOOGLE));
        assert_eq!(
            rtb.response_content.as_deref(),
            Some("{\"ad_networks\":[{\"network\":\"google\"}]}")
        );
    }

    #[test]
    fn empty_response_content_maps_to_none() {
        let ad = AdIndexInfo {
            post_id: 42,
            rtb_ad_metadata: Some(PbRtbAdMetadata {
                dsp_id: 0,
                response_content: String::new(),
            }),
            ..Default::default()
        };
        let rtb = promoted_metadata_for(ad)
            .rtb_ad_metadata
            .expect("rtb_ad_metadata should be set even with empty content");
        assert_eq!(rtb.dsp_id, Some(DspId::GOOGLE));
        assert_eq!(rtb.response_content, None);
    }

    #[test]
    fn dsp_id_maps_to_enum_value() {
        let ad = AdIndexInfo {
            post_id: 42,
            rtb_ad_metadata: Some(PbRtbAdMetadata {
                dsp_id: 6,
                response_content: "x".to_string(),
            }),
            ..Default::default()
        };
        let rtb = promoted_metadata_for(ad).rtb_ad_metadata.unwrap();
        assert_eq!(rtb.dsp_id, Some(DspId::TABOOLA));
    }

    #[test]
    fn absent_rtb_ad_metadata_yields_none() {
        let ad = AdIndexInfo {
            post_id: 42,
            ..Default::default()
        };
        assert!(promoted_metadata_for(ad).rtb_ad_metadata.is_none());
    }

    fn ssp_ad() -> AdIndexInfo {
        AdIndexInfo {
            post_id: 0,
            impression_id: 0xabc123,
            rtb_ad_metadata: Some(PbRtbAdMetadata {
                dsp_id: 0,
                response_content: "{\"ad_networks\":[]}".to_string(),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn ssp_ad_emits_rtb_image_ad_item() {
        let entry = marshal_ad(&ssp_ad(), 7, RequestType::ForYou);

        assert_eq!(entry.entry_id, "rtb-image-ad-abc123");
        assert_eq!(entry.sort_index, 7);

        let item = match entry.content {
            TimelineEntryContent::Item(item) => item,
            _ => panic!("expected an item entry"),
        };
        assert!(item.client_event_info.is_none());

        let rtb = match item.content {
            TimelineItemContent::RtbImageAd(rtb) => rtb,
            _ => panic!("expected an RtbImageAd item"),
        };

        assert_eq!(rtb.title, "GoogleSDKNativeAd");
        assert_eq!(rtb.landing_url.url, "emptyLandingUrl");
        assert_eq!(rtb.landing_url.url_type, UrlType::EXTERNAL_URL);
        assert_eq!(rtb.vanity_url.as_deref(), Some("GoogleSDKNativeAd"));
        let image = rtb.image.expect("image placeholder present");
        assert_eq!(image.url, "GoogleSDKNativeAd");
        assert_eq!((image.width, image.height), (0, 0));
        assert!(rtb.text.is_none());
        assert_eq!(rtb.creative_id.as_deref(), Some("default-dsp-creative-id"));
        assert!(rtb.advertiser_name.is_none());
        assert!(rtb.bundle.is_none());

        let pm = rtb.promoted_metadata;
        assert_eq!(pm.impression_string.as_deref(), Some("IS1:abc123"));
        assert_eq!(pm.impression_id.as_deref(), Some("IS1:abc123"));
        let meta = pm.rtb_ad_metadata.expect("rtb_ad_metadata present");
        assert_eq!(meta.dsp_id, Some(DspId::GOOGLE));
        assert_eq!(
            meta.response_content.as_deref(),
            Some("{\"ad_networks\":[]}")
        );
    }

    #[test]
    fn tweet_backed_ad_with_rtb_metadata_stays_a_tweet_item() {
        let ad = AdIndexInfo {
            post_id: 42,
            rtb_ad_metadata: Some(PbRtbAdMetadata {
                dsp_id: 0,
                response_content: "x".to_string(),
            }),
            ..Default::default()
        };
        let entry = marshal_ad(&ad, 0, RequestType::ForYou);
        assert_eq!(entry.entry_id, "promoted-tweet-42-0");
        let item = match entry.content {
            TimelineEntryContent::Item(item) => item,
            _ => panic!("expected an item entry"),
        };
        assert!(matches!(item.content, TimelineItemContent::Tweet(_)));
    }

    #[test]
    fn ssp_ad_without_rtb_metadata_stays_a_tweet_item() {
        let ad = AdIndexInfo {
            post_id: 0,
            ..Default::default()
        };
        let entry = marshal_ad(&ad, 0, RequestType::ForYou);
        let item = match entry.content {
            TimelineEntryContent::Item(item) => item,
            _ => panic!("expected an item entry"),
        };
        assert!(matches!(item.content, TimelineItemContent::Tweet(_)));
    }
}
