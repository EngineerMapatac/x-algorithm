
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

use crate::schema::media_entity;

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlaceType(pub i32);

impl PlaceType {
  pub const UNKNOWN: PlaceType = PlaceType(0);
  pub const COUNTRY: PlaceType = PlaceType(1);
  pub const ADMIN: PlaceType = PlaceType(2);
  pub const CITY: PlaceType = PlaceType(3);
  pub const NEIGHBORHOOD: PlaceType = PlaceType(4);
  pub const POI: PlaceType = PlaceType(5);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::UNKNOWN,
    Self::COUNTRY,
    Self::ADMIN,
    Self::CITY,
    Self::NEIGHBORHOOD,
    Self::POI,
  ];
}

impl TSerializable for PlaceType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PlaceType> {
    let enum_value = i_prot.read_i32()?;
    Ok(PlaceType::from(enum_value))
  }
}

impl From<i32> for PlaceType {
  fn from(i: i32) -> Self {
    match i {
      0 => PlaceType::UNKNOWN,
      1 => PlaceType::COUNTRY,
      2 => PlaceType::ADMIN,
      3 => PlaceType::CITY,
      4 => PlaceType::NEIGHBORHOOD,
      5 => PlaceType::POI,
      _ => PlaceType(i)
    }
  }
}

impl From<&i32> for PlaceType {
  fn from(i: &i32) -> Self {
    PlaceType::from(*i)
  }
}

impl From<PlaceType> for i32 {
  fn from(e: PlaceType) -> i32 {
    e.0
  }
}

