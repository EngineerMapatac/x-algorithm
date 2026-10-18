use std::collections::BTreeSet;
use tracing::warn;
use xai_home_mixer_proto::Prompt as PromptProto;
use xai_prompts_thrift::action::ButtonAction as FlipButtonAction;
use xai_prompts_thrift::action::ButtonBehavior as FlipButtonBehavior;
use xai_prompts_thrift::action::CtaButtonStyle as FlipCtaButtonStyle;
use xai_prompts_thrift::callback::Callback as FlipCallback;
use xai_prompts_thrift::client_event_info::ClientEventInfo as FlipClientEventInfo;
use xai_prompts_thrift::cover::{FullCover as FlipFullCover, HalfCover as FlipHalfCover};
use xai_prompts_thrift::cover_display_type::HalfCoverDisplayType as FlipHalfCoverDisplayType;
use xai_prompts_thrift::dismiss_info::DismissInfo as FlipDismissInfo;
use xai_prompts_thrift::feedback_info::{FeedbackAction as FlipFeedbackAction, FeedbackInfo};
use xai_prompts_thrift::horizon_icon::HorizonIcon as FlipHorizonIcon;
use xai_prompts_thrift::image::{
    Image as FlipImage, ImageAnimationType as FlipImageAnimationType,
    ImageDisplayType as FlipImageDisplayType,
};
use xai_prompts_thrift::injection::Injection;
use xai_prompts_thrift::inline_prompt::InlinePrompt as FlipInlinePrompt;
use xai_prompts_thrift::prompt_user_facepile::PromptUserFacepile as FlipUserFacepile;
use xai_prompts_thrift::relevance_prompt::{
    RelevancePrompt as FlipRelevancePrompt, RelevancePromptDisplayType as FlipRelevanceDisplayType,
};
use xai_prompts_thrift::rich_text::{
    ReferenceObject as FlipRefObj, RichText as FlipRichText, RichTextAlignment as FlipAlignment,
    RichTextFormat as FlipFormat,
};
use xai_prompts_thrift::url::{Url as FlipUrl, UrlType as FlipUrlType};
use xai_urt_thrift::button::ButtonStyle;
use xai_urt_thrift::cover::{
    self as urt_cover, CoverBehaviorDismiss, CoverBehaviorNavigate, CoverCta, CoverCtaBehavior,
    CoverImage,
};
use xai_urt_thrift::entry::{TimelineEntry, TimelineEntryContent};
use xai_urt_thrift::icon::HorizonIcon;
use xai_urt_thrift::instruction::{ShowCover, TimelineInstruction};
use xai_urt_thrift::item::{TimelineItem, TimelineItemContent};
use xai_urt_thrift::message::{
    HeaderImagePrompt, InlinePrompt, MessageAction, MessageContent, MessageImage, MessagePrompt,
    MessageTextAction, UserFacepile,
};
use xai_urt_thrift::metadata::{
    self, Callback, ClientEventDetails, ClientEventInfo, DismissInfo,
    FeedbackInfo as UrtFeedbackInfo, GeneralContext, ImageVariant, TimelinesDetails, Url, UrlType,
    UrtEndpointOptions,
};
use xai_urt_thrift::prompt::{Prompt, PromptContent};
use xai_urt_thrift::relevance_prompt::{RelevancePrompt, RelevancePromptDisplayType};
use xai_urt_thrift::richtext::{
    ReferenceObject, RichText, RichTextAlignment, RichTextCashtag, RichTextEntity, RichTextFormat,
    RichTextHashtag, RichTextList, RichTextMention, RichTextUser,
};
use xai_urt_thrift::social_context::SocialContext;

const ENTRY_NAMESPACE: &str = "flip-prompt-message";

pub(super) enum PromptUrtResult {
    Entry(TimelineEntry),
    Instruction(TimelineInstruction),
}

pub(super) fn marshal_prompt(module: &PromptProto, sort_index: i64) -> Option<PromptUrtResult> {
    let injection: Injection = xai_prompts_thrift::deserialize_binary(&module.injection)
        .map_err(|e| warn!("failed to deserialize injection: {e}"))
        .ok()?;

    match injection {
        Injection::InlinePrompt(p) => {
            marshal_inline_prompt(&p, sort_index).map(PromptUrtResult::Entry)
        }
        Injection::RelevancePrompt(p) => {
            marshal_relevance_prompt(&p, sort_index).map(PromptUrtResult::Entry)
        }
        Injection::FullCover(c) => marshal_full_cover(&c).map(PromptUrtResult::Instruction),
        Injection::HalfCover(c) => marshal_half_cover(&c).map(PromptUrtResult::Instruction),
    }
}

