
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
pub struct MediaSizeType(pub i32);

impl MediaSizeType {
  pub const ORIG: MediaSizeType = MediaSizeType(0);
  pub const LARGE: MediaSizeType = MediaSizeType(1);
  pub const MEDIUM: MediaSizeType = MediaSizeType(2);
  pub const SMALL: MediaSizeType = MediaSizeType(3);
  pub const THUMB: MediaSizeType = MediaSizeType(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::ORIG,
    Self::LARGE,
    Self::MEDIUM,
    Self::SMALL,
    Self::THUMB,
  ];
}

impl TSerializable for MediaSizeType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaSizeType> {
    let enum_value = i_prot.read_i32()?;
    Ok(MediaSizeType::from(enum_value))
  }
}

impl From<i32> for MediaSizeType {
  fn from(i: i32) -> Self {
    match i {
      0 => MediaSizeType::ORIG,
      1 => MediaSizeType::LARGE,
      2 => MediaSizeType::MEDIUM,
      3 => MediaSizeType::SMALL,
      4 => MediaSizeType::THUMB,
      _ => MediaSizeType(i)
    }
  }
}

impl From<&i32> for MediaSizeType {
  fn from(i: &i32) -> Self {
    MediaSizeType::from(*i)
  }
}

impl From<MediaSizeType> for i32 {
  fn from(e: MediaSizeType) -> i32 {
    e.0
  }
}

