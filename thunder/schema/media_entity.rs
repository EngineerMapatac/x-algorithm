
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

use crate::schema::media_common;
use crate::schema::tweet_media;


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ColorValue {
  pub red: Option<i8>,
  pub green: Option<i8>,
  pub blue: Option<i8>,
}

impl ColorValue {
  pub fn new<F1, F2, F3>(red: F1, green: F2, blue: F3) -> ColorValue where F1: Into<Option<i8>>, F2: Into<Option<i8>>, F3: Into<Option<i8>> {
    ColorValue {
      red: red.into(),
      green: green.into(),
      blue: blue.into(),
    }
  }
}

impl TSerializable for ColorValue {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ColorValue> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i8> = Some(0);
    let mut f_2: Option<i8> = Some(0);
    let mut f_3: Option<i8> = Some(0);
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_i8()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i8()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i8()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ColorValue {
      red: f_1,
      green: f_2,
      blue: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ColorValue");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.red {
      o_prot.write_field_begin(&TFieldIdentifier::new("red", TType::I08, 1))?;
      o_prot.write_i8(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.green {
      o_prot.write_field_begin(&TFieldIdentifier::new("green", TType::I08, 2))?;
      o_prot.write_i8(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.blue {
      o_prot.write_field_begin(&TFieldIdentifier::new("blue", TType::I08, 3))?;
      o_prot.write_i8(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediaEntity {
  pub from_index: Option<i16>,
  pub to_index: Option<i16>,
    pub url: Option<String>,
    pub display_url: Option<String>,
    pub media_url: Option<String>,
    pub media_url_https: Option<String>,
    pub expanded_url: Option<String>,
  pub media_id: Option<media_common::MediaId>,
  pub nsfw: Option<bool>,
  pub sizes: Option<BTreeSet<tweet_media::MediaSize>>,
  pub media_path: Option<String>,
  pub is_protected: Option<bool>,
      pub source_status_id: Option<i64>,
              pub source_user_id: Option<i64>,
            pub media_info: Option<tweet_media::MediaInfo>,
    pub media_key: Option<media_common::MediaKey>,
}

impl MediaEntity {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F21>(from_index: F1, to_index: F2, url: F3, display_url: F4, media_url: F5, media_url_https: F6, expanded_url: F7, media_id: F8, nsfw: F9, sizes: F10, media_path: F11, is_protected: F12, source_status_id: F13, source_user_id: F14, media_info: F15, media_key: F21) -> MediaEntity where F1: Into<Option<i16>>, F2: Into<Option<i16>>, F3: Into<Option<String>>, F4: Into<Option<String>>, F5: Into<Option<String>>, F6: Into<Option<String>>, F7: Into<Option<String>>, F8: Into<Option<media_common::MediaId>>, F9: Into<Option<bool>>, F10: Into<Option<BTreeSet<tweet_media::MediaSize>>>, F11: Into<Option<String>>, F12: Into<Option<bool>>, F13: Into<Option<i64>>, F14: Into<Option<i64>>, F15: Into<Option<tweet_media::MediaInfo>>, F21: Into<Option<media_common::MediaKey>> {
    MediaEntity {
      from_index: from_index.into(),
      to_index: to_index.into(),
      url: url.into(),
      display_url: display_url.into(),
      media_url: media_url.into(),
      media_url_https: media_url_https.into(),
      expanded_url: expanded_url.into(),
      media_id: media_id.into(),
      nsfw: nsfw.into(),
      sizes: sizes.into(),
      media_path: media_path.into(),
      is_protected: is_protected.into(),
      source_status_id: source_status_id.into(),
      source_user_id: source_user_id.into(),
      media_info: media_info.into(),
      media_key: media_key.into(),
    }
  }
}

impl TSerializable for MediaEntity {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaEntity> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i16> = Some(0);
    let mut f_2: Option<i16> = Some(0);
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<String> = Some("".to_owned());
    let mut f_5: Option<String> = Some("".to_owned());
    let mut f_6: Option<String> = Some("".to_owned());
    let mut f_7: Option<String> = Some("".to_owned());
    let mut f_8: Option<media_common::MediaId> = Some(0);
    let mut f_9: Option<bool> = Some(false);
    let mut f_10: Option<BTreeSet<tweet_media::MediaSize>> = Some(BTreeSet::new());
    let mut f_11: Option<String> = Some("".to_owned());
    let mut f_12: Option<bool> = None;
    let mut f_13: Option<i64> = None;
    let mut f_14: Option<i64> = None;
    let mut f_15: Option<tweet_media::MediaInfo> = None;
    let mut f_21: Option<media_common::MediaKey> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_i16()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i16()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_string()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_string()?;
          f_5 = Some(val);
        },
        6 => {
          let val = i_prot.read_string()?;
          f_6 = Some(val);
        },
        7 => {
          let val = i_prot.read_string()?;
          f_7 = Some(val);
        },
        8 => {
          let val = i_prot.read_i64()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<tweet_media::MediaSize> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_0 = tweet_media::MediaSize::read_from_in_protocol(i_prot)?;
            val.insert(set_elem_0);
          }
          i_prot.read_set_end()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_string()?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_bool()?;
          f_12 = Some(val);
        },
        13 => {
          let val = i_prot.read_i64()?;
          f_13 = Some(val);
        },
        14 => {
          let val = i_prot.read_i64()?;
          f_14 = Some(val);
        },
        15 => {
          let val = tweet_media::MediaInfo::read_from_in_protocol(i_prot)?;
          f_15 = Some(val);
        },
        21 => {
          let val = media_common::MediaKey::read_from_in_protocol(i_prot)?;
          f_21 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = MediaEntity {
      from_index: f_1,
      to_index: f_2,
      url: f_3,
      display_url: f_4,
      media_url: f_5,
      media_url_https: f_6,
      expanded_url: f_7,
      media_id: f_8,
      nsfw: f_9,
      sizes: f_10,
      media_path: f_11,
      is_protected: f_12,
      source_status_id: f_13,
      source_user_id: f_14,
      media_info: f_15,
      media_key: f_21,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MediaEntity");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.from_index {
      o_prot.write_field_begin(&TFieldIdentifier::new("from_index", TType::I16, 1))?;
      o_prot.write_i16(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.to_index {
      o_prot.write_field_begin(&TFieldIdentifier::new("to_index", TType::I16, 2))?;
      o_prot.write_i16(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.url {
      o_prot.write_field_begin(&TFieldIdentifier::new("url", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.display_url {
      o_prot.write_field_begin(&TFieldIdentifier::new("display_url", TType::String, 4))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media_url {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_url", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media_url_https {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_url_https", TType::String, 6))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.expanded_url {
      o_prot.write_field_begin(&TFieldIdentifier::new("expanded_url", TType::String, 7))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.media_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_id", TType::I64, 8))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.nsfw {
      o_prot.write_field_begin(&TFieldIdentifier::new("nsfw", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.sizes {
      o_prot.write_field_begin(&TFieldIdentifier::new("sizes", TType::Set, 10))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media_path {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_path", TType::String, 11))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_protected {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_protected", TType::Bool, 12))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.source_status_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_status_id", TType::I64, 13))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.source_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_user_id", TType::I64, 14))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media_info {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_info", TType::Struct, 15))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media_key {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_key", TType::Struct, 21))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