fn marshal_inline_prompt(p: &FlipInlinePrompt, sort_index: i64) -> Option<TimelineEntry> {
    let id = p
        .injection_identifier
        .clone()
        .unwrap_or_else(|| "0".to_string());

    let content = match &p.image {
        Some(img) => MessageContent::HeaderImagePrompt(HeaderImagePrompt {
            header_image: convert_message_image(img),
            header_text: Some(p.header_text.text.clone()),
            body_text: p.body_text.as_ref().map(|b| b.text.clone()),
            primary_button_action: p.primary_action.as_ref().map(convert_button_action),
            secondary_button_action: p.secondary_action.as_ref().map(convert_button_action),
            action: None,
            header_rich_text: Some(convert_rich_text(&p.header_text)),
            body_rich_text: p.body_text.as_ref().map(convert_rich_text),
        }),
        None => MessageContent::InlinePrompt(InlinePrompt {
            header_text: p.header_text.text.clone(),
            body_text: p.body_text.as_ref().map(|b| b.text.clone()),
            primary_button_action: p.primary_action.as_ref().map(convert_button_action),
            secondary_button_action: p.secondary_action.as_ref().map(convert_button_action),
            header_rich_text: Some(convert_rich_text(&p.header_text)),
            body_rich_text: p.body_text.as_ref().map(convert_rich_text),
            social_context: p.social_context.as_ref().map(convert_social_context),
            user_facepile: p.prompt_user_facepile.as_ref().map(convert_user_facepile),
        }),
    };

    let message_prompt = MessagePrompt {
        content,
        impression_callbacks: p
            .impression_callbacks
            .as_ref()
            .map(|cbs| cbs.iter().map(convert_callback).collect()),
    };

    let client_event_info = build_inline_prompt_client_event_info(p);
    let feedback_info = p.feedback_info.as_ref().map(convert_feedback_info);

    let entry_id = format!("{}-{}", ENTRY_NAMESPACE, id);
    Some(TimelineEntry {
        entry_id,
        sort_index,
        content: TimelineEntryContent::Item(TimelineItem {
            content: TimelineItemContent::Message(message_prompt),
            client_event_info,
            feedback_info,
            prompt: None,
            reactive_triggers: None,
        }),
        expiry_time: None,
    })
}

fn build_inline_prompt_client_event_info(p: &FlipInlinePrompt) -> Option<ClientEventInfo> {
    let details = ClientEventDetails {
        timelines_details: Some(TimelinesDetails {
            injection_type: Some("Message".to_string()),
            controller_data: None,
            source_data: None,
        }),
        ..Default::default()
    };
    Some(ClientEventInfo {
        component: p.injection_identifier.clone(),
        element: None,
        details: Some(details),
        action: None,
        entity_token: None,
    })
}

fn marshal_full_cover(c: &FlipFullCover) -> Option<TimelineInstruction> {
    let cover = urt_cover::FullCover {
        display_type: urt_cover::FullCoverDisplayType::COVER,
        primary_text: convert_rich_text(&c.primary_text),
        primary_cover_cta: convert_cover_cta(&c.primary_button_action),
        secondary_cover_cta: c.secondary_button_action.as_ref().map(convert_cover_cta),
        secondary_text: c.secondary_text.as_ref().map(convert_rich_text),
        image: c.image.as_ref().map(convert_image_variant),
        details: c.detail_text.as_ref().map(convert_rich_text),
        dismiss_info: c.dismiss_info.as_ref().map(convert_dismiss_info),
        image_display_type: c.image.as_ref().map(convert_cover_image_display_type),
        impression_callbacks: c
            .impression_callbacks
            .as_ref()
            .map(|cbs| cbs.iter().map(convert_callback).collect()),
    };

    let client_event_info = convert_client_event_info(&c.client_event_info);

    Some(TimelineInstruction::ShowCover(ShowCover {
        cover: urt_cover::Cover::FullCover(cover),
        client_event_info: Some(client_event_info),
    }))
}

