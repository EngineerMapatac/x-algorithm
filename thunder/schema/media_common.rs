
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_extern_crates)]
#![allow(clippy::too_many_arguments, clippy::type_complexity, clippy::vec_box, clippy::wrong_self_convention)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::{From, TryFrom};
use std::default::Default;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use thrift::OrderedFloat;
use thrift::{ApplicationError, ApplicationErrorKind, ProtocolError, ProtocolErrorKind, TThriftClient};
use thrift::protocol::{TFieldIdentifier, TListIdentifier, TMapIdentifier, TMessageIdentifier, TMessageType, TInputProtocol, TOutputProtocol, TSerializable, TSetIdentifier, TStructIdentifier, TType};
use thrift::protocol::field_id;
use thrift::protocol::verify_expected_message_type;
use thrift::protocol::verify_expected_sequence_number;
use thrift::protocol::verify_expected_service_call;
use thrift::protocol::verify_required_field_exists;
use thrift::server::TProcessor;

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediaCategory(pub i32);

impl MediaCategory {
  pub const AVATAR_IMAGE: MediaCategory = MediaCategory(0);
  pub const BANNER_IMAGE: MediaCategory = MediaCategory(1);
  pub const BACKGROUND_IMAGE: MediaCategory = MediaCategory(2);
  pub const TWEET_IMAGE: MediaCategory = MediaCategory(3);
  pub const CARD_IMAGE: MediaCategory = MediaCategory(4);
  pub const DM_IMAGE: MediaCategory = MediaCategory(5);
  pub const PREVIEW_IMAGE: MediaCategory = MediaCategory(6);
  pub const TWEET_VIDEO: MediaCategory = MediaCategory(7);
  pub const APP_IMAGE: MediaCategory = MediaCategory(9);
  pub const SUPPORT_IMAGE: MediaCategory = MediaCategory(10);
  pub const DM_GROUP_IMAGE: MediaCategory = MediaCategory(12);
  pub const AMPLIFY_VIDEO: MediaCategory = MediaCategory(13);
  pub const PERISCOPE_VIDEO: MediaCategory = MediaCategory(15);
  pub const TWEET_GIF: MediaCategory = MediaCategory(16);
  pub const DM_GIF: MediaCategory = MediaCategory(17);
  pub const DM_VIDEO: MediaCategory = MediaCategory(18);
  pub const AD_IMAGE: MediaCategory = MediaCategory(19);
  pub const PERISCOPE_AVATAR_IMAGE: MediaCategory = MediaCategory(20);
  pub const LIVE_EVENT_IMAGE: MediaCategory = MediaCategory(23);
  pub const B2C_PROFILE_IMAGE: MediaCategory = MediaCategory(24);
  pub const TWITPIC: MediaCategory = MediaCategory(25);
  pub const TWITTER_BROADCAST: MediaCategory = MediaCategory(26);
  pub const NEWS_IMAGE: MediaCategory = MediaCategory(27);
  pub const PERISCOPE_BROADCAST: MediaCategory = MediaCategory(28);
  pub const SUBTITLES: MediaCategory = MediaCategory(29);
  pub const SEMANTIC_CORE_IMAGE: MediaCategory = MediaCategory(30);
  pub const VIDEO_SLICE: MediaCategory = MediaCategory(31);
  pub const LIST_BANNER_IMAGE: MediaCategory = MediaCategory(32);
  pub const COMMERCE_PRODUCT_IMAGE: MediaCategory = MediaCategory(33);
  pub const COMMUNITY_BANNER_IMAGE: MediaCategory = MediaCategory(34);
  pub const MODEL_3D: MediaCategory = MediaCategory(35);
  pub const INTEGRATION_TEST_IMAGE: MediaCategory = MediaCategory(36);
  pub const PLAYABLE_MEDIA: MediaCategory = MediaCategory(37);
  pub const CUSTOMTIMELINE_BANNER_IMAGE: MediaCategory = MediaCategory(38);
  pub const CUSTOMTIMELINE_ICON_IMAGE: MediaCategory = MediaCategory(39);
  pub const CUSTOMTIMELINE_ITEM_IMAGE: MediaCategory = MediaCategory(40);
  pub const COMPANY_LOGO_IMAGE: MediaCategory = MediaCategory(41);
  pub const GROK_GENERATED_IMAGE: MediaCategory = MediaCategory(42);
  pub const RESERVED_43: MediaCategory = MediaCategory(43);
  pub const RESERVED_44: MediaCategory = MediaCategory(44);
  pub const RESERVED_45: MediaCategory = MediaCategory(45);
  pub const RESERVED_46: MediaCategory = MediaCategory(46);
  pub const RESERVED_47: MediaCategory = MediaCategory(47);
  pub const RESERVED_48: MediaCategory = MediaCategory(48);
  pub const RESERVED_49: MediaCategory = MediaCategory(49);
  pub const RESERVED_50: MediaCategory = MediaCategory(50);
  pub const RESERVED_51: MediaCategory = MediaCategory(51);
  pub const RESERVED_52: MediaCategory = MediaCategory(52);
  pub const RESERVED_53: MediaCategory = MediaCategory(53);
  pub const RESERVED_54: MediaCategory = MediaCategory(54);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::AVATAR_IMAGE,
    Self::BANNER_IMAGE,
    Self::BACKGROUND_IMAGE,
    Self::TWEET_IMAGE,
    Self::CARD_IMAGE,
    Self::DM_IMAGE,
    Self::PREVIEW_IMAGE,
    Self::TWEET_VIDEO,
    Self::APP_IMAGE,
    Self::SUPPORT_IMAGE,
    Self::DM_GROUP_IMAGE,
    Self::AMPLIFY_VIDEO,
    Self::PERISCOPE_VIDEO,
    Self::TWEET_GIF,
    Self::DM_GIF,
    Self::DM_VIDEO,
    Self::AD_IMAGE,
    Self::PERISCOPE_AVATAR_IMAGE,
    Self::LIVE_EVENT_IMAGE,
    Self::B2C_PROFILE_IMAGE,
    Self::TWITPIC,
    Self::TWITTER_BROADCAST,
    Self::NEWS_IMAGE,
    Self::PERISCOPE_BROADCAST,
    Self::SUBTITLES,
    Self::SEMANTIC_CORE_IMAGE,
    Self::VIDEO_SLICE,
    Self::LIST_BANNER_IMAGE,
    Self::COMMERCE_PRODUCT_IMAGE,
    Self::COMMUNITY_BANNER_IMAGE,
    Self::MODEL_3D,
    Self::INTEGRATION_TEST_IMAGE,
    Self::PLAYABLE_MEDIA,
    Self::CUSTOMTIMELINE_BANNER_IMAGE,
    Self::CUSTOMTIMELINE_ICON_IMAGE,
    Self::CUSTOMTIMELINE_ITEM_IMAGE,
    Self::COMPANY_LOGO_IMAGE,
    Self::GROK_GENERATED_IMAGE,
    Self::RESERVED_43,
    Self::RESERVED_44,
    Self::RESERVED_45,
    Self::RESERVED_46,
    Self::RESERVED_47,
    Self::RESERVED_48,
    Self::RESERVED_49,
    Self::RESERVED_50,
    Self::RESERVED_51,
    Self::RESERVED_52,
    Self::RESERVED_53,
    Self::RESERVED_54,
  ];
}