impl From<&PlaceType> for i32 {
  fn from(e: &PlaceType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlaceNameType(pub i32);

impl PlaceNameType {
  pub const NORMAL: PlaceNameType = PlaceNameType(0);
  pub const ABBREVIATION: PlaceNameType = PlaceNameType(1);
  pub const SYNONYM: PlaceNameType = PlaceNameType(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NORMAL,
    Self::ABBREVIATION,
    Self::SYNONYM,
  ];
}

impl TSerializable for PlaceNameType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PlaceNameType> {
    let enum_value = i_prot.read_i32()?;
    Ok(PlaceNameType::from(enum_value))
  }
}

impl From<i32> for PlaceNameType {
  fn from(i: i32) -> Self {
    match i {
      0 => PlaceNameType::NORMAL,
      1 => PlaceNameType::ABBREVIATION,
      2 => PlaceNameType::SYNONYM,
      _ => PlaceNameType(i)
    }
  }
}

impl From<&i32> for PlaceNameType {
  fn from(i: &i32) -> Self {
    PlaceNameType::from(*i)
  }
}

impl From<PlaceNameType> for i32 {
  fn from(e: PlaceNameType) -> i32 {
    e.0
  }
}

impl From<&PlaceNameType> for i32 {
  fn from(e: &PlaceNameType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediaTagType(pub i32);

impl MediaTagType {
  pub const USER: MediaTagType = MediaTagType(0);
  pub const RESERVED_1: MediaTagType = MediaTagType(1);
  pub const RESERVED_2: MediaTagType = MediaTagType(2);
  pub const RESERVED_3: MediaTagType = MediaTagType(3);
  pub const RESERVED_4: MediaTagType = MediaTagType(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::USER,
    Self::RESERVED_1,
    Self::RESERVED_2,
    Self::RESERVED_3,
    Self::RESERVED_4,
  ];
}

impl TSerializable for MediaTagType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaTagType> {
    let enum_value = i_prot.read_i32()?;
    Ok(MediaTagType::from(enum_value))
  }
}

impl From<i32> for MediaTagType {
  fn from(i: i32) -> Self {
    match i {
      0 => MediaTagType::USER,
      1 => MediaTagType::RESERVED_1,
      2 => MediaTagType::RESERVED_2,
      3 => MediaTagType::RESERVED_3,
      4 => MediaTagType::RESERVED_4,
      _ => MediaTagType(i)
    }
  }
}

impl From<&i32> for MediaTagType {
  fn from(i: &i32) -> Self {
    MediaTagType::from(*i)
  }
}

impl From<MediaTagType> for i32 {
  fn from(e: MediaTagType) -> i32 {
    e.0
  }
}

impl From<&MediaTagType> for i32 {
  fn from(e: &MediaTagType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SuggestType(pub i32);

impl SuggestType {
  pub const WTF_CARD: SuggestType = SuggestType(0);
  pub const WORLD_CUP: SuggestType = SuggestType(1);
  pub const WTD_CARD: SuggestType = SuggestType(2);
  pub const NEWS_CARD: SuggestType = SuggestType(3);
  pub const RESERVED_4: SuggestType = SuggestType(4);
  pub const RESERVED_5: SuggestType = SuggestType(5);
  pub const RESERVED_6: SuggestType = SuggestType(6);
  pub const RESERVED_7: SuggestType = SuggestType(7);
  pub const RESERVED_8: SuggestType = SuggestType(8);
  pub const RESERVED_9: SuggestType = SuggestType(9);
  pub const RESERVED_10: SuggestType = SuggestType(10);
  pub const RESERVED_11: SuggestType = SuggestType(11);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::WTF_CARD,
    Self::WORLD_CUP,
    Self::WTD_CARD,
    Self::NEWS_CARD,
    Self::RESERVED_4,
    Self::RESERVED_5,
    Self::RESERVED_6,
    Self::RESERVED_7,
    Self::RESERVED_8,
    Self::RESERVED_9,
    Self::RESERVED_10,
    Self::RESERVED_11,
  ];
}

impl TSerializable for SuggestType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SuggestType> {
    let enum_value = i_prot.read_i32()?;
    Ok(SuggestType::from(enum_value))
  }
}

impl From<i32> for SuggestType {
  fn from(i: i32) -> Self {
    match i {
      0 => SuggestType::WTF_CARD,
      1 => SuggestType::WORLD_CUP,
      2 => SuggestType::WTD_CARD,
      3 => SuggestType::NEWS_CARD,
      4 => SuggestType::RESERVED_4,
      5 => SuggestType::RESERVED_5,
      6 => SuggestType::RESERVED_6,
      7 => SuggestType::RESERVED_7,
      8 => SuggestType::RESERVED_8,
      9 => SuggestType::RESERVED_9,
      10 => SuggestType::RESERVED_10,
      11 => SuggestType::RESERVED_11,
      _ => SuggestType(i)
    }
  }
}

impl From<&i32> for SuggestType {
  fn from(i: &i32) -> Self {
    SuggestType::from(*i)
  }
}

impl From<SuggestType> for i32 {
  fn from(e: SuggestType) -> i32 {
    e.0
  }
}

impl From<&SuggestType> for i32 {
  fn from(e: &SuggestType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TwitterSuggestsVisibilityType(pub i32);

impl TwitterSuggestsVisibilityType {
    pub const PUBLIC: TwitterSuggestsVisibilityType = TwitterSuggestsVisibilityType(1);
    pub const RESTRICTED: TwitterSuggestsVisibilityType = TwitterSuggestsVisibilityType(2);
    pub const PRIVATE: TwitterSuggestsVisibilityType = TwitterSuggestsVisibilityType(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PUBLIC,
    Self::RESTRICTED,
    Self::PRIVATE,
  ];
}

impl TSerializable for TwitterSuggestsVisibilityType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TwitterSuggestsVisibilityType> {
    let enum_value = i_prot.read_i32()?;
    Ok(TwitterSuggestsVisibilityType::from(enum_value))
  }
}

impl From<i32> for TwitterSuggestsVisibilityType {
  fn from(i: i32) -> Self {
    match i {
      1 => TwitterSuggestsVisibilityType::PUBLIC,
      2 => TwitterSuggestsVisibilityType::RESTRICTED,
      3 => TwitterSuggestsVisibilityType::PRIVATE,
      _ => TwitterSuggestsVisibilityType(i)
    }
  }
}

impl From<&i32> for TwitterSuggestsVisibilityType {
  fn from(i: &i32) -> Self {
    TwitterSuggestsVisibilityType::from(*i)
  }
}

impl From<TwitterSuggestsVisibilityType> for i32 {
  fn from(e: TwitterSuggestsVisibilityType) -> i32 {
    e.0
  }
}

impl From<&TwitterSuggestsVisibilityType> for i32 {
  fn from(e: &TwitterSuggestsVisibilityType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpamSignalType(pub i32);

impl SpamSignalType {
  pub const MENTION: SpamSignalType = SpamSignalType(1);
  pub const SEARCH: SpamSignalType = SpamSignalType(2);
  pub const STREAMING: SpamSignalType = SpamSignalType(4);
  pub const RESERVED_VALUE_8: SpamSignalType = SpamSignalType(8);
  pub const RESERVED_VALUE_9: SpamSignalType = SpamSignalType(9);
  pub const RESERVED_VALUE_10: SpamSignalType = SpamSignalType(10);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::MENTION,
    Self::SEARCH,
    Self::STREAMING,
    Self::RESERVED_VALUE_8,
    Self::RESERVED_VALUE_9,
    Self::RESERVED_VALUE_10,
  ];
}

impl TSerializable for SpamSignalType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SpamSignalType> {
    let enum_value = i_prot.read_i32()?;
    Ok(SpamSignalType::from(enum_value))
  }
}

impl From<i32> for SpamSignalType {
  fn from(i: i32) -> Self {
    match i {
      1 => SpamSignalType::MENTION,
      2 => SpamSignalType::SEARCH,
      4 => SpamSignalType::STREAMING,
      8 => SpamSignalType::RESERVED_VALUE_8,
      9 => SpamSignalType::RESERVED_VALUE_9,
      10 => SpamSignalType::RESERVED_VALUE_10,
      _ => SpamSignalType(i)
    }
  }
}

impl From<&i32> for SpamSignalType {
  fn from(i: &i32) -> Self {
    SpamSignalType::from(*i)
  }
}

impl From<SpamSignalType> for i32 {
  fn from(e: SpamSignalType) -> i32 {
    e.0
  }
}

impl From<&SpamSignalType> for i32 {
  fn from(e: &SpamSignalType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetCreateContextKey(pub i32);

impl TweetCreateContextKey {
  pub const PERISCOPE_IS_LIVE: TweetCreateContextKey = TweetCreateContextKey(0);
  pub const PERISCOPE_CREATOR_ID: TweetCreateContextKey = TweetCreateContextKey(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PERISCOPE_IS_LIVE,
    Self::PERISCOPE_CREATOR_ID,
  ];
}

impl TSerializable for TweetCreateContextKey {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetCreateContextKey> {
    let enum_value = i_prot.read_i32()?;
    Ok(TweetCreateContextKey::from(enum_value))
  }
}

impl From<i32> for TweetCreateContextKey {
  fn from(i: i32) -> Self {
    match i {
      0 => TweetCreateContextKey::PERISCOPE_IS_LIVE,
      1 => TweetCreateContextKey::PERISCOPE_CREATOR_ID,
      _ => TweetCreateContextKey(i)
    }
  }
}

impl From<&i32> for TweetCreateContextKey {
  fn from(i: &i32) -> Self {
    TweetCreateContextKey::from(*i)
  }
}

impl From<TweetCreateContextKey> for i32 {
  fn from(e: TweetCreateContextKey) -> i32 {
    e.0
  }
}

impl From<&TweetCreateContextKey> for i32 {
  fn from(e: &TweetCreateContextKey) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComposerSource(pub i32);

impl ComposerSource {
  pub const STANDARD: ComposerSource = ComposerSource(1);
  pub const CAMERA: ComposerSource = ComposerSource(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::STANDARD,
    Self::CAMERA,
  ];
}

impl TSerializable for ComposerSource {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ComposerSource> {
    let enum_value = i_prot.read_i32()?;
    Ok(ComposerSource::from(enum_value))
  }
}

impl From<i32> for ComposerSource {
  fn from(i: i32) -> Self {
    match i {
      1 => ComposerSource::STANDARD,
      2 => ComposerSource::CAMERA,
      _ => ComposerSource(i)
    }
  }
}

impl From<&i32> for ComposerSource {
  fn from(i: &i32) -> Self {
    ComposerSource::from(*i)
  }
}

impl From<ComposerSource> for i32 {
  fn from(e: ComposerSource) -> i32 {
    e.0
  }
}

impl From<&ComposerSource> for i32 {
  fn from(e: &ComposerSource) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CollabInvitationStatus(pub i32);

impl CollabInvitationStatus {
  pub const PENDING: CollabInvitationStatus = CollabInvitationStatus(0);
  pub const ACCEPTED: CollabInvitationStatus = CollabInvitationStatus(1);
  pub const REJECTED: CollabInvitationStatus = CollabInvitationStatus(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PENDING,
    Self::ACCEPTED,
    Self::REJECTED,
  ];
}

impl TSerializable for CollabInvitationStatus {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CollabInvitationStatus> {
    let enum_value = i_prot.read_i32()?;
    Ok(CollabInvitationStatus::from(enum_value))
  }
}

impl From<i32> for CollabInvitationStatus {
  fn from(i: i32) -> Self {
    match i {
      0 => CollabInvitationStatus::PENDING,
      1 => CollabInvitationStatus::ACCEPTED,
      2 => CollabInvitationStatus::REJECTED,
      _ => CollabInvitationStatus(i)
    }
  }
}

impl From<&i32> for CollabInvitationStatus {
  fn from(i: &i32) -> Self {
    CollabInvitationStatus::from(*i)
  }
}

impl From<CollabInvitationStatus> for i32 {
  fn from(e: CollabInvitationStatus) -> i32 {
    e.0
  }
}

impl From<&CollabInvitationStatus> for i32 {
  fn from(e: &CollabInvitationStatus) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DisallowedReply(pub i32);

impl DisallowedReply {
  pub const LINKS: DisallowedReply = DisallowedReply(0);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::LINKS,
  ];
}

impl TSerializable for DisallowedReply {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DisallowedReply> {
    let enum_value = i_prot.read_i32()?;
    Ok(DisallowedReply::from(enum_value))
  }
}

impl From<i32> for DisallowedReply {
  fn from(i: i32) -> Self {
    match i {
      0 => DisallowedReply::LINKS,
      _ => DisallowedReply(i)
    }
  }
}

impl From<&i32> for DisallowedReply {
  fn from(i: &i32) -> Self {
    DisallowedReply::from(*i)
  }
}

impl From<DisallowedReply> for i32 {
  fn from(e: DisallowedReply) -> i32 {
    e.0
  }
}

impl From<&DisallowedReply> for i32 {
  fn from(e: &DisallowedReply) -> i32 {
    e.0
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Reply {
            pub in_reply_to_status_id: Option<i64>,
            pub in_reply_to_user_id: Option<i64>,
}

impl Reply {
  pub fn new<F1, F2>(in_reply_to_status_id: F1, in_reply_to_user_id: F2) -> Reply where F1: Into<Option<i64>>, F2: Into<Option<i64>> {
    Reply {
      in_reply_to_status_id: in_reply_to_status_id.into(),
      in_reply_to_user_id: in_reply_to_user_id.into(),
    }
  }
}

impl TSerializable for Reply {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Reply> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = Some(0);
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Reply {
      in_reply_to_status_id: f_1,
      in_reply_to_user_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Reply");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.in_reply_to_status_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("in_reply_to_status_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.in_reply_to_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("in_reply_to_user_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DirectedAtUser {
  pub user_id: Option<i64>,
  pub screen_name: Option<String>,
}

impl DirectedAtUser {
  pub fn new<F1, F2>(user_id: F1, screen_name: F2) -> DirectedAtUser where F1: Into<Option<i64>>, F2: Into<Option<String>> {
    DirectedAtUser {
      user_id: user_id.into(),
      screen_name: screen_name.into(),
    }
  }
}

impl TSerializable for DirectedAtUser {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DirectedAtUser> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<String> = Some("".to_owned());
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
    let ret = DirectedAtUser {
      user_id: f_1,
      screen_name: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DirectedAtUser");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.screen_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("screen_name", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Share {
        pub source_status_id: Option<i64>,
  pub source_user_id: Option<i64>,
                pub parent_status_id: Option<i64>,
}

impl Share {
  pub fn new<F1, F2, F3>(source_status_id: F1, source_user_id: F2, parent_status_id: F3) -> Share where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>> {
    Share {
      source_status_id: source_status_id.into(),
      source_user_id: source_user_id.into(),
      parent_status_id: parent_status_id.into(),
    }
  }
}

impl TSerializable for Share {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Share> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Share {
      source_status_id: f_1,
      source_user_id: f_2,
      parent_status_id: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Share");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.source_status_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_status_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.source_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_user_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.parent_status_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("parent_status_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShortenedUrl {
    pub short_url: Option<String>,
    pub long_url: Option<String>,
      pub display_text: Option<String>,
}

impl ShortenedUrl {
  pub fn new<F1, F2, F3>(short_url: F1, long_url: F2, display_text: F3) -> ShortenedUrl where F1: Into<Option<String>>, F2: Into<Option<String>>, F3: Into<Option<String>> {
    ShortenedUrl {
      short_url: short_url.into(),
      long_url: long_url.into(),
      display_text: display_text.into(),
    }
  }
}

impl TSerializable for ShortenedUrl {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ShortenedUrl> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<String> = Some("".to_owned());
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
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ShortenedUrl {
      short_url: f_1,
      long_url: f_2,
      display_text: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ShortenedUrl");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.short_url {
      o_prot.write_field_begin(&TFieldIdentifier::new("short_url", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.long_url {
      o_prot.write_field_begin(&TFieldIdentifier::new("long_url", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.display_text {
      o_prot.write_field_begin(&TFieldIdentifier::new("display_text", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QuotedTweet {
  pub tweet_id: Option<i64>,
  pub user_id: Option<i64>,
}

impl QuotedTweet {
  pub fn new<F1, F2>(tweet_id: F1, user_id: F2) -> QuotedTweet where F1: Into<Option<i64>>, F2: Into<Option<i64>> {
    QuotedTweet {
      tweet_id: tweet_id.into(),
      user_id: user_id.into(),
    }
  }
}

impl TSerializable for QuotedTweet {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<QuotedTweet> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = QuotedTweet {
      tweet_id: f_1,
      user_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("QuotedTweet");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Contributor {
  pub user_id: Option<i64>,
  pub screen_name: Option<String>,
}

impl Contributor {
  pub fn new<F1, F2>(user_id: F1, screen_name: F2) -> Contributor where F1: Into<Option<i64>>, F2: Into<Option<String>> {
    Contributor {
      user_id: user_id.into(),
      screen_name: screen_name.into(),
    }
  }
}

impl TSerializable for Contributor {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Contributor> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<String> = None;
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
    let ret = Contributor {
      user_id: f_1,
      screen_name: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Contributor");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.screen_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("screen_name", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GeoCoordinates {
  pub latitude: Option<OrderedFloat<f64>>,
  pub longitude: Option<OrderedFloat<f64>>,
  pub geo_precision: Option<i32>,
            pub display: Option<bool>,
}

impl GeoCoordinates {
  pub fn new<F1, F2, F3, F4>(latitude: F1, longitude: F2, geo_precision: F3, display: F4) -> GeoCoordinates where F1: Into<Option<OrderedFloat<f64>>>, F2: Into<Option<OrderedFloat<f64>>>, F3: Into<Option<i32>>, F4: Into<Option<bool>> {
    GeoCoordinates {
      latitude: latitude.into(),
      longitude: longitude.into(),
      geo_precision: geo_precision.into(),
      display: display.into(),
    }
  }
}

impl TSerializable for GeoCoordinates {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<GeoCoordinates> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<OrderedFloat<f64>> = Some(OrderedFloat::from(0.0));
    let mut f_2: Option<OrderedFloat<f64>> = Some(OrderedFloat::from(0.0));
    let mut f_3: Option<i32> = Some(0);
    let mut f_4: Option<bool> = Some(false);
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = OrderedFloat::from(i_prot.read_double()?);
          f_1 = Some(val);
        },
        2 => {
          let val = OrderedFloat::from(i_prot.read_double()?);
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i32()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_bool()?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = GeoCoordinates {
      latitude: f_1,
      longitude: f_2,
      geo_precision: f_3,
      display: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("GeoCoordinates");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.latitude {
      o_prot.write_field_begin(&TFieldIdentifier::new("latitude", TType::Double, 1))?;
      o_prot.write_double(fld_var.into())?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.longitude {
      o_prot.write_field_begin(&TFieldIdentifier::new("longitude", TType::Double, 2))?;
      o_prot.write_double(fld_var.into())?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.geo_precision {
      o_prot.write_field_begin(&TFieldIdentifier::new("geo_precision", TType::I32, 3))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.display {
      o_prot.write_field_begin(&TFieldIdentifier::new("display", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlaceName {
  pub name: Option<String>,
  pub language: Option<String>,
  pub type_: Option<PlaceNameType>,
  pub preferred: Option<bool>,
}

impl PlaceName {
  pub fn new<F1, F2, F3, F4>(name: F1, language: F2, type_: F3, preferred: F4) -> PlaceName where F1: Into<Option<String>>, F2: Into<Option<String>>, F3: Into<Option<PlaceNameType>>, F4: Into<Option<bool>> {
    PlaceName {
      name: name.into(),
      language: language.into(),
      type_: type_.into(),
      preferred: preferred.into(),
    }
  }
}

impl TSerializable for PlaceName {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PlaceName> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<PlaceNameType> = None;
    let mut f_4: Option<bool> = Some(false);
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
          let val = PlaceNameType::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_bool()?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = PlaceName {
      name: f_1,
      language: f_2,
      type_: f_3,
      preferred: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("PlaceName");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.language {
      o_prot.write_field_begin(&TFieldIdentifier::new("language", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.type_ {
      o_prot.write_field_begin(&TFieldIdentifier::new("type", TType::I32, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.preferred {
      o_prot.write_field_begin(&TFieldIdentifier::new("preferred", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NarrowcastPlace {
        pub id: Option<String>,
}

impl NarrowcastPlace {
  pub fn new<F1>(id: F1) -> NarrowcastPlace where F1: Into<Option<String>> {
    NarrowcastPlace {
      id: id.into(),
    }
  }
}

impl TSerializable for NarrowcastPlace {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<NarrowcastPlace> {
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
    let ret = NarrowcastPlace {
      id: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("NarrowcastPlace");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdvertisingDisclosure {
    pub is_paid_promotion: Option<bool>,
}

impl AdvertisingDisclosure {
  pub fn new<F1>(is_paid_promotion: F1) -> AdvertisingDisclosure where F1: Into<Option<bool>> {
    AdvertisingDisclosure {
      is_paid_promotion: is_paid_promotion.into(),
    }
  }
}

impl TSerializable for AdvertisingDisclosure {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AdvertisingDisclosure> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = Some(false);
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_bool()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = AdvertisingDisclosure {
      is_paid_promotion: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AdvertisingDisclosure");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.is_paid_promotion {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_paid_promotion", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContentDisclosure {
    pub advertising_disclosure: Option<AdvertisingDisclosure>,
}

impl ContentDisclosure {
  pub fn new<F1>(advertising_disclosure: F1) -> ContentDisclosure where F1: Into<Option<AdvertisingDisclosure>> {
    ContentDisclosure {
      advertising_disclosure: advertising_disclosure.into(),
    }
  }
}

impl TSerializable for ContentDisclosure {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ContentDisclosure> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<AdvertisingDisclosure> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = AdvertisingDisclosure::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ContentDisclosure {
      advertising_disclosure: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ContentDisclosure");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.advertising_disclosure {
      o_prot.write_field_begin(&TFieldIdentifier::new("advertising_disclosure", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Place {
        pub id: Option<String>,
    pub type_: Option<PlaceType>,
            pub full_name: Option<String>,
              pub name: Option<String>,
    pub attributes: Option<BTreeMap<String, String>>,
  pub names: Option<BTreeSet<PlaceName>>,
    pub country_code: Option<String>,
            pub country_name: Option<String>,
    pub bounding_box: Option<Vec<GeoCoordinates>>,
      pub containers: Option<BTreeSet<String>>,
    pub centroid: Option<GeoCoordinates>,
}

impl Place {
  pub fn new<F1, F2, F3, F4, F5, F7, F9, F10, F11, F12, F13>(id: F1, type_: F2, full_name: F3, name: F4, attributes: F5, names: F7, country_code: F9, country_name: F10, bounding_box: F11, containers: F12, centroid: F13) -> Place where F1: Into<Option<String>>, F2: Into<Option<PlaceType>>, F3: Into<Option<String>>, F4: Into<Option<String>>, F5: Into<Option<BTreeMap<String, String>>>, F7: Into<Option<BTreeSet<PlaceName>>>, F9: Into<Option<String>>, F10: Into<Option<String>>, F11: Into<Option<Vec<GeoCoordinates>>>, F12: Into<Option<BTreeSet<String>>>, F13: Into<Option<GeoCoordinates>> {
    Place {
      id: id.into(),
      type_: type_.into(),
      full_name: full_name.into(),
      name: name.into(),
      attributes: attributes.into(),
      names: names.into(),
      country_code: country_code.into(),
      country_name: country_name.into(),
      bounding_box: bounding_box.into(),
      containers: containers.into(),
      centroid: centroid.into(),
    }
  }
}

impl TSerializable for Place {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Place> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<PlaceType> = None;
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<String> = Some("".to_owned());
    let mut f_5: Option<BTreeMap<String, String>> = Some(BTreeMap::new());
    let mut f_7: Option<BTreeSet<PlaceName>> = Some(BTreeSet::new());
    let mut f_9: Option<String> = None;
    let mut f_10: Option<String> = None;
    let mut f_11: Option<Vec<GeoCoordinates>> = None;
    let mut f_12: Option<BTreeSet<String>> = None;
    let mut f_13: Option<GeoCoordinates> = None;
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
          let val = PlaceType::read_from_in_protocol(i_prot)?;
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
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, String> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_0 = i_prot.read_string()?;
            let map_val_1 = i_prot.read_string()?;
            val.insert(map_key_0, map_val_1);
          }
          i_prot.read_map_end()?;
          f_5 = Some(val);
        },
        7 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<PlaceName> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_2 = PlaceName::read_from_in_protocol(i_prot)?;
            val.insert(set_elem_2);
          }
          i_prot.read_set_end()?;
          f_7 = Some(val);
        },
        9 => {
          let val = i_prot.read_string()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_string()?;
          f_10 = Some(val);
        },
        11 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<GeoCoordinates> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_3 = GeoCoordinates::read_from_in_protocol(i_prot)?;
            val.push(list_elem_3);
          }
          i_prot.read_list_end()?;
          f_11 = Some(val);
        },
        12 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<String> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_4 = i_prot.read_string()?;
            val.insert(set_elem_4);
          }
          i_prot.read_set_end()?;
          f_12 = Some(val);
        },
        13 => {
          let val = GeoCoordinates::read_from_in_protocol(i_prot)?;
          f_13 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Place {
      id: f_1,
      type_: f_2,
      full_name: f_3,
      name: f_4,
      attributes: f_5,
      names: f_7,
      country_code: f_9,
      country_name: f_10,
      bounding_box: f_11,
      containers: f_12,
      centroid: f_13,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Place");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.type_ {
      o_prot.write_field_begin(&TFieldIdentifier::new("type", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.full_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("full_name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 4))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.attributes {
      o_prot.write_field_begin(&TFieldIdentifier::new("attributes", TType::Map, 5))?;
      o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::String, fld_var.len() as i32))?;
      for (k, v) in fld_var {
        o_prot.write_string(k)?;
        o_prot.write_string(v)?;
      }
      o_prot.write_map_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.names {
      o_prot.write_field_begin(&TFieldIdentifier::new("names", TType::Set, 7))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.country_code {
      o_prot.write_field_begin(&TFieldIdentifier::new("country_code", TType::String, 9))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.country_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("country_name", TType::String, 10))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.bounding_box {
      o_prot.write_field_begin(&TFieldIdentifier::new("bounding_box", TType::List, 11))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.containers {
      o_prot.write_field_begin(&TFieldIdentifier::new("containers", TType::Set, 12))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::String, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_string(e)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.centroid {
      o_prot.write_field_begin(&TFieldIdentifier::new("centroid", TType::Struct, 13))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UrlEntity {
      pub from_index: Option<i16>,
      pub to_index: Option<i16>,
    pub url: Option<String>,
          pub expanded: Option<String>,
            pub display: Option<String>,
  pub click_count: Option<i64>,
}

impl UrlEntity {
  pub fn new<F1, F2, F3, F4, F5, F6>(from_index: F1, to_index: F2, url: F3, expanded: F4, display: F5, click_count: F6) -> UrlEntity where F1: Into<Option<i16>>, F2: Into<Option<i16>>, F3: Into<Option<String>>, F4: Into<Option<String>>, F5: Into<Option<String>>, F6: Into<Option<i64>> {
    UrlEntity {
      from_index: from_index.into(),
      to_index: to_index.into(),
      url: url.into(),
      expanded: expanded.into(),
      display: display.into(),
      click_count: click_count.into(),
    }
  }
}

impl TSerializable for UrlEntity {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UrlEntity> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i16> = Some(0);
    let mut f_2: Option<i16> = Some(0);
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<String> = None;
    let mut f_5: Option<String> = None;
    let mut f_6: Option<i64> = None;
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
          let val = i_prot.read_i64()?;
          f_6 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = UrlEntity {
      from_index: f_1,
      to_index: f_2,
      url: f_3,
      expanded: f_4,
      display: f_5,
      click_count: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UrlEntity");
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
    if let Some(ref fld_var) = self.expanded {
      o_prot.write_field_begin(&TFieldIdentifier::new("expanded", TType::String, 4))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.display {
      o_prot.write_field_begin(&TFieldIdentifier::new("display", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.click_count {
      o_prot.write_field_begin(&TFieldIdentifier::new("click_count", TType::I64, 6))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MentionEntity {
      pub from_index: Option<i16>,
      pub to_index: Option<i16>,
    pub screen_name: Option<String>,
                      pub user_id: Option<i64>,
            pub name: Option<String>,
            pub is_unmentioned: Option<bool>,
}

impl MentionEntity {
  pub fn new<F1, F2, F3, F4, F5, F6>(from_index: F1, to_index: F2, screen_name: F3, user_id: F4, name: F5, is_unmentioned: F6) -> MentionEntity where F1: Into<Option<i16>>, F2: Into<Option<i16>>, F3: Into<Option<String>>, F4: Into<Option<i64>>, F5: Into<Option<String>>, F6: Into<Option<bool>> {
    MentionEntity {
      from_index: from_index.into(),
      to_index: to_index.into(),
      screen_name: screen_name.into(),
      user_id: user_id.into(),
      name: name.into(),
      is_unmentioned: is_unmentioned.into(),
    }
  }
}

impl TSerializable for MentionEntity {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MentionEntity> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i16> = Some(0);
    let mut f_2: Option<i16> = Some(0);
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<i64> = None;
    let mut f_5: Option<String> = None;
    let mut f_6: Option<bool> = None;
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
          let val = i_prot.read_i64()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_string()?;
          f_5 = Some(val);
        },
        6 => {
          let val = i_prot.read_bool()?;
          f_6 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = MentionEntity {
      from_index: f_1,
      to_index: f_2,
      screen_name: f_3,
      user_id: f_4,
      name: f_5,
      is_unmentioned: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MentionEntity");
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
    if let Some(ref fld_var) = self.screen_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("screen_name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_unmentioned {
      o_prot.write_field_begin(&TFieldIdentifier::new("isUnmentioned", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlockingUnmentions {
  pub unmentioned_user_ids: Option<Vec<i64>>,
}

impl BlockingUnmentions {
  pub fn new<F1>(unmentioned_user_ids: F1) -> BlockingUnmentions where F1: Into<Option<Vec<i64>>> {
    BlockingUnmentions {
      unmentioned_user_ids: unmentioned_user_ids.into(),
    }
  }
}

impl TSerializable for BlockingUnmentions {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<BlockingUnmentions> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_5 = i_prot.read_i64()?;
            val.push(list_elem_5);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = BlockingUnmentions {
      unmentioned_user_ids: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("BlockingUnmentions");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.unmentioned_user_ids {
      o_prot.write_field_begin(&TFieldIdentifier::new("unmentioned_user_ids", TType::List, 1))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::I64, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_i64(*e)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SettingsUnmentions {
  pub unmentioned_user_ids: Option<Vec<i64>>,
}

impl SettingsUnmentions {
  pub fn new<F1>(unmentioned_user_ids: F1) -> SettingsUnmentions where F1: Into<Option<Vec<i64>>> {
    SettingsUnmentions {
      unmentioned_user_ids: unmentioned_user_ids.into(),
    }
  }
}

impl TSerializable for SettingsUnmentions {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SettingsUnmentions> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_6 = i_prot.read_i64()?;
            val.push(list_elem_6);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = SettingsUnmentions {
      unmentioned_user_ids: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SettingsUnmentions");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.unmentioned_user_ids {
      o_prot.write_field_begin(&TFieldIdentifier::new("unmentioned_user_ids", TType::List, 1))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::I64, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_i64(*e)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HashtagEntity {
      pub from_index: Option<i16>,
      pub to_index: Option<i16>,
    pub text: Option<String>,
}

impl HashtagEntity {
  pub fn new<F1, F2, F3>(from_index: F1, to_index: F2, text: F3) -> HashtagEntity where F1: Into<Option<i16>>, F2: Into<Option<i16>>, F3: Into<Option<String>> {
    HashtagEntity {
      from_index: from_index.into(),
      to_index: to_index.into(),
      text: text.into(),
    }
  }
}

impl TSerializable for HashtagEntity {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<HashtagEntity> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i16> = Some(0);
    let mut f_2: Option<i16> = Some(0);
    let mut f_3: Option<String> = Some("".to_owned());
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = HashtagEntity {
      from_index: f_1,
      to_index: f_2,
      text: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("HashtagEntity");
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
    if let Some(ref fld_var) = self.text {
      o_prot.write_field_begin(&TFieldIdentifier::new("text", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CashtagEntity {
      pub from_index: Option<i16>,
      pub to_index: Option<i16>,
    pub text: Option<String>,
}

impl CashtagEntity {
  pub fn new<F1, F2, F3>(from_index: F1, to_index: F2, text: F3) -> CashtagEntity where F1: Into<Option<i16>>, F2: Into<Option<i16>>, F3: Into<Option<String>> {
    CashtagEntity {
      from_index: from_index.into(),
      to_index: to_index.into(),
      text: text.into(),
    }
  }
}

impl TSerializable for CashtagEntity {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CashtagEntity> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i16> = Some(0);
    let mut f_2: Option<i16> = Some(0);
    let mut f_3: Option<String> = Some("".to_owned());
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = CashtagEntity {
      from_index: f_1,
      to_index: f_2,
      text: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("CashtagEntity");
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
    if let Some(ref fld_var) = self.text {
      o_prot.write_field_begin(&TFieldIdentifier::new("text", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TimestampEntity {
      pub from_index: Option<i16>,
      pub to_index: Option<i16>,
    pub text: Option<String>,
    pub seconds: Option<i32>,
}

impl TimestampEntity {
  pub fn new<F1, F2, F3, F4>(from_index: F1, to_index: F2, text: F3, seconds: F4) -> TimestampEntity where F1: Into<Option<i16>>, F2: Into<Option<i16>>, F3: Into<Option<String>>, F4: Into<Option<i32>> {
    TimestampEntity {
      from_index: from_index.into(),
      to_index: to_index.into(),
      text: text.into(),
      seconds: seconds.into(),
    }
  }
}

impl TSerializable for TimestampEntity {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TimestampEntity> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i16> = Some(0);
    let mut f_2: Option<i16> = Some(0);
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<i32> = None;
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
          let val = i_prot.read_i32()?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TimestampEntity {
      from_index: f_1,
      to_index: f_2,
      text: f_3,
      seconds: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TimestampEntity");
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
    if let Some(ref fld_var) = self.text {
      o_prot.write_field_begin(&TFieldIdentifier::new("text", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.seconds {
      o_prot.write_field_begin(&TFieldIdentifier::new("seconds", TType::I32, 4))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediaTag {
  pub tag_type: Option<MediaTagType>,
  pub user_id: Option<i64>,
  pub screen_name: Option<String>,
  pub name: Option<String>,
}

impl MediaTag {
  pub fn new<F1, F2, F3, F4>(tag_type: F1, user_id: F2, screen_name: F3, name: F4) -> MediaTag where F1: Into<Option<MediaTagType>>, F2: Into<Option<i64>>, F3: Into<Option<String>>, F4: Into<Option<String>> {
    MediaTag {
      tag_type: tag_type.into(),
      user_id: user_id.into(),
      screen_name: screen_name.into(),
      name: name.into(),
    }
  }
}

impl TSerializable for MediaTag {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaTag> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<MediaTagType> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<String> = None;
    let mut f_4: Option<String> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = MediaTagType::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = MediaTag {
      tag_type: f_1,
      user_id: f_2,
      screen_name: f_3,
      name: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MediaTag");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.tag_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("tag_type", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.screen_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("screen_name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 4))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetMediaTags {
}

impl TweetMediaTags {
  pub fn new() -> TweetMediaTags {
    TweetMediaTags {}
  }
}

impl TSerializable for TweetMediaTags {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetMediaTags> {
    i_prot.read_struct_begin()?;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      i_prot.skip(field_ident.field_type)?;
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TweetMediaTags {};
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetMediaTags");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserMention {
  pub user_id: Option<i64>,
  pub screen_name: Option<String>,
  pub name: Option<String>,
}

impl UserMention {
  pub fn new<F1, F2, F3>(user_id: F1, screen_name: F2, name: F3) -> UserMention where F1: Into<Option<i64>>, F2: Into<Option<String>>, F3: Into<Option<String>> {
    UserMention {
      user_id: user_id.into(),
      screen_name: screen_name.into(),
      name: name.into(),
    }
  }
}

impl TSerializable for UserMention {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UserMention> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<String> = None;
    let mut f_3: Option<String> = None;
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
          let val = i_prot.read_string()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = UserMention {
      user_id: f_1,
      screen_name: f_2,
      name: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UserMention");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.screen_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("screen_name", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReplyAddresses {
  pub users: Option<Vec<UserMention>>,
}

impl ReplyAddresses {
  pub fn new<F1>(users: F1) -> ReplyAddresses where F1: Into<Option<Vec<UserMention>>> {
    ReplyAddresses {
      users: users.into(),
    }
  }
}

impl TSerializable for ReplyAddresses {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ReplyAddresses> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<UserMention>> = Some(Vec::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<UserMention> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_7 = UserMention::read_from_in_protocol(i_prot)?;
            val.push(list_elem_7);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ReplyAddresses {
      users: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ReplyAddresses");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.users {
      o_prot.write_field_begin(&TFieldIdentifier::new("users", TType::List, 1))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchedulingInfo {
      pub scheduled_tweet_id: Option<i64>,
}

impl SchedulingInfo {
  pub fn new<F1>(scheduled_tweet_id: F1) -> SchedulingInfo where F1: Into<Option<i64>> {
    SchedulingInfo {
      scheduled_tweet_id: scheduled_tweet_id.into(),
    }
  }
}

impl TSerializable for SchedulingInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SchedulingInfo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = SchedulingInfo {
      scheduled_tweet_id: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SchedulingInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.scheduled_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("scheduled_tweet_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TwitterSuggestInfo {
  pub suggest_type: Option<SuggestType>,
  pub visibility_type: Option<TwitterSuggestsVisibilityType>,
  pub personalized_for_user_id: Option<i64>,
  pub display_timestamp_secs: Option<i64>,
}

impl TwitterSuggestInfo {
  pub fn new<F1, F2, F3, F4>(suggest_type: F1, visibility_type: F2, personalized_for_user_id: F3, display_timestamp_secs: F4) -> TwitterSuggestInfo where F1: Into<Option<SuggestType>>, F2: Into<Option<TwitterSuggestsVisibilityType>>, F3: Into<Option<i64>>, F4: Into<Option<i64>> {
    TwitterSuggestInfo {
      suggest_type: suggest_type.into(),
      visibility_type: visibility_type.into(),
      personalized_for_user_id: personalized_for_user_id.into(),
      display_timestamp_secs: display_timestamp_secs.into(),
    }
  }
}

impl TSerializable for TwitterSuggestInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TwitterSuggestInfo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<SuggestType> = None;
    let mut f_2: Option<TwitterSuggestsVisibilityType> = None;
    let mut f_3: Option<i64> = None;
    let mut f_4: Option<i64> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = SuggestType::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = TwitterSuggestsVisibilityType::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i64()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_i64()?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TwitterSuggestInfo {
      suggest_type: f_1,
      visibility_type: f_2,
      personalized_for_user_id: f_3,
      display_timestamp_secs: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TwitterSuggestInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.suggest_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("suggest_type", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.visibility_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("visibility_type", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.personalized_for_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("personalized_for_user_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.display_timestamp_secs {
      o_prot.write_field_begin(&TFieldIdentifier::new("display_timestamp_secs", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceSource {
            pub id: i64,
    pub parameter: Option<String>,
    pub internal_name: Option<String>,
    pub name: Option<String>,
      pub url: Option<String>,
        pub display: Option<String>,
      pub client_app_id: Option<i64>,
}

impl DeviceSource {
  pub fn new<F2, F3, F4, F5, F6, F7>(id: i64, parameter: F2, internal_name: F3, name: F4, url: F5, display: F6, client_app_id: F7) -> DeviceSource where F2: Into<Option<String>>, F3: Into<Option<String>>, F4: Into<Option<String>>, F5: Into<Option<String>>, F6: Into<Option<String>>, F7: Into<Option<i64>> {
    DeviceSource {
      id,
      parameter: parameter.into(),
      internal_name: internal_name.into(),
      name: name.into(),
      url: url.into(),
      display: display.into(),
      client_app_id: client_app_id.into(),
    }
  }
}

impl TSerializable for DeviceSource {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DeviceSource> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<String> = Some("".to_owned());
    let mut f_5: Option<String> = Some("".to_owned());
    let mut f_6: Option<String> = Some("".to_owned());
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
          let val = i_prot.read_string()?;
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
    verify_required_field_exists("DeviceSource.id", &f_1)?;
    let ret = DeviceSource {
      id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      parameter: f_2,
      internal_name: f_3,
      name: f_4,
      url: f_5,
      display: f_6,
      client_app_id: f_7,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DeviceSource");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
    o_prot.write_i64(self.id)?;
    o_prot.write_field_end()?;
    if let Some(ref fld_var) = self.parameter {
      o_prot.write_field_begin(&TFieldIdentifier::new("parameter", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.internal_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("internal_name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 4))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.url {
      o_prot.write_field_begin(&TFieldIdentifier::new("url", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.display {
      o_prot.write_field_begin(&TFieldIdentifier::new("display", TType::String, 6))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.client_app_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("client_app_id", TType::I64, 7))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Narrowcast {
  pub location: Option<Vec<String>>,
}

impl Narrowcast {
  pub fn new<F2>(location: F2) -> Narrowcast where F2: Into<Option<Vec<String>>> {
    Narrowcast {
      location: location.into(),
    }
  }
}

impl TSerializable for Narrowcast {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Narrowcast> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<Vec<String>> = Some(Vec::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<String> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_8 = i_prot.read_string()?;
            val.push(list_elem_8);
          }
          i_prot.read_list_end()?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Narrowcast {
      location: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Narrowcast");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.location {
      o_prot.write_field_begin(&TFieldIdentifier::new("location", TType::List, 2))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::String, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_string(e)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StatusCounts {
          pub retweet_count: Option<i64>,
        pub reply_count: Option<i64>,
            pub favorite_count: Option<i64>,
    pub unique_users_impressed_count: Option<i64>,
          pub descendent_reply_count: Option<i64>,
          pub quote_count: Option<i64>,
    pub bookmark_count: Option<i64>,
}

impl StatusCounts {
  pub fn new<F1, F2, F3, F4, F5, F6, F7>(retweet_count: F1, reply_count: F2, favorite_count: F3, unique_users_impressed_count: F4, descendent_reply_count: F5, quote_count: F6, bookmark_count: F7) -> StatusCounts where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<i64>>, F6: Into<Option<i64>>, F7: Into<Option<i64>> {
    StatusCounts {
      retweet_count: retweet_count.into(),
      reply_count: reply_count.into(),
      favorite_count: favorite_count.into(),
      unique_users_impressed_count: unique_users_impressed_count.into(),
      descendent_reply_count: descendent_reply_count.into(),
      quote_count: quote_count.into(),
      bookmark_count: bookmark_count.into(),
    }
  }
}

impl TSerializable for StatusCounts {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<StatusCounts> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
    let mut f_4: Option<i64> = None;
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<i64> = None;
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
          let val = i_prot.read_i64()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_i64()?;
          f_5 = Some(val);
        },
        6 => {
          let val = i_prot.read_i64()?;
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
    let ret = StatusCounts {
      retweet_count: f_1,
      reply_count: f_2,
      favorite_count: f_3,
      unique_users_impressed_count: f_4,
      descendent_reply_count: f_5,
      quote_count: f_6,
      bookmark_count: f_7,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("StatusCounts");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.retweet_count {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweet_count", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.reply_count {
      o_prot.write_field_begin(&TFieldIdentifier::new("reply_count", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.favorite_count {
      o_prot.write_field_begin(&TFieldIdentifier::new("favorite_count", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.unique_users_impressed_count {
      o_prot.write_field_begin(&TFieldIdentifier::new("unique_users_impressed_count", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.descendent_reply_count {
      o_prot.write_field_begin(&TFieldIdentifier::new("descendent_reply_count", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quote_count {
      o_prot.write_field_begin(&TFieldIdentifier::new("quote_count", TType::I64, 6))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.bookmark_count {
      o_prot.write_field_begin(&TFieldIdentifier::new("bookmark_count", TType::I64, 7))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StatusPerspective {
  pub user_id: Option<i64>,
    pub favorited: Option<bool>,
    pub retweeted: Option<bool>,
    pub retweet_id: Option<i64>,
          pub reported: Option<bool>,
    pub bookmarked: Option<bool>,
    pub downvoted: Option<bool>,
}

impl StatusPerspective {
  pub fn new<F1, F2, F3, F4, F5, F6, F7>(user_id: F1, favorited: F2, retweeted: F3, retweet_id: F4, reported: F5, bookmarked: F6, downvoted: F7) -> StatusPerspective where F1: Into<Option<i64>>, F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<i64>>, F5: Into<Option<bool>>, F6: Into<Option<bool>>, F7: Into<Option<bool>> {
    StatusPerspective {
      user_id: user_id.into(),
      favorited: favorited.into(),
      retweeted: retweeted.into(),
      retweet_id: retweet_id.into(),
      reported: reported.into(),
      bookmarked: bookmarked.into(),
      downvoted: downvoted.into(),
    }
  }
}

impl TSerializable for StatusPerspective {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<StatusPerspective> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<bool> = Some(false);
    let mut f_3: Option<bool> = Some(false);
    let mut f_4: Option<i64> = None;
    let mut f_5: Option<bool> = Some(false);
    let mut f_6: Option<bool> = None;
    let mut f_7: Option<bool> = None;
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
          let val = i_prot.read_bool()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_i64()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_bool()?;
          f_5 = Some(val);
        },
        6 => {
          let val = i_prot.read_bool()?;
          f_6 = Some(val);
        },
        7 => {
          let val = i_prot.read_bool()?;
          f_7 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = StatusPerspective {
      user_id: f_1,
      favorited: f_2,
      retweeted: f_3,
      retweet_id: f_4,
      reported: f_5,
      bookmarked: f_6,
      downvoted: f_7,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("StatusPerspective");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.favorited {
      o_prot.write_field_begin(&TFieldIdentifier::new("favorited", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.retweeted {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweeted", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.retweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweet_id", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.reported {
      o_prot.write_field_begin(&TFieldIdentifier::new("reported", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.bookmarked {
      o_prot.write_field_begin(&TFieldIdentifier::new("bookmarked", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.downvoted {
      o_prot.write_field_begin(&TFieldIdentifier::new("downvoted", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Language {
    pub language: String,
    pub right_to_left: Option<bool>,
    pub confidence: Option<OrderedFloat<f64>>,
}

impl Language {
  pub fn new<F2, F3>(language: String, right_to_left: F2, confidence: F3) -> Language where F2: Into<Option<bool>>, F3: Into<Option<OrderedFloat<f64>>> {
    Language {
      language,
      right_to_left: right_to_left.into(),
      confidence: confidence.into(),
    }
  }
}

impl TSerializable for Language {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Language> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = None;
    let mut f_2: Option<bool> = Some(false);
    let mut f_3: Option<OrderedFloat<f64>> = Some(OrderedFloat::from(0.0));
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
          let val = i_prot.read_bool()?;
          f_2 = Some(val);
        },
        3 => {
          let val = OrderedFloat::from(i_prot.read_double()?);
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("Language.language", &f_1)?;
    let ret = Language {
      language: f_1.expect("auto-generated code should have checked for presence of required fields"),
      right_to_left: f_2,
      confidence: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Language");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("language", TType::String, 1))?;
    o_prot.write_string(&self.language)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.right_to_left {
      o_prot.write_field_begin(&TFieldIdentifier::new("right_to_left", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.confidence {
      o_prot.write_field_begin(&TFieldIdentifier::new("confidence", TType::Double, 3))?;
      o_prot.write_double(fld_var.into())?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SupplementalLanguage {
    pub language: String,
}

impl SupplementalLanguage {
  pub fn new(language: String) -> SupplementalLanguage {
    SupplementalLanguage {
      language,
    }
  }
}

impl TSerializable for SupplementalLanguage {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SupplementalLanguage> {
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
    verify_required_field_exists("SupplementalLanguage.language", &f_1)?;
    let ret = SupplementalLanguage {
      language: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SupplementalLanguage");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("language", TType::String, 1))?;
    o_prot.write_string(&self.language)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpamLabel {
        pub spam: Option<bool>,
}

impl SpamLabel {
  pub fn new<F1>(spam: F1) -> SpamLabel where F1: Into<Option<bool>> {
    SpamLabel {
      spam: spam.into(),
    }
  }
}

impl TSerializable for SpamLabel {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SpamLabel> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = Some(false);
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_bool()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = SpamLabel {
      spam: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SpamLabel");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.spam {
      o_prot.write_field_begin(&TFieldIdentifier::new("spam", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CardBindingValues {
}

impl CardBindingValues {
  pub fn new() -> CardBindingValues {
    CardBindingValues {}
  }
}

impl TSerializable for CardBindingValues {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CardBindingValues> {
    i_prot.read_struct_begin()?;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      i_prot.skip(field_ident.field_type)?;
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = CardBindingValues {};
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("CardBindingValues");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CardReference {
            pub card_uri: Option<String>,
}

impl CardReference {
  pub fn new<F1>(card_uri: F1) -> CardReference where F1: Into<Option<String>> {
    CardReference {
      card_uri: card_uri.into(),
    }
  }
}

impl TSerializable for CardReference {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CardReference> {
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
    let ret = CardReference {
      card_uri: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("CardReference");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.card_uri {
      o_prot.write_field_begin(&TFieldIdentifier::new("card_uri", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetPivot {
}

impl TweetPivot {
  pub fn new() -> TweetPivot {
    TweetPivot {}
  }
}

impl TSerializable for TweetPivot {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetPivot> {
    i_prot.read_struct_begin()?;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      i_prot.skip(field_ident.field_type)?;
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TweetPivot {};
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetPivot");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetPivots {
  pub tweet_pivots: Vec<TweetPivot>,
}

impl TweetPivots {
  pub fn new(tweet_pivots: Vec<TweetPivot>) -> TweetPivots {
    TweetPivots {
      tweet_pivots,
    }
  }
}

impl TSerializable for TweetPivots {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetPivots> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<TweetPivot>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<TweetPivot> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_9 = TweetPivot::read_from_in_protocol(i_prot)?;
            val.push(list_elem_9);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("TweetPivots.tweet_pivots", &f_1)?;
    let ret = TweetPivots {
      tweet_pivots: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetPivots");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("tweet_pivots", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, self.tweet_pivots.len() as i32))?;
    for e in &self.tweet_pivots {
      e.write_to_out_protocol(o_prot)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EscherbirdEntityAnnotations {
}

impl EscherbirdEntityAnnotations {
  pub fn new() -> EscherbirdEntityAnnotations {
    EscherbirdEntityAnnotations {}
  }
}

impl TSerializable for EscherbirdEntityAnnotations {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<EscherbirdEntityAnnotations> {
    i_prot.read_struct_begin()?;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      i_prot.skip(field_ident.field_type)?;
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = EscherbirdEntityAnnotations {};
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("EscherbirdEntityAnnotations");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TextRange {
      pub from_index: i32,
      pub to_index: i32,
}

impl TextRange {
  pub fn new(from_index: i32, to_index: i32) -> TextRange {
    TextRange {
      from_index,
      to_index,
    }
  }
}

impl TSerializable for TextRange {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TextRange> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i32> = None;
    let mut f_2: Option<i32> = None;
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
          let val = i_prot.read_i32()?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("TextRange.from_index", &f_1)?;
    verify_required_field_exists("TextRange.to_index", &f_2)?;
    let ret = TextRange {
      from_index: f_1.expect("auto-generated code should have checked for presence of required fields"),
      to_index: f_2.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TextRange");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("from_index", TType::I32, 1))?;
    o_prot.write_i32(self.from_index)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("to_index", TType::I32, 2))?;
    o_prot.write_i32(self.to_index)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetCoreData {
  pub user_id: Option<i64>,
                pub created_at_secs: Option<i64>,
    pub reply: Option<Reply>,
    pub share: Option<Share>,
                pub nullcast: Option<bool>,
          pub conversation_id: Option<i64>,
}

impl TweetCoreData {
  pub fn new<F1, F4, F5, F7, F11, F14>(user_id: F1, created_at_secs: F4, reply: F5, share: F7, nullcast: F11, conversation_id: F14) -> TweetCoreData where F1: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<Reply>>, F7: Into<Option<Share>>, F11: Into<Option<bool>>, F14: Into<Option<i64>> {
    TweetCoreData {
      user_id: user_id.into(),
      created_at_secs: created_at_secs.into(),
      reply: reply.into(),
      share: share.into(),
      nullcast: nullcast.into(),
      conversation_id: conversation_id.into(),
    }
  }
}

impl TSerializable for TweetCoreData {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetCoreData> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<Reply> = None;
    let mut f_7: Option<Share> = None;
    let mut f_11: Option<bool> = Some(false);
    let mut f_14: Option<i64> = None;
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
        4 => {
          let val = i_prot.read_i64()?;
          f_4 = Some(val);
        },
        5 => {
          let val = Reply::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        7 => {
          let val = Share::read_from_in_protocol(i_prot)?;
          f_7 = Some(val);
        },
        11 => {
          let val = i_prot.read_bool()?;
          f_11 = Some(val);
        },
        14 => {
          let val = i_prot.read_i64()?;
          f_14 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TweetCoreData {
      user_id: f_1,
      created_at_secs: f_4,
      reply: f_5,
      share: f_7,
      nullcast: f_11,
      conversation_id: f_14,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetCoreData");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_secs {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_secs", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.reply {
      o_prot.write_field_begin(&TFieldIdentifier::new("reply", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.share {
      o_prot.write_field_begin(&TFieldIdentifier::new("share", TType::Struct, 7))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.nullcast {
      o_prot.write_field_begin(&TFieldIdentifier::new("nullcast", TType::Bool, 11))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.conversation_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("conversation_id", TType::I64, 14))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PureCoreData {
  pub user_id: Option<i64>,
                          pub text: Option<String>,
        pub created_via: Option<String>,
                pub created_at_secs: Option<i64>,
    pub reply: Option<Reply>,
    pub share: Option<Share>,
  pub conversation_id: Option<i64>,
}

impl PureCoreData {
  pub fn new<F1, F2, F3, F4, F5, F7, F9>(user_id: F1, text: F2, created_via: F3, created_at_secs: F4, reply: F5, share: F7, conversation_id: F9) -> PureCoreData where F1: Into<Option<i64>>, F2: Into<Option<String>>, F3: Into<Option<String>>, F4: Into<Option<i64>>, F5: Into<Option<Reply>>, F7: Into<Option<Share>>, F9: Into<Option<i64>> {
    PureCoreData {
      user_id: user_id.into(),
      text: text.into(),
      created_via: created_via.into(),
      created_at_secs: created_at_secs.into(),
      reply: reply.into(),
      share: share.into(),
      conversation_id: conversation_id.into(),
    }
  }
}

impl TSerializable for PureCoreData {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PureCoreData> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<Reply> = None;
    let mut f_7: Option<Share> = None;
    let mut f_9: Option<i64> = None;
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
          let val = i_prot.read_string()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_i64()?;
          f_4 = Some(val);
        },
        5 => {
          let val = Reply::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        7 => {
          let val = Share::read_from_in_protocol(i_prot)?;
          f_7 = Some(val);
        },
        9 => {
          let val = i_prot.read_i64()?;
          f_9 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = PureCoreData {
      user_id: f_1,
      text: f_2,
      created_via: f_3,
      created_at_secs: f_4,
      reply: f_5,
      share: f_7,
      conversation_id: f_9,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("PureCoreData");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.text {
      o_prot.write_field_begin(&TFieldIdentifier::new("text", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.created_via {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_via", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_secs {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_secs", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.reply {
      o_prot.write_field_begin(&TFieldIdentifier::new("reply", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.share {
      o_prot.write_field_begin(&TFieldIdentifier::new("share", TType::Struct, 7))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.conversation_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("conversation_id", TType::I64, 9))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Communities {
  pub community_ids: Vec<i64>,
  pub community_post_channel_id: Option<String>,
}

impl Communities {
  pub fn new<F2>(community_ids: Vec<i64>, community_post_channel_id: F2) -> Communities where F2: Into<Option<String>> {
    Communities {
      community_ids,
      community_post_channel_id: community_post_channel_id.into(),
    }
  }
}

impl TSerializable for Communities {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Communities> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    let mut f_2: Option<String> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_10 = i_prot.read_i64()?;
            val.push(list_elem_10);
          }
          i_prot.read_list_end()?;
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
    verify_required_field_exists("Communities.community_ids", &f_1)?;
    let ret = Communities {
      community_ids: f_1.expect("auto-generated code should have checked for presence of required fields"),
      community_post_channel_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Communities");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("community_ids", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.community_ids.len() as i32))?;
    for e in &self.community_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    if let Some(ref fld_var) = self.community_post_channel_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("community_post_channel_id", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExtendedTweetMetadata {
    pub unused1: Option<i32>,
            pub api_compatible_truncation_index: i32,
    pub unused3: Option<i32>,
    pub unused4: Option<bool>,
    pub unused5: Option<TextRange>,
    pub unused6: Option<TextRange>,
}

impl ExtendedTweetMetadata {
  pub fn new<F1, F3, F4, F5, F6>(unused1: F1, api_compatible_truncation_index: i32, unused3: F3, unused4: F4, unused5: F5, unused6: F6) -> ExtendedTweetMetadata where F1: Into<Option<i32>>, F3: Into<Option<i32>>, F4: Into<Option<bool>>, F5: Into<Option<TextRange>>, F6: Into<Option<TextRange>> {
    ExtendedTweetMetadata {
      unused1: unused1.into(),
      api_compatible_truncation_index,
      unused3: unused3.into(),
      unused4: unused4.into(),
      unused5: unused5.into(),
      unused6: unused6.into(),
    }
  }
}

impl TSerializable for ExtendedTweetMetadata {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ExtendedTweetMetadata> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i32> = Some(0);
    let mut f_2: Option<i32> = None;
    let mut f_3: Option<i32> = Some(0);
    let mut f_4: Option<bool> = Some(false);
    let mut f_5: Option<TextRange> = None;
    let mut f_6: Option<TextRange> = None;
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
          let val = i_prot.read_i32()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i32()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_bool()?;
          f_4 = Some(val);
        },
        5 => {
          let val = TextRange::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        6 => {
          let val = TextRange::read_from_in_protocol(i_prot)?;
          f_6 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ExtendedTweetMetadata.api_compatible_truncation_index", &f_2)?;
    let ret = ExtendedTweetMetadata {
      unused1: f_1,
      api_compatible_truncation_index: f_2.expect("auto-generated code should have checked for presence of required fields"),
      unused3: f_3,
      unused4: f_4,
      unused5: f_5,
      unused6: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ExtendedTweetMetadata");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.unused1 {
      o_prot.write_field_begin(&TFieldIdentifier::new("unused1", TType::I32, 1))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_begin(&TFieldIdentifier::new("api_compatible_truncation_index", TType::I32, 2))?;
    o_prot.write_i32(self.api_compatible_truncation_index)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.unused3 {
      o_prot.write_field_begin(&TFieldIdentifier::new("unused3", TType::I32, 3))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.unused4 {
      o_prot.write_field_begin(&TFieldIdentifier::new("unused4", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.unused5 {
      o_prot.write_field_begin(&TFieldIdentifier::new("unused5", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.unused6 {
      o_prot.write_field_begin(&TFieldIdentifier::new("unused6", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DirectedAtUserMetadata {
    pub user_id: Option<i64>,
}

impl DirectedAtUserMetadata {
  pub fn new<F1>(user_id: F1) -> DirectedAtUserMetadata where F1: Into<Option<i64>> {
    DirectedAtUserMetadata {
      user_id: user_id.into(),
    }
  }
}

impl TSerializable for DirectedAtUserMetadata {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DirectedAtUserMetadata> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = DirectedAtUserMetadata {
      user_id: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DirectedAtUserMetadata");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SelfThreadMetadata {
              pub id: i64,
      pub is_leaf: Option<bool>,
}

impl SelfThreadMetadata {
  pub fn new<F2>(id: i64, is_leaf: F2) -> SelfThreadMetadata where F2: Into<Option<bool>> {
    SelfThreadMetadata {
      id,
      is_leaf: is_leaf.into(),
    }
  }
}

impl TSerializable for SelfThreadMetadata {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SelfThreadMetadata> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<bool> = Some(false);
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
          let val = i_prot.read_bool()?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("SelfThreadMetadata.id", &f_1)?;
    let ret = SelfThreadMetadata {
      id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      is_leaf: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SelfThreadMetadata");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
    o_prot.write_i64(self.id)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.is_leaf {
      o_prot.write_field_begin(&TFieldIdentifier::new("isLeaf", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConversationControlByInvitation {
  pub invited_user_ids: Vec<i64>,
  pub conversation_tweet_author_id: i64,
  pub invite_via_mention: Option<bool>,
}

impl ConversationControlByInvitation {
  pub fn new<F3>(invited_user_ids: Vec<i64>, conversation_tweet_author_id: i64, invite_via_mention: F3) -> ConversationControlByInvitation where F3: Into<Option<bool>> {
    ConversationControlByInvitation {
      invited_user_ids,
      conversation_tweet_author_id,
      invite_via_mention: invite_via_mention.into(),
    }
  }
}

impl TSerializable for ConversationControlByInvitation {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConversationControlByInvitation> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_11 = i_prot.read_i64()?;
            val.push(list_elem_11);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ConversationControlByInvitation.invited_user_ids", &f_1)?;
    verify_required_field_exists("ConversationControlByInvitation.conversation_tweet_author_id", &f_2)?;
    let ret = ConversationControlByInvitation {
      invited_user_ids: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_tweet_author_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      invite_via_mention: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConversationControlByInvitation");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("invited_user_ids", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.invited_user_ids.len() as i32))?;
    for e in &self.invited_user_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_tweet_author_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_tweet_author_id)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.invite_via_mention {
      o_prot.write_field_begin(&TFieldIdentifier::new("invite_via_mention", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConversationControlCommunity {
  pub invited_user_ids: Vec<i64>,
  pub conversation_tweet_author_id: i64,
  pub invite_via_mention: Option<bool>,
}

impl ConversationControlCommunity {
  pub fn new<F3>(invited_user_ids: Vec<i64>, conversation_tweet_author_id: i64, invite_via_mention: F3) -> ConversationControlCommunity where F3: Into<Option<bool>> {
    ConversationControlCommunity {
      invited_user_ids,
      conversation_tweet_author_id,
      invite_via_mention: invite_via_mention.into(),
    }
  }
}

impl TSerializable for ConversationControlCommunity {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConversationControlCommunity> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_12 = i_prot.read_i64()?;
            val.push(list_elem_12);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ConversationControlCommunity.invited_user_ids", &f_1)?;
    verify_required_field_exists("ConversationControlCommunity.conversation_tweet_author_id", &f_2)?;
    let ret = ConversationControlCommunity {
      invited_user_ids: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_tweet_author_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      invite_via_mention: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConversationControlCommunity");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("invited_user_ids", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.invited_user_ids.len() as i32))?;
    for e in &self.invited_user_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_tweet_author_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_tweet_author_id)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.invite_via_mention {
      o_prot.write_field_begin(&TFieldIdentifier::new("invite_via_mention", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConversationControlFollowers {
  pub invited_user_ids: Vec<i64>,
  pub conversation_tweet_author_id: i64,
  pub invite_via_mention: Option<bool>,
}

impl ConversationControlFollowers {
  pub fn new<F3>(invited_user_ids: Vec<i64>, conversation_tweet_author_id: i64, invite_via_mention: F3) -> ConversationControlFollowers where F3: Into<Option<bool>> {
    ConversationControlFollowers {
      invited_user_ids,
      conversation_tweet_author_id,
      invite_via_mention: invite_via_mention.into(),
    }
  }
}

impl TSerializable for ConversationControlFollowers {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConversationControlFollowers> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_13 = i_prot.read_i64()?;
            val.push(list_elem_13);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ConversationControlFollowers.invited_user_ids", &f_1)?;
    verify_required_field_exists("ConversationControlFollowers.conversation_tweet_author_id", &f_2)?;
    let ret = ConversationControlFollowers {
      invited_user_ids: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_tweet_author_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      invite_via_mention: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConversationControlFollowers");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("invited_user_ids", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.invited_user_ids.len() as i32))?;
    for e in &self.invited_user_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_tweet_author_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_tweet_author_id)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.invite_via_mention {
      o_prot.write_field_begin(&TFieldIdentifier::new("invite_via_mention", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConversationControlSubscribers {
  pub invited_user_ids: Vec<i64>,
  pub conversation_tweet_author_id: i64,
  pub invite_via_mention: Option<bool>,
}

impl ConversationControlSubscribers {
  pub fn new<F3>(invited_user_ids: Vec<i64>, conversation_tweet_author_id: i64, invite_via_mention: F3) -> ConversationControlSubscribers where F3: Into<Option<bool>> {
    ConversationControlSubscribers {
      invited_user_ids,
      conversation_tweet_author_id,
      invite_via_mention: invite_via_mention.into(),
    }
  }
}

impl TSerializable for ConversationControlSubscribers {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConversationControlSubscribers> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_14 = i_prot.read_i64()?;
            val.push(list_elem_14);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ConversationControlSubscribers.invited_user_ids", &f_1)?;
    verify_required_field_exists("ConversationControlSubscribers.conversation_tweet_author_id", &f_2)?;
    let ret = ConversationControlSubscribers {
      invited_user_ids: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_tweet_author_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      invite_via_mention: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConversationControlSubscribers");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("invited_user_ids", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.invited_user_ids.len() as i32))?;
    for e in &self.invited_user_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_tweet_author_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_tweet_author_id)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.invite_via_mention {
      o_prot.write_field_begin(&TFieldIdentifier::new("invite_via_mention", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConversationControlVerified {
  pub invited_user_ids: Vec<i64>,
  pub conversation_tweet_author_id: i64,
  pub invite_via_mention: Option<bool>,
}

impl ConversationControlVerified {
  pub fn new<F3>(invited_user_ids: Vec<i64>, conversation_tweet_author_id: i64, invite_via_mention: F3) -> ConversationControlVerified where F3: Into<Option<bool>> {
    ConversationControlVerified {
      invited_user_ids,
      conversation_tweet_author_id,
      invite_via_mention: invite_via_mention.into(),
    }
  }
}

impl TSerializable for ConversationControlVerified {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConversationControlVerified> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_15 = i_prot.read_i64()?;
            val.push(list_elem_15);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ConversationControlVerified.invited_user_ids", &f_1)?;
    verify_required_field_exists("ConversationControlVerified.conversation_tweet_author_id", &f_2)?;
    let ret = ConversationControlVerified {
      invited_user_ids: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_tweet_author_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      invite_via_mention: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConversationControlVerified");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("invited_user_ids", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.invited_user_ids.len() as i32))?;
    for e in &self.invited_user_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_tweet_author_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_tweet_author_id)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.invite_via_mention {
      o_prot.write_field_begin(&TFieldIdentifier::new("invite_via_mention", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConversationControlLocal {
  pub invited_user_ids: Vec<i64>,
  pub conversation_tweet_author_id: i64,
  pub invite_via_mention: Option<bool>,
}

impl ConversationControlLocal {
  pub fn new<F3>(invited_user_ids: Vec<i64>, conversation_tweet_author_id: i64, invite_via_mention: F3) -> ConversationControlLocal where F3: Into<Option<bool>> {
    ConversationControlLocal {
      invited_user_ids,
      conversation_tweet_author_id,
      invite_via_mention: invite_via_mention.into(),
    }
  }
}

impl TSerializable for ConversationControlLocal {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConversationControlLocal> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_16 = i_prot.read_i64()?;
            val.push(list_elem_16);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ConversationControlLocal.invited_user_ids", &f_1)?;
    verify_required_field_exists("ConversationControlLocal.conversation_tweet_author_id", &f_2)?;
    let ret = ConversationControlLocal {
      invited_user_ids: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_tweet_author_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      invite_via_mention: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConversationControlLocal");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("invited_user_ids", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.invited_user_ids.len() as i32))?;
    for e in &self.invited_user_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_tweet_author_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_tweet_author_id)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.invite_via_mention {
      o_prot.write_field_begin(&TFieldIdentifier::new("invite_via_mention", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConversationControlPremium {
  pub conversation_tweet_author_id: i64,
}

impl ConversationControlPremium {
  pub fn new(conversation_tweet_author_id: i64) -> ConversationControlPremium {
    ConversationControlPremium {
      conversation_tweet_author_id,
    }
  }
}

impl TSerializable for ConversationControlPremium {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConversationControlPremium> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ConversationControlPremium.conversation_tweet_author_id", &f_1)?;
    let ret = ConversationControlPremium {
      conversation_tweet_author_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConversationControlPremium");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_tweet_author_id", TType::I64, 1))?;
    o_prot.write_i64(self.conversation_tweet_author_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConversationControl {
  Community(ConversationControlCommunity),
  ByInvitation(ConversationControlByInvitation),
  Followers(ConversationControlFollowers),
  Subscribers(ConversationControlSubscribers),
  Verified(ConversationControlVerified),
  Local(ConversationControlLocal),
  Premium(ConversationControlPremium),
}

impl TSerializable for ConversationControl {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConversationControl> {
    let mut ret: Option<ConversationControl> = None;
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
          let val = ConversationControlCommunity::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(ConversationControl::Community(val));
          }
          received_field_count += 1;
        },
        2 => {
          let val = ConversationControlByInvitation::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(ConversationControl::ByInvitation(val));
          }
          received_field_count += 1;
        },
        3 => {
          let val = ConversationControlFollowers::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(ConversationControl::Followers(val));
          }
          received_field_count += 1;
        },
        4 => {
          let val = ConversationControlSubscribers::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(ConversationControl::Subscribers(val));
          }
          received_field_count += 1;
        },
        5 => {
          let val = ConversationControlVerified::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(ConversationControl::Verified(val));
          }
          received_field_count += 1;
        },
        6 => {
          let val = ConversationControlLocal::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(ConversationControl::Local(val));
          }
          received_field_count += 1;
        },
        7 => {
          let val = ConversationControlPremium::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(ConversationControl::Premium(val));
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
            "received empty union from remote ConversationControl"
          )
        )
      )
    } else if received_field_count > 1 {
      Err(
        thrift::Error::Protocol(
          ProtocolError::new(
            ProtocolErrorKind::InvalidData,
            "received multiple fields for union from remote ConversationControl"
          )
        )
      )
    } else {
      Ok(ret.expect("return value should have been constructed"))
    }
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConversationControl");
    o_prot.write_struct_begin(&struct_ident)?;
    match *self {
      ConversationControl::Community(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("community", TType::Struct, 1))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      ConversationControl::ByInvitation(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("byInvitation", TType::Struct, 2))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      ConversationControl::Followers(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("followers", TType::Struct, 3))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      ConversationControl::Subscribers(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("subscribers", TType::Struct, 4))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      ConversationControl::Verified(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("verified", TType::Struct, 5))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      ConversationControl::Local(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("local", TType::Struct, 6))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      ConversationControl::Premium(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("premium", TType::Struct, 7))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExclusiveTweetControl {
  pub conversation_author_id: i64,
}

impl ExclusiveTweetControl {
  pub fn new(conversation_author_id: i64) -> ExclusiveTweetControl {
    ExclusiveTweetControl {
      conversation_author_id,
    }
  }
}

impl TSerializable for ExclusiveTweetControl {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ExclusiveTweetControl> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ExclusiveTweetControl.conversation_author_id", &f_1)?;
    let ret = ExclusiveTweetControl {
      conversation_author_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ExclusiveTweetControl");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_author_id", TType::I64, 1))?;
    o_prot.write_i64(self.conversation_author_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PremiumTweetControl {
  pub conversation_author_id: i64,
}

impl PremiumTweetControl {
  pub fn new(conversation_author_id: i64) -> PremiumTweetControl {
    PremiumTweetControl {
      conversation_author_id,
    }
  }
}

impl TSerializable for PremiumTweetControl {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PremiumTweetControl> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("PremiumTweetControl.conversation_author_id", &f_1)?;
    let ret = PremiumTweetControl {
      conversation_author_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("PremiumTweetControl");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_author_id", TType::I64, 1))?;
    o_prot.write_i64(self.conversation_author_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TrustedFriendsControl {
    pub trusted_friends_list_id: i64,
}

impl TrustedFriendsControl {
  pub fn new(trusted_friends_list_id: i64) -> TrustedFriendsControl {
    TrustedFriendsControl {
      trusted_friends_list_id,
    }
  }
}

impl TSerializable for TrustedFriendsControl {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TrustedFriendsControl> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("TrustedFriendsControl.trusted_friends_list_id", &f_1)?;
    let ret = TrustedFriendsControl {
      trusted_friends_list_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TrustedFriendsControl");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("trusted_friends_list_id", TType::I64, 1))?;
    o_prot.write_i64(self.trusted_friends_list_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InvitedCollaborator {
  pub collaborator_user_id: i64,
  pub collab_invitation_status: CollabInvitationStatus,
}

impl InvitedCollaborator {
  pub fn new(collaborator_user_id: i64, collab_invitation_status: CollabInvitationStatus) -> InvitedCollaborator {
    InvitedCollaborator {
      collaborator_user_id,
      collab_invitation_status,
    }
  }
}

impl TSerializable for InvitedCollaborator {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<InvitedCollaborator> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<CollabInvitationStatus> = None;
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
          let val = CollabInvitationStatus::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("InvitedCollaborator.collaborator_user_id", &f_1)?;
    verify_required_field_exists("InvitedCollaborator.collab_invitation_status", &f_2)?;
    let ret = InvitedCollaborator {
      collaborator_user_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      collab_invitation_status: f_2.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("InvitedCollaborator");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("collaborator_user_id", TType::I64, 1))?;
    o_prot.write_i64(self.collaborator_user_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("collab_invitation_status", TType::I32, 2))?;
    self.collab_invitation_status.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CollabInvitation {
  pub invited_collaborators: Vec<InvitedCollaborator>,
}

impl CollabInvitation {
  pub fn new(invited_collaborators: Vec<InvitedCollaborator>) -> CollabInvitation {
    CollabInvitation {
      invited_collaborators,
    }
  }
}

impl TSerializable for CollabInvitation {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CollabInvitation> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<InvitedCollaborator>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<InvitedCollaborator> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_17 = InvitedCollaborator::read_from_in_protocol(i_prot)?;
            val.push(list_elem_17);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("CollabInvitation.invited_collaborators", &f_1)?;
    let ret = CollabInvitation {
      invited_collaborators: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("CollabInvitation");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("invited_collaborators", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, self.invited_collaborators.len() as i32))?;
    for e in &self.invited_collaborators {
      e.write_to_out_protocol(o_prot)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CollabTweet {
  pub collaborator_user_ids: Vec<i64>,
}

impl CollabTweet {
  pub fn new(collaborator_user_ids: Vec<i64>) -> CollabTweet {
    CollabTweet {
      collaborator_user_ids,
    }
  }
}

impl TSerializable for CollabTweet {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CollabTweet> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_18 = i_prot.read_i64()?;
            val.push(list_elem_18);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("CollabTweet.collaborator_user_ids", &f_1)?;
    let ret = CollabTweet {
      collaborator_user_ids: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("CollabTweet");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("collaborator_user_ids", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.collaborator_user_ids.len() as i32))?;
    for e in &self.collaborator_user_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CollabControl {
  CollabInvitation(CollabInvitation),
  CollabTweet(CollabTweet),
}

impl TSerializable for CollabControl {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CollabControl> {
    let mut ret: Option<CollabControl> = None;
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
          let val = CollabInvitation::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(CollabControl::CollabInvitation(val));
          }
          received_field_count += 1;
        },
        2 => {
          let val = CollabTweet::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(CollabControl::CollabTweet(val));
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
            "received empty union from remote CollabControl"
          )
        )
      )
    } else if received_field_count > 1 {
      Err(
        thrift::Error::Protocol(
          ProtocolError::new(
            ProtocolErrorKind::InvalidData,
            "received multiple fields for union from remote CollabControl"
          )
        )
      )
    } else {
      Ok(ret.expect("return value should have been constructed"))
    }
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("CollabControl");
    o_prot.write_struct_begin(&struct_ident)?;
    match *self {
      CollabControl::CollabInvitation(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("collab_invitation", TType::Struct, 1))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      CollabControl::CollabTweet(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("collab_tweet", TType::Struct, 2))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DisallowedReplyControl {
      pub control: DisallowedReply,
}

impl DisallowedReplyControl {
  pub fn new(control: DisallowedReply) -> DisallowedReplyControl {
    DisallowedReplyControl {
      control,
    }
  }
}

impl TSerializable for DisallowedReplyControl {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DisallowedReplyControl> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<DisallowedReply> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = DisallowedReply::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("DisallowedReplyControl.control", &f_1)?;
    let ret = DisallowedReplyControl {
      control: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DisallowedReplyControl");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("control", TType::I32, 1))?;
    self.control.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tweet {
                pub id: Option<i64>,
            pub core_data: Option<TweetCoreData>,
    pub media: Option<Vec<media_entity::MediaEntity>>,
    pub quoted_tweet: Option<QuotedTweet>,
    pub language: Option<Language>,
}

impl Tweet {
  pub fn new<F1, F2, F7, F11, F18>(id: F1, core_data: F2, media: F7, quoted_tweet: F11, language: F18) -> Tweet where F1: Into<Option<i64>>, F2: Into<Option<TweetCoreData>>, F7: Into<Option<Vec<media_entity::MediaEntity>>>, F11: Into<Option<QuotedTweet>>, F18: Into<Option<Language>> {
    Tweet {
      id: id.into(),
      core_data: core_data.into(),
      media: media.into(),
      quoted_tweet: quoted_tweet.into(),
      language: language.into(),
    }
  }
}

impl TSerializable for Tweet {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Tweet> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<TweetCoreData> = None;
    let mut f_7: Option<Vec<media_entity::MediaEntity>> = None;
    let mut f_11: Option<QuotedTweet> = None;
    let mut f_18: Option<Language> = None;
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
          let val = TweetCoreData::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        7 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<media_entity::MediaEntity> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_19 = media_entity::MediaEntity::read_from_in_protocol(i_prot)?;
            val.push(list_elem_19);
          }
          i_prot.read_list_end()?;
          f_7 = Some(val);
        },
        11 => {
          let val = QuotedTweet::read_from_in_protocol(i_prot)?;
          f_11 = Some(val);
        },
        18 => {
          let val = Language::read_from_in_protocol(i_prot)?;
          f_18 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Tweet {
      id: f_1,
      core_data: f_2,
      media: f_7,
      quoted_tweet: f_11,
      language: f_18,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Tweet");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.core_data {
      o_prot.write_field_begin(&TFieldIdentifier::new("core_data", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media {
      o_prot.write_field_begin(&TFieldIdentifier::new("media", TType::List, 7))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.quoted_tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet", TType::Struct, 11))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.language {
      o_prot.write_field_begin(&TFieldIdentifier::new("language", TType::Struct, 18))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