fn marshal_half_cover(c: &FlipHalfCover) -> Option<TimelineInstruction> {
    let display_type = c
        .display_type
        .as_ref()
        .map(convert_half_cover_display_type)
        .unwrap_or(urt_cover::HalfCoverDisplayType::COVER);

    let cover = urt_cover::HalfCover {
        display_type,
        primary_text: convert_rich_text(&c.primary_text),
        primary_cover_cta: convert_cover_cta(&c.primary_button_action),
        secondary_cover_cta: c.secondary_button_action.as_ref().map(convert_cover_cta),
        secondary_text: c.secondary_text.as_ref().map(convert_rich_text),
        image: None,
        impression_callbacks: c
            .impression_callbacks
            .as_ref()
            .map(|cbs| cbs.iter().map(convert_callback).collect()),
        dismissible: c.dismissible,
        cover_image: c.image.as_ref().map(convert_cover_image),
        dismiss_info: c.dismiss_info.as_ref().map(convert_dismiss_info),
    };

    let client_event_info = convert_client_event_info(&c.client_event_info);

    Some(TimelineInstruction::ShowCover(ShowCover {
        cover: urt_cover::Cover::HalfCover(cover),
        client_event_info: Some(client_event_info),
    }))
}

fn marshal_relevance_prompt(p: &FlipRelevancePrompt, sort_index: i64) -> Option<TimelineEntry> {
    let confirmation = build_relevance_confirmation(p);

    let is_relevant_callback = convert_relevance_callback(&p.is_relevant_button);
    let not_relevant_callback = convert_relevance_callback(&p.not_relevant_button);

    let display_type = match p.display_type {
        FlipRelevanceDisplayType::NORMAL => Some(RelevancePromptDisplayType::NORMAL),
        FlipRelevanceDisplayType::COMPACT => Some(RelevancePromptDisplayType::COMPACT),
        FlipRelevanceDisplayType::LARGE => Some(RelevancePromptDisplayType::LARGE),
        _ => None,
    };

    let relevance = RelevancePrompt {
        title: p.title.text.clone(),
        confirmation,
        is_relevant_text: p.is_relevant_button.text.clone(),
        not_relevant_text: p.not_relevant_button.text.clone(),
        is_relevant_callback,
        not_relevant_callback,
        reactive_triggers: None,
        display_type,
        is_relevant_follow_up: None,
        not_relevant_follow_up: None,
        subtitle: p.subtitle.clone(),
        neutral_text: None,
        neutral_callback: None,
    };

    let prompt = Prompt {
        content: PromptContent::RelevancePrompt(relevance),
        client_event_info: Some(ClientEventInfo {
            component: Some(format!("onboarding_{}", p.injection_identifier)),
            element: Some("relevance_prompt".to_string()),
            details: None,
            action: None,
            entity_token: None,
        }),
        impression_callbacks: Some(
            p.impression_callbacks
                .iter()
                .map(convert_callback)
                .collect(),
        ),
    };

    let entry_id = format!("{}-{}", ENTRY_NAMESPACE, p.injection_identifier);
    Some(TimelineEntry {
        entry_id,
        sort_index,
        content: TimelineEntryContent::Item(TimelineItem {
            content: TimelineItemContent::Prompt(prompt),
            client_event_info: None,
            feedback_info: None,
            prompt: None,
            reactive_triggers: None,
        }),
        expiry_time: None,
    })
}

fn build_relevance_confirmation(p: &FlipRelevancePrompt) -> String {
    button_dismiss_feedback_text(&p.is_relevant_button).unwrap_or_default()
}

fn button_dismiss_feedback_text(button: &FlipButtonAction) -> Option<String> {
    match &button.button_behavior {
        FlipButtonBehavior::Dismiss(d) => d.feedback_message.as_ref().map(|rt| rt.text.clone()),
        _ => None,
    }
}

fn convert_relevance_callback(button: &FlipButtonAction) -> Callback {
    let endpoint = button
        .callbacks
        .as_ref()
        .and_then(|cbs| cbs.first())
        .map(|cb| cb.endpoint.clone())
        .unwrap_or_default();
    Callback { endpoint }
}

fn convert_rich_text(rt: &FlipRichText) -> RichText {
    let entities = rt
        .entities
        .iter()
        .map(|e| RichTextEntity {
            from_index: e.from_index,
            to_index: e.to_index,
            ref_: e.ref_.as_ref().map(convert_reference_object),
            format: e.format.as_ref().map(convert_format),
        })
        .collect();
    RichText {
        text: rt.text.clone(),
        entities,
        rtl: rt.rtl,
        alignment: rt.alignment.as_ref().map(convert_alignment),
    }
}