impl From<&MediaSizeType> for i32 {
  fn from(e: &MediaSizeType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediaResizeMethod(pub i32);

impl MediaResizeMethod {
  pub const FIT: MediaResizeMethod = MediaResizeMethod(0);
  pub const CROP: MediaResizeMethod = MediaResizeMethod(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::FIT,
    Self::CROP,
  ];
}

impl TSerializable for MediaResizeMethod {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaResizeMethod> {
    let enum_value = i_prot.read_i32()?;
    Ok(MediaResizeMethod::from(enum_value))
  }
}

impl From<i32> for MediaResizeMethod {
  fn from(i: i32) -> Self {
    match i {
      0 => MediaResizeMethod::FIT,
      1 => MediaResizeMethod::CROP,
      _ => MediaResizeMethod(i)
    }
  }
}

impl From<&i32> for MediaResizeMethod {
  fn from(i: &i32) -> Self {
    MediaResizeMethod::from(*i)
  }
}

impl From<MediaResizeMethod> for i32 {
  fn from(e: MediaResizeMethod) -> i32 {
    e.0
  }
}

impl From<&MediaResizeMethod> for i32 {
  fn from(e: &MediaResizeMethod) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediaContentType(pub i32);

impl MediaContentType {
  pub const IMAGE_GIF: MediaContentType = MediaContentType(0);
  pub const IMAGE_JPEG: MediaContentType = MediaContentType(1);
  pub const IMAGE_PNG: MediaContentType = MediaContentType(2);
  pub const VIDEO_MP4: MediaContentType = MediaContentType(3);
  pub const VIDEO_GENERIC: MediaContentType = MediaContentType(4);
  pub const RESERVED_5: MediaContentType = MediaContentType(5);
  pub const RESERVED_6: MediaContentType = MediaContentType(6);
  pub const RESERVED_7: MediaContentType = MediaContentType(7);
  pub const RESERVED_8: MediaContentType = MediaContentType(8);
  pub const RESERVED_9: MediaContentType = MediaContentType(9);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::IMAGE_GIF,
    Self::IMAGE_JPEG,
    Self::IMAGE_PNG,
    Self::VIDEO_MP4,
    Self::VIDEO_GENERIC,
    Self::RESERVED_5,
    Self::RESERVED_6,
    Self::RESERVED_7,
    Self::RESERVED_8,
    Self::RESERVED_9,
  ];
}

impl TSerializable for MediaContentType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaContentType> {
    let enum_value = i_prot.read_i32()?;
    Ok(MediaContentType::from(enum_value))
  }
}

impl From<i32> for MediaContentType {
  fn from(i: i32) -> Self {
    match i {
      0 => MediaContentType::IMAGE_GIF,
      1 => MediaContentType::IMAGE_JPEG,
      2 => MediaContentType::IMAGE_PNG,
      3 => MediaContentType::VIDEO_MP4,
      4 => MediaContentType::VIDEO_GENERIC,
      5 => MediaContentType::RESERVED_5,
      6 => MediaContentType::RESERVED_6,
      7 => MediaContentType::RESERVED_7,
      8 => MediaContentType::RESERVED_8,
      9 => MediaContentType::RESERVED_9,
      _ => MediaContentType(i)
    }
  }
}

impl From<&i32> for MediaContentType {
  fn from(i: &i32) -> Self {
    MediaContentType::from(*i)
  }
}

impl From<MediaContentType> for i32 {
  fn from(e: MediaContentType) -> i32 {
    e.0
  }
}

impl From<&MediaContentType> for i32 {
  fn from(e: &MediaContentType) -> i32 {
    e.0
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediaSize {
  pub size_type: Option<MediaSizeType>,
  pub resize_method: Option<MediaResizeMethod>,
  pub deprecated_content_type: Option<MediaContentType>,
  pub width: Option<i32>,
  pub height: Option<i32>,
}

impl MediaSize {
  pub fn new<F1, F2, F3, F4, F5>(size_type: F1, resize_method: F2, deprecated_content_type: F3, width: F4, height: F5) -> MediaSize where F1: Into<Option<MediaSizeType>>, F2: Into<Option<MediaResizeMethod>>, F3: Into<Option<MediaContentType>>, F4: Into<Option<i32>>, F5: Into<Option<i32>> {
    MediaSize {
      size_type: size_type.into(),
      resize_method: resize_method.into(),
      deprecated_content_type: deprecated_content_type.into(),
      width: width.into(),
      height: height.into(),
    }
  }
}

impl TSerializable for MediaSize {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaSize> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<MediaSizeType> = None;
    let mut f_2: Option<MediaResizeMethod> = None;
    let mut f_3: Option<MediaContentType> = None;
    let mut f_4: Option<i32> = Some(0);
    let mut f_5: Option<i32> = Some(0);
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = MediaSizeType::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = MediaResizeMethod::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = MediaContentType::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_i32()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_i32()?;
          f_5 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = MediaSize {
      size_type: f_1,
      resize_method: f_2,
      deprecated_content_type: f_3,
      width: f_4,
      height: f_5,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MediaSize");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.size_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("size_type", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.resize_method {
      o_prot.write_field_begin(&TFieldIdentifier::new("resize_method", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.deprecated_content_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("deprecated_content_type", TType::I32, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.width {
      o_prot.write_field_begin(&TFieldIdentifier::new("width", TType::I32, 4))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.height {
      o_prot.write_field_begin(&TFieldIdentifier::new("height", TType::I32, 5))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AspectRatio {
  pub numerator: Option<i16>,
  pub denominator: Option<i16>,
}

impl AspectRatio {
  pub fn new<F1, F2>(numerator: F1, denominator: F2) -> AspectRatio where F1: Into<Option<i16>>, F2: Into<Option<i16>> {
    AspectRatio {
      numerator: numerator.into(),
      denominator: denominator.into(),
    }
  }
}

impl TSerializable for AspectRatio {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AspectRatio> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i16> = Some(0);
    let mut f_2: Option<i16> = Some(0);
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = AspectRatio {
      numerator: f_1,
      denominator: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AspectRatio");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.numerator {
      o_prot.write_field_begin(&TFieldIdentifier::new("numerator", TType::I16, 1))?;
      o_prot.write_i16(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.denominator {
      o_prot.write_field_begin(&TFieldIdentifier::new("denominator", TType::I16, 2))?;
      o_prot.write_i16(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VideoVariant {
  pub url: Option<String>,
  pub content_type: Option<String>,
  pub bit_rate: Option<i32>,
}

impl VideoVariant {
  pub fn new<F1, F2, F3>(url: F1, content_type: F2, bit_rate: F3) -> VideoVariant where F1: Into<Option<String>>, F2: Into<Option<String>>, F3: Into<Option<i32>> {
    VideoVariant {
      url: url.into(),
      content_type: content_type.into(),
      bit_rate: bit_rate.into(),
    }
  }
}

impl TSerializable for VideoVariant {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<VideoVariant> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<i32> = None;
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
          let val = i_prot.read_string()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i32()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = VideoVariant {
      url: f_1,
      content_type: f_2,
      bit_rate: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("VideoVariant");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.url {
      o_prot.write_field_begin(&TFieldIdentifier::new("url", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.content_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("content_type", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.bit_rate {
      o_prot.write_field_begin(&TFieldIdentifier::new("bit_rate", TType::I32, 3))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Model3dAsset {
  pub url: Option<String>,
  pub content_type: Option<String>,
}

impl Model3dAsset {
  pub fn new<F1, F2>(url: F1, content_type: F2) -> Model3dAsset where F1: Into<Option<String>>, F2: Into<Option<String>> {
    Model3dAsset {
      url: url.into(),
      content_type: content_type.into(),
    }
  }
}

impl TSerializable for Model3dAsset {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Model3dAsset> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<String> = Some("".to_owned());
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
          let val = i_prot.read_string()?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Model3dAsset {
      url: f_1,
      content_type: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Model3dAsset");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.url {
      o_prot.write_field_begin(&TFieldIdentifier::new("url", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.content_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("content_type", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageInfo {
  pub unused: Option<String>,
}

impl ImageInfo {
  pub fn new<F1>(unused: F1) -> ImageInfo where F1: Into<Option<String>> {
    ImageInfo {
      unused: unused.into(),
    }
  }
}

impl TSerializable for ImageInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ImageInfo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = None;
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
    let ret = ImageInfo {
      unused: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ImageInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.unused {
      o_prot.write_field_begin(&TFieldIdentifier::new("unused", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AnimatedGifInfo {
  pub aspect_ratio: Option<AspectRatio>,
  pub variants: Option<BTreeSet<VideoVariant>>,
}

impl AnimatedGifInfo {
  pub fn new<F1, F2>(aspect_ratio: F1, variants: F2) -> AnimatedGifInfo where F1: Into<Option<AspectRatio>>, F2: Into<Option<BTreeSet<VideoVariant>>> {
    AnimatedGifInfo {
      aspect_ratio: aspect_ratio.into(),
      variants: variants.into(),
    }
  }
}

impl TSerializable for AnimatedGifInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AnimatedGifInfo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<AspectRatio> = None;
    let mut f_2: Option<BTreeSet<VideoVariant>> = Some(BTreeSet::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = AspectRatio::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<VideoVariant> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_0 = VideoVariant::read_from_in_protocol(i_prot)?;
            val.insert(set_elem_0);
          }
          i_prot.read_set_end()?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = AnimatedGifInfo {
      aspect_ratio: f_1,
      variants: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AnimatedGifInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.aspect_ratio {
      o_prot.write_field_begin(&TFieldIdentifier::new("aspect_ratio", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.variants {
      o_prot.write_field_begin(&TFieldIdentifier::new("variants", TType::Set, 2))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VideoInfo {
  pub duration_millis: Option<i32>,
  pub aspect_ratio: Option<AspectRatio>,
  pub variants: Option<BTreeSet<VideoVariant>>,
}

impl VideoInfo {
  pub fn new<F1, F2, F3>(duration_millis: F1, aspect_ratio: F2, variants: F3) -> VideoInfo where F1: Into<Option<i32>>, F2: Into<Option<AspectRatio>>, F3: Into<Option<BTreeSet<VideoVariant>>> {
    VideoInfo {
      duration_millis: duration_millis.into(),
      aspect_ratio: aspect_ratio.into(),
      variants: variants.into(),
    }
  }
}

impl TSerializable for VideoInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<VideoInfo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i32> = Some(0);
    let mut f_2: Option<AspectRatio> = None;
    let mut f_3: Option<BTreeSet<VideoVariant>> = Some(BTreeSet::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_i32()?;
          f_1 = Some(val);
        },
        2 => {
          let val = AspectRatio::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<VideoVariant> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_1 = VideoVariant::read_from_in_protocol(i_prot)?;
            val.insert(set_elem_1);
          }
          i_prot.read_set_end()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = VideoInfo {
      duration_millis: f_1,
      aspect_ratio: f_2,
      variants: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("VideoInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.duration_millis {
      o_prot.write_field_begin(&TFieldIdentifier::new("duration_millis", TType::I32, 1))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.aspect_ratio {
      o_prot.write_field_begin(&TFieldIdentifier::new("aspect_ratio", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.variants {
      o_prot.write_field_begin(&TFieldIdentifier::new("variants", TType::Set, 3))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Model3dInfo {
  pub unused: Option<String>,
  pub assets: Option<BTreeSet<Model3dAsset>>,
}

impl Model3dInfo {
  pub fn new<F1, F2>(unused: F1, assets: F2) -> Model3dInfo where F1: Into<Option<String>>, F2: Into<Option<BTreeSet<Model3dAsset>>> {
    Model3dInfo {
      unused: unused.into(),
      assets: assets.into(),
    }
  }
}

impl TSerializable for Model3dInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Model3dInfo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = None;
    let mut f_2: Option<BTreeSet<Model3dAsset>> = Some(BTreeSet::new());
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
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<Model3dAsset> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_2 = Model3dAsset::read_from_in_protocol(i_prot)?;
            val.insert(set_elem_2);
          }
          i_prot.read_set_end()?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Model3dInfo {
      unused: f_1,
      assets: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Model3dInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.unused {
      o_prot.write_field_begin(&TFieldIdentifier::new("unused", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.assets {
      o_prot.write_field_begin(&TFieldIdentifier::new("assets", TType::Set, 2))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MediaInfo {
  ImageInfo(ImageInfo),
  AnimatedGifInfo(AnimatedGifInfo),
  VideoInfo(VideoInfo),
  Model3dInfo(Model3dInfo),
}

impl TSerializable for MediaInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaInfo> {
    let mut ret: Option<MediaInfo> = None;
    let mut received_field_count = 0;
    i_prot.read_struct_begin()?;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = ImageInfo::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(MediaInfo::ImageInfo(val));
          }
          received_field_count += 1;
        },
        2 => {
          let val = AnimatedGifInfo::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(MediaInfo::AnimatedGifInfo(val));
          }
          received_field_count += 1;
        },
        3 => {
          let val = VideoInfo::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(MediaInfo::VideoInfo(val));
          }
          received_field_count += 1;
        },
        4 => {
          let val = Model3dInfo::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(MediaInfo::Model3dInfo(val));
          }
          received_field_count += 1;
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
          received_field_count += 1;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    if received_field_count == 0 {
      Err(
        thrift::Error::Protocol(
          ProtocolError::new(
            ProtocolErrorKind::InvalidData,
            "received empty union from remote MediaInfo"
          )
        )
      )
    } else if received_field_count > 1 {
      Err(
        thrift::Error::Protocol(
          ProtocolError::new(
            ProtocolErrorKind::InvalidData,
            "received multiple fields for union from remote MediaInfo"
          )
        )
      )
    } else {
      Ok(ret.expect("return value should have been constructed"))
    }
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MediaInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    match *self {
      MediaInfo::ImageInfo(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("image_info", TType::Struct, 1))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      MediaInfo::AnimatedGifInfo(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("animated_gif_info", TType::Struct, 2))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      MediaInfo::VideoInfo(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("video_info", TType::Struct, 3))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      MediaInfo::Model3dInfo(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("model_3d_info", TType::Struct, 4))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