impl TSerializable for MediaCategory {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaCategory> {
    let enum_value = i_prot.read_i32()?;
    Ok(MediaCategory::from(enum_value))
  }
}

impl From<i32> for MediaCategory {
  fn from(i: i32) -> Self {
    match i {
      0 => MediaCategory::AVATAR_IMAGE,
      1 => MediaCategory::BANNER_IMAGE,
      2 => MediaCategory::BACKGROUND_IMAGE,
      3 => MediaCategory::TWEET_IMAGE,
      4 => MediaCategory::CARD_IMAGE,
      5 => MediaCategory::DM_IMAGE,
      6 => MediaCategory::PREVIEW_IMAGE,
      7 => MediaCategory::TWEET_VIDEO,
      9 => MediaCategory::APP_IMAGE,
      10 => MediaCategory::SUPPORT_IMAGE,
      12 => MediaCategory::DM_GROUP_IMAGE,
      13 => MediaCategory::AMPLIFY_VIDEO,
      15 => MediaCategory::PERISCOPE_VIDEO,
      16 => MediaCategory::TWEET_GIF,
      17 => MediaCategory::DM_GIF,
      18 => MediaCategory::DM_VIDEO,
      19 => MediaCategory::AD_IMAGE,
      20 => MediaCategory::PERISCOPE_AVATAR_IMAGE,
      23 => MediaCategory::LIVE_EVENT_IMAGE,
      24 => MediaCategory::B2C_PROFILE_IMAGE,
      25 => MediaCategory::TWITPIC,
      26 => MediaCategory::TWITTER_BROADCAST,
      27 => MediaCategory::NEWS_IMAGE,
      28 => MediaCategory::PERISCOPE_BROADCAST,
      29 => MediaCategory::SUBTITLES,
      30 => MediaCategory::SEMANTIC_CORE_IMAGE,
      31 => MediaCategory::VIDEO_SLICE,
      32 => MediaCategory::LIST_BANNER_IMAGE,
      33 => MediaCategory::COMMERCE_PRODUCT_IMAGE,
      34 => MediaCategory::COMMUNITY_BANNER_IMAGE,
      35 => MediaCategory::MODEL_3D,
      36 => MediaCategory::INTEGRATION_TEST_IMAGE,
      37 => MediaCategory::PLAYABLE_MEDIA,
      38 => MediaCategory::CUSTOMTIMELINE_BANNER_IMAGE,
      39 => MediaCategory::CUSTOMTIMELINE_ICON_IMAGE,
      40 => MediaCategory::CUSTOMTIMELINE_ITEM_IMAGE,
      41 => MediaCategory::COMPANY_LOGO_IMAGE,
      42 => MediaCategory::GROK_GENERATED_IMAGE,
      43 => MediaCategory::RESERVED_43,
      44 => MediaCategory::RESERVED_44,
      45 => MediaCategory::RESERVED_45,
      46 => MediaCategory::RESERVED_46,
      47 => MediaCategory::RESERVED_47,
      48 => MediaCategory::RESERVED_48,
      49 => MediaCategory::RESERVED_49,
      50 => MediaCategory::RESERVED_50,
      51 => MediaCategory::RESERVED_51,
      52 => MediaCategory::RESERVED_52,
      53 => MediaCategory::RESERVED_53,
      54 => MediaCategory::RESERVED_54,
      _ => MediaCategory(i)
    }
  }
}