fn convert_reference_object(r: &FlipRefObj) -> ReferenceObject {
    match r {
        FlipRefObj::Url(u) => ReferenceObject::Url(convert_url(u)),
        FlipRefObj::User(u) => ReferenceObject::User(RichTextUser { id: u.id }),
        FlipRefObj::Mention(m) => ReferenceObject::Mention(RichTextMention {
            id: m.id,
            screen_name: m.screen_name.clone(),
        }),
        FlipRefObj::Hashtag(h) => ReferenceObject::Hashtag(RichTextHashtag {
            text: h.text.clone(),
        }),
        FlipRefObj::Cashtag(c) => ReferenceObject::Cashtag(RichTextCashtag {
            text: c.text.clone(),
        }),
        FlipRefObj::TwitterList(l) => ReferenceObject::TwitterList(RichTextList {
            id: l.id,
            url: l.url.clone(),
        }),
    }
}

fn convert_format(f: &FlipFormat) -> RichTextFormat {
    match *f {
        FlipFormat::PLAIN => RichTextFormat::PLAIN,
        FlipFormat::STRONG => RichTextFormat::STRONG,
        _ => RichTextFormat::PLAIN,
    }
}

fn convert_alignment(a: &FlipAlignment) -> RichTextAlignment {
    match *a {
        FlipAlignment::NATURAL => RichTextAlignment::NATURAL,
        FlipAlignment::CENTER => RichTextAlignment::CENTER,
        _ => RichTextAlignment::NATURAL,
    }
}

fn convert_url(u: &FlipUrl) -> Url {
    let url_type = match u.url_type {
        FlipUrlType::EXTERNAL_URL => UrlType::EXTERNAL_URL,
        FlipUrlType::DEEP_LINK => UrlType::DEEP_LINK,
        FlipUrlType::URT_ENDPOINT => UrlType::URT_ENDPOINT,
        _ => UrlType::EXTERNAL_URL,
    };
    Url {
        url_type,
        url: u.url.clone(),
        urt_endpoint_options: u
            .urt_endpoint_options
            .as_ref()
            .map(|opts| UrtEndpointOptions {
                request_params: opts.request_params.clone(),
                title: opts.title.clone(),
                cache_id: opts.cache_id.clone(),
                subtitle: opts.subtitle.clone(),
                timeline_with_context: None,
            }),
    }
}

fn convert_callback(cb: &FlipCallback) -> Callback {
    Callback {
        endpoint: cb.endpoint.clone(),
    }
}

fn convert_button_action(ba: &FlipButtonAction) -> MessageTextAction {
    let action_url = match &ba.button_behavior {
        FlipButtonBehavior::Navigate(n) => Some(n.url.url.clone()),
        FlipButtonBehavior::Dismiss(_) => None,
    };
    let client_event_info = convert_client_event_info(&ba.client_event_info);
    MessageTextAction {
        text: ba.text.clone(),
        action: MessageAction {
            dismiss_on_click: ba.dismiss_on_click.unwrap_or(true),
            url: action_url,
            client_event_info: Some(client_event_info),
            on_click_callbacks: ba
                .callbacks
                .as_ref()
                .map(|cbs| cbs.iter().map(convert_callback).collect()),
            on_click_reactive_trigger: None,
        },
    }
}

fn convert_cover_cta(ba: &FlipButtonAction) -> CoverCta {
    CoverCta {
        text: ba.text.clone(),
        cta_behavior: convert_cover_cta_behavior(&ba.button_behavior),
        callbacks: ba
            .callbacks
            .as_ref()
            .map(|cbs| cbs.iter().map(convert_callback).collect()),
        client_event_info: Some(convert_client_event_info(&ba.client_event_info)),
        icon: ba.icon.as_ref().map(convert_horizon_icon),
        button_style: ba.button_style.as_ref().map(convert_button_style),
    }
}

fn convert_cover_cta_behavior(behavior: &FlipButtonBehavior) -> CoverCtaBehavior {
    match behavior {
        FlipButtonBehavior::Navigate(n) => CoverCtaBehavior::Navigate(CoverBehaviorNavigate {
            url: Some(convert_url(&n.url)),
        }),
        FlipButtonBehavior::Dismiss(d) => CoverCtaBehavior::Dismiss(CoverBehaviorDismiss {
            feedback_message: d.feedback_message.as_ref().map(convert_rich_text),
        }),
    }
}

