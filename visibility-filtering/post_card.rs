use xai_core_entities::entities::{GizmoduckUserProfile, PureCoreData};
use xai_x_thrift::media_entity::MediaEntity;
use xai_x_thrift::tweet_media::MediaInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostCard {
    pub id: u64,
    pub text: String,
    pub author: Author,
    pub media: Vec<CardMedia>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Author {
    pub id: u64,
    pub name: Option<String>,
    pub screen_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardMedia {
    pub kind: CardMediaKind,
    pub url: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardMediaKind {
    Image,
    Gif,
    Video,
    Model3d,
    Unknown,
}

impl PostCard {
    pub(crate) fn new(
        id: u64,
        core: PureCoreData,
        profile: Option<&GizmoduckUserProfile>,
        media: &[MediaEntity],
    ) -> Self {
        Self {
            id,
            text: core.text,
            author: Author {
                id: core.author_id,
                name: profile.map(|profile| profile.name.clone()),
                screen_name: profile.map(|profile| profile.screen_name.clone()),
                avatar_url: profile
                    .and_then(|profile| profile.profile_image_urls.as_ref())
                    .and_then(|urls| urls.normal.clone()),
            },
            media: media.iter().map(CardMedia::of).collect(),
        }
    }
}

impl CardMedia {
    fn of(entity: &MediaEntity) -> Self {
        Self {
            kind: match &entity.media_info {
                Some(MediaInfo::ImageInfo(_)) => CardMediaKind::Image,
                Some(MediaInfo::AnimatedGifInfo(_)) => CardMediaKind::Gif,
                Some(MediaInfo::VideoInfo(_)) => CardMediaKind::Video,
                Some(MediaInfo::Model3dInfo(_)) => CardMediaKind::Model3d,
                None => CardMediaKind::Unknown,
            },
            url: entity.media_url_https.clone(),
        }
    }
}
