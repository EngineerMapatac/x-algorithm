use super::client_event::{wtf_item_client_event_info, wtf_module_client_event_info};

const ENTRY_NAMESPACE_USER: &str = "user";
const ENTRY_NAMESPACE_WTF: &str = "who-to-follow";
use xai_account_recommendations_mixer_proto as wtf_proto;
use xai_home_mixer_proto::WhoToFollowModule;
use xai_urt_thrift::entry::{TimelineEntry, TimelineEntryContent};
use xai_urt_thrift::item::{TimelineItem, TimelineItemContent};
use xai_urt_thrift::item_user::{User, UserDisplayType};
use xai_urt_thrift::metadata::{ContextType, GeneralContext, Url, UrlType};
use xai_urt_thrift::promoted::PromotedMetadata;
use xai_urt_thrift::social_context::SocialContext;
use xai_urt_thrift::timeline_module::{
    ModuleDisplayType, ModuleFooter, ModuleHeader, ModuleItem, TimelineModule,
};

use xai_candidate_pipeline::component_library::utils::client_utils::ClientPlatform;

pub(super) fn marshal_wtf(
    wtf: &WhoToFollowModule,
    sort_index: i64,
    client_app_id: i32,
    module_id: i64,
) -> Option<TimelineEntry> {
    let is_android = ClientPlatform::is_android(client_app_id);
    let user_display_type = if is_android {
        UserDisplayType::USER_COMPACT
    } else {
        UserDisplayType::USER
    };
    let module_display_type = if is_android {
        ModuleDisplayType::COMPACT_CAROUSEL
    } else {
        ModuleDisplayType::VERTICAL
    };
    let resp = wtf.who_to_follow_response.as_ref()?;

    let module_entry_id = format!("{}-{}", ENTRY_NAMESPACE_WTF, module_id);

    let header = resp.header.as_ref().and_then(|h| {
        h.title.as_ref().map(|title| ModuleHeader {
            text: title.clone(),
            sticky: Some(false),
            context: None,
            social_context: None,
            icon: None,
            button: None,
            custom_icon: None,
            display_type: None,
            landing_url: None,
        })
    });

    let footer = resp.footer.as_ref().and_then(|f| {
        f.action.as_ref().map(|action| ModuleFooter {
            text: action.title.clone(),
            url: None,
            landing_url: Some(Url {
                url_type: UrlType::DEEP_LINK,
                url: action.action_url.clone(),
                urt_endpoint_options: None,
            }),
            dismiss_after_interaction: None,
            display_type: None,
        })
    });

    let module_items: Vec<ModuleItem> = resp
        .user_recommendations
        .iter()
        .map(|rec| {
            let social_context = rec.social_text.as_ref().map(|text| {
                let context_type = match rec.context_type() {
                    wtf_proto::ContextType::Geo => ContextType::LOCATION,
                    wtf_proto::ContextType::NewUser => ContextType::NEW_USER,
                    _ => ContextType::FOLLOW,
                };
                SocialContext::GeneralContext(GeneralContext {
                    context_type,
                    text: text.clone(),
                    url: None,
                    context_image_urls: None,
                    landing_url: None,
                })
            });

            let promoted_metadata = rec.ad_impression.as_ref().and_then(|imp| {
                imp.impression_id.as_ref().map(|id| PromotedMetadata {
                    advertiser_id: rec.user_id,
                    impression_id: None,
                    disclosure_type: None,
                    experiment_values: None,
                    promoted_trend_id: None,
                    promoted_trend_name: None,
                    promoted_trend_query_term: None,
                    ad_metadata_container: None,
                    promoted_trend_description: None,
                    impression_string: Some(id.clone()),
                    click_tracking_info: None,
                    advertiser_name: None,
                    rtb_ad_metadata: None,
                })
            });

            let user = User {
                id: rec.user_id,
                display_type: user_display_type,
                promoted_metadata,
                social_context,
                highlights: None,
                enable_reactive_blending: Some(true),
                reactive_triggers: None,
                awards_given: None,
            };
            ModuleItem {
                entry_id: format!(
                    "{}-{}-{}",
                    module_entry_id, ENTRY_NAMESPACE_USER, rec.user_id
                ),
                item: TimelineItem {
                    content: TimelineItemContent::User(user),
                    client_event_info: Some(wtf_item_client_event_info(
                        rec.tracking_token.as_deref(),
                    )),
                    feedback_info: None,
                    prompt: None,
                    reactive_triggers: None,
                },
                dispensable: None,
                tree_display: None,
                pill_group: None,
            }
        })
        .collect();

    let first_tracking_token = resp
        .user_recommendations
        .first()
        .and_then(|rec| rec.tracking_token.as_deref());

    Some(TimelineEntry {
        entry_id: module_entry_id,
        sort_index,
        content: TimelineEntryContent::TimelineModule(TimelineModule {
            items: module_items,
            display_type: module_display_type,
            header,
            footer,
            client_event_info: Some(wtf_module_client_event_info(first_tracking_token)),
            feedback_info: None,
            metadata: None,
            show_more_behavior: None,
        }),
        expiry_time: None,
    })
}