fn convert_horizon_icon(icon: &FlipHorizonIcon) -> HorizonIcon {
    HorizonIcon(icon.0)
}

fn convert_button_style(style: &FlipCtaButtonStyle) -> ButtonStyle {
    ButtonStyle(style.0)
}

fn convert_half_cover_display_type(
    dt: &FlipHalfCoverDisplayType,
) -> urt_cover::HalfCoverDisplayType {
    match *dt {
        FlipHalfCoverDisplayType::COVER => urt_cover::HalfCoverDisplayType::COVER,
        FlipHalfCoverDisplayType::CENTER_COVER => urt_cover::HalfCoverDisplayType::CENTER_COVER,
        _ => urt_cover::HalfCoverDisplayType::COVER,
    }
}

fn convert_client_event_info(cei: &FlipClientEventInfo) -> ClientEventInfo {
    ClientEventInfo {
        component: cei.component.clone(),
        element: cei.element.clone(),
        details: None,
        action: cei.action.clone(),
        entity_token: None,
    }
}

fn convert_message_image(img: &FlipImage) -> MessageImage {
    let variant = ImageVariant {
        url: img.image.url.clone(),
        width: img.image.width,
        height: img.image.height,
        palette: None,
    };
    MessageImage {
        image_variants: BTreeSet::from([variant]),
        background_color: None,
    }
}

fn convert_image_variant(img: &FlipImage) -> ImageVariant {
    ImageVariant {
        url: img.image.url.clone(),
        width: img.image.width,
        height: img.image.height,
        palette: None,
    }
}

fn convert_cover_image(img: &FlipImage) -> CoverImage {
    CoverImage {
        image: convert_image_variant(img),
        image_display_type: convert_cover_image_display_type(img),
        image_animation_type: img
            .image_animation_type
            .as_ref()
            .map(convert_cover_image_animation_type),
    }
}

fn convert_cover_image_display_type(img: &FlipImage) -> urt_cover::ImageDisplayType {
    match img.image_display_type {
        FlipImageDisplayType::ICON => urt_cover::ImageDisplayType::ICON,
        FlipImageDisplayType::FULL_WIDTH => urt_cover::ImageDisplayType::FULL_WIDTH,
        FlipImageDisplayType::ICON_SMALL => urt_cover::ImageDisplayType::ICON_SMALL,
        _ => urt_cover::ImageDisplayType::ICON,
    }
}

fn convert_cover_image_animation_type(
    at: &FlipImageAnimationType,
) -> urt_cover::ImageAnimationType {
    match *at {
        FlipImageAnimationType::BOUNCE => urt_cover::ImageAnimationType::BOUNCE,
        _ => urt_cover::ImageAnimationType::BOUNCE,
    }
}

fn convert_dismiss_info(di: &FlipDismissInfo) -> DismissInfo {
    DismissInfo {
        callbacks: di
            .callbacks
            .as_ref()
            .map(|cbs| cbs.iter().map(convert_callback).collect()),
    }
}

fn convert_social_context(rt: &FlipRichText) -> SocialContext {
    SocialContext::GeneralContext(GeneralContext {
        context_type: metadata::ContextType::FOLLOW,
        text: rt.text.clone(),
        url: None,
        context_image_urls: None,
        landing_url: None,
    })
}

fn convert_user_facepile(uf: &FlipUserFacepile) -> UserFacepile {
    UserFacepile {
        user_ids: uf.user_ids.clone(),
        featured_user_ids: uf.featured_user_ids.clone(),
        action: uf.action.as_ref().map(convert_button_action),
        action_type: None,
        displays_featuring_text: uf.displays_featuring_text,
        display_type: None,
    }
}

fn convert_feedback_info(fi: &FeedbackInfo) -> UrtFeedbackInfo {
    let keys: Vec<String> = fi
        .actions
        .iter()
        .enumerate()
        .map(|(i, action)| match action {
            FlipFeedbackAction::DismissAction(_) => {
                format!("flip-dismiss-{}", i)
            }
        })
        .collect();
    UrtFeedbackInfo {
        feedback_keys: keys,
        feedback_metadata: None,
        display_context: None,
        client_event_info: None,
    }
}