impl From<&i32> for MediaCategory {
  fn from(i: &i32) -> Self {
    MediaCategory::from(*i)
  }
}

impl From<MediaCategory> for i32 {
  fn from(e: MediaCategory) -> i32 {
    e.0
  }
}

impl From<&MediaCategory> for i32 {
  fn from(e: &MediaCategory) -> i32 {
    e.0
  }
}

pub type MediaId = i64;


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediaKey {
  pub media_category: Option<MediaCategory>,
  pub media_id: Option<MediaId>,
}

impl MediaKey {
  pub fn new<F1, F2>(media_category: F1, media_id: F2) -> MediaKey where F1: Into<Option<MediaCategory>>, F2: Into<Option<MediaId>> {
    MediaKey {
      media_category: media_category.into(),
      media_id: media_id.into(),
    }
  }
}

impl TSerializable for MediaKey {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaKey> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<MediaCategory> = None;
    let mut f_2: Option<MediaId> = Some(0);
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = MediaCategory::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = MediaKey {
      media_category: f_1,
      media_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MediaKey");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.media_category {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_category", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.media_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductKey {
  pub entity_id: Option<String>,
  pub media_key: Option<MediaKey>,
}

impl ProductKey {
  pub fn new<F1, F2>(entity_id: F1, media_key: F2) -> ProductKey where F1: Into<Option<String>>, F2: Into<Option<MediaKey>> {
    ProductKey {
      entity_id: entity_id.into(),
      media_key: media_key.into(),
    }
  }
}

impl TSerializable for ProductKey {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProductKey> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<MediaKey> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_string()?;
          f_1 = Some(val);
        },
        2 => {
          let val = MediaKey::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ProductKey {
      entity_id: f_1,
      media_key: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ProductKey");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.entity_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("entity_id", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media_key {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_key", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OwnershipInfo {
  pub user_id: Option<i64>,
  pub tweet_id: Option<i64>,
  pub dm_id: Option<i64>,
  pub entity_id: Option<String>,
  pub attributable_user_id: Option<i64>,
  pub fleet_id: Option<String>,
  pub twitter_article_id: Option<i64>,
}

impl OwnershipInfo {
  pub fn new<F1, F2, F3, F4, F5, F6, F7>(user_id: F1, tweet_id: F2, dm_id: F3, entity_id: F4, attributable_user_id: F5, fleet_id: F6, twitter_article_id: F7) -> OwnershipInfo where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<String>>, F5: Into<Option<i64>>, F6: Into<Option<String>>, F7: Into<Option<i64>> {
    OwnershipInfo {
      user_id: user_id.into(),
      tweet_id: tweet_id.into(),
      dm_id: dm_id.into(),
      entity_id: entity_id.into(),
      attributable_user_id: attributable_user_id.into(),
      fleet_id: fleet_id.into(),
      twitter_article_id: twitter_article_id.into(),
    }
  }
}

impl TSerializable for OwnershipInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<OwnershipInfo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
    let mut f_4: Option<String> = None;
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<String> = None;
    let mut f_7: Option<i64> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_i64()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i64()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_string()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_i64()?;
          f_5 = Some(val);
        },
        6 => {
          let val = i_prot.read_string()?;
          f_6 = Some(val);
        },
        7 => {
          let val = i_prot.read_i64()?;
          f_7 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = OwnershipInfo {
      user_id: f_1,
      tweet_id: f_2,
      dm_id: f_3,
      entity_id: f_4,
      attributable_user_id: f_5,
      fleet_id: f_6,
      twitter_article_id: f_7,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("OwnershipInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.dm_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("dm_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.entity_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("entity_id", TType::String, 4))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.attributable_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("attributable_user_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.fleet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("fleet_id", TType::String, 6))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.twitter_article_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("twitterArticleId", TType::I64, 7))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Context {
  pub values: Option<BTreeMap<String, String>>,
}

impl Context {
  pub fn new<F1>(values: F1) -> Context where F1: Into<Option<BTreeMap<String, String>>> {
    Context {
      values: values.into(),
    }
  }
}

impl TSerializable for Context {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Context> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<BTreeMap<String, String>> = Some(BTreeMap::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, String> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_0 = i_prot.read_string()?;
            let map_val_1 = i_prot.read_string()?;
            val.insert(map_key_0, map_val_1);
          }
          i_prot.read_map_end()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Context {
      values: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Context");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.values {
      o_prot.write_field_begin(&TFieldIdentifier::new("values", TType::Map, 1))?;
      o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::String, fld_var.len() as i32))?;
      for (k, v) in fld_var {
        o_prot.write_string(k)?;
        o_prot.write_string(v)?;
      }
      o_prot.write_map_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UploadSourceInfo {
  pub url: Option<String>,
}

impl UploadSourceInfo {
  pub fn new<F1>(url: F1) -> UploadSourceInfo where F1: Into<Option<String>> {
    UploadSourceInfo {
      url: url.into(),
    }
  }
}

impl TSerializable for UploadSourceInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UploadSourceInfo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_string()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = UploadSourceInfo {
      url: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UploadSourceInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.url {
      o_prot.write_field_begin(&TFieldIdentifier::new("url", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

