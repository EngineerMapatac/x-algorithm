
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_extern_crates)]
#![allow(clippy::too_many_arguments, clippy::type_complexity, clippy::vec_box, clippy::wrong_self_convention, clippy::doc_overindented_list_items)]
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

use crate::schema::tweet;
use crate::schema::user;

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MigrationMode(pub i32);

impl MigrationMode {
  pub const OFF: MigrationMode = MigrationMode(0);
  pub const DARK: MigrationMode = MigrationMode(1);
  pub const LIGHT: MigrationMode = MigrationMode(2);
  pub const DARK_WRITE: MigrationMode = MigrationMode(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::OFF,
    Self::DARK,
    Self::LIGHT,
    Self::DARK_WRITE,
  ];
}

impl TSerializable for MigrationMode {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MigrationMode> {
    let enum_value = i_prot.read_i32()?;
    Ok(MigrationMode::from(enum_value))
  }
}

impl From<i32> for MigrationMode {
  fn from(i: i32) -> Self {
    match i {
      0 => MigrationMode::OFF,
      1 => MigrationMode::DARK,
      2 => MigrationMode::LIGHT,
      3 => MigrationMode::DARK_WRITE,
      _ => MigrationMode(i)
    }
  }
}

impl From<&i32> for MigrationMode {
  fn from(i: &i32) -> Self {
    MigrationMode::from(*i)
  }
}

impl From<MigrationMode> for i32 {
  fn from(e: MigrationMode) -> i32 {
    e.0
  }
}

impl From<&MigrationMode> for i32 {
  fn from(e: &MigrationMode) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserIdentifierType(pub i32);

impl UserIdentifierType {
  pub const EMAIL: UserIdentifierType = UserIdentifierType(1);
  pub const PHONE_NUMBER: UserIdentifierType = UserIdentifierType(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::EMAIL,
    Self::PHONE_NUMBER,
  ];
}

impl TSerializable for UserIdentifierType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UserIdentifierType> {
    let enum_value = i_prot.read_i32()?;
    Ok(UserIdentifierType::from(enum_value))
  }
}

impl From<i32> for UserIdentifierType {
  fn from(i: i32) -> Self {
    match i {
      1 => UserIdentifierType::EMAIL,
      2 => UserIdentifierType::PHONE_NUMBER,
      _ => UserIdentifierType(i)
    }
  }
}

impl From<&i32> for UserIdentifierType {
  fn from(i: &i32) -> Self {
    UserIdentifierType::from(*i)
  }
}

impl From<UserIdentifierType> for i32 {
  fn from(e: UserIdentifierType) -> i32 {
    e.0
  }
}

impl From<&UserIdentifierType> for i32 {
  fn from(e: &UserIdentifierType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RemediateFavoriteGraphType(pub i32);

impl RemediateFavoriteGraphType {
  pub const FAVORITE: RemediateFavoriteGraphType = RemediateFavoriteGraphType(0);
  pub const UNFAVORITE: RemediateFavoriteGraphType = RemediateFavoriteGraphType(1);
  pub const SELF_FAVORITE: RemediateFavoriteGraphType = RemediateFavoriteGraphType(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::FAVORITE,
    Self::UNFAVORITE,
    Self::SELF_FAVORITE,
  ];
}

impl TSerializable for RemediateFavoriteGraphType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<RemediateFavoriteGraphType> {
    let enum_value = i_prot.read_i32()?;
    Ok(RemediateFavoriteGraphType::from(enum_value))
  }
}

impl From<i32> for RemediateFavoriteGraphType {
  fn from(i: i32) -> Self {
    match i {
      0 => RemediateFavoriteGraphType::FAVORITE,
      1 => RemediateFavoriteGraphType::UNFAVORITE,
      2 => RemediateFavoriteGraphType::SELF_FAVORITE,
      _ => RemediateFavoriteGraphType(i)
    }
  }
}

impl From<&i32> for RemediateFavoriteGraphType {
  fn from(i: &i32) -> Self {
    RemediateFavoriteGraphType::from(*i)
  }
}

impl From<RemediateFavoriteGraphType> for i32 {
  fn from(e: RemediateFavoriteGraphType) -> i32 {
    e.0
  }
}

impl From<&RemediateFavoriteGraphType> for i32 {
  fn from(e: &RemediateFavoriteGraphType) -> i32 {
    e.0
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FollowEvent {
  pub user_id: Option<i64>,
  pub target_id: Option<i64>,
  pub event_time_ms: Option<i64>,
}

impl FollowEvent {
  pub fn new<F1, F2, F3>(user_id: F1, target_id: F2, event_time_ms: F3) -> FollowEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>> {
    FollowEvent {
      user_id: user_id.into(),
      target_id: target_id.into(),
      event_time_ms: event_time_ms.into(),
    }
  }
}

impl TSerializable for FollowEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FollowEvent> {
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
    let ret = FollowEvent {
      user_id: f_1,
      target_id: f_2,
      event_time_ms: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FollowEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.target_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("target_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BatchFollowEvent {
  pub user_id: i64,
  pub event_time_ms: i64,
  pub target_ids: Vec<i64>,
}

impl BatchFollowEvent {
  pub fn new(user_id: i64, event_time_ms: i64, target_ids: Vec<i64>) -> BatchFollowEvent {
    BatchFollowEvent {
      user_id,
      event_time_ms,
      target_ids,
    }
  }
}

impl TSerializable for BatchFollowEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<BatchFollowEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<Vec<i64>> = None;
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
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_0 = i_prot.read_i64()?;
            val.push(list_elem_0);
          }
          i_prot.read_list_end()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("BatchFollowEvent.user_id", &f_1)?;
    verify_required_field_exists("BatchFollowEvent.event_time_ms", &f_2)?;
    verify_required_field_exists("BatchFollowEvent.target_ids", &f_3)?;
    let ret = BatchFollowEvent {
      user_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      event_time_ms: f_2.expect("auto-generated code should have checked for presence of required fields"),
      target_ids: f_3.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("BatchFollowEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
    o_prot.write_i64(self.user_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 2))?;
    o_prot.write_i64(self.event_time_ms)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("target_ids", TType::List, 3))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.target_ids.len() as i32))?;
    for e in &self.target_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnfollowEvent {
  pub user_id: Option<i64>,
  pub target_id: Option<i64>,
  pub event_time_ms: Option<i64>,
}

impl UnfollowEvent {
  pub fn new<F1, F2, F3>(user_id: F1, target_id: F2, event_time_ms: F3) -> UnfollowEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>> {
    UnfollowEvent {
      user_id: user_id.into(),
      target_id: target_id.into(),
      event_time_ms: event_time_ms.into(),
    }
  }
}

impl TSerializable for UnfollowEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UnfollowEvent> {
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
    let ret = UnfollowEvent {
      user_id: f_1,
      target_id: f_2,
      event_time_ms: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UnfollowEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.target_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("target_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FollowRetweetsEvent {
  pub user_id: Option<i64>,
  pub target_id: Option<i64>,
  pub event_time_ms: Option<i64>,
}

impl FollowRetweetsEvent {
  pub fn new<F1, F2, F3>(user_id: F1, target_id: F2, event_time_ms: F3) -> FollowRetweetsEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>> {
    FollowRetweetsEvent {
      user_id: user_id.into(),
      target_id: target_id.into(),
      event_time_ms: event_time_ms.into(),
    }
  }
}

impl TSerializable for FollowRetweetsEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FollowRetweetsEvent> {
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
    let ret = FollowRetweetsEvent {
      user_id: f_1,
      target_id: f_2,
      event_time_ms: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FollowRetweetsEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.target_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("target_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnfollowRetweetsEvent {
  pub user_id: Option<i64>,
  pub target_id: Option<i64>,
  pub event_time_ms: Option<i64>,
}

impl UnfollowRetweetsEvent {
  pub fn new<F1, F2, F3>(user_id: F1, target_id: F2, event_time_ms: F3) -> UnfollowRetweetsEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>> {
    UnfollowRetweetsEvent {
      user_id: user_id.into(),
      target_id: target_id.into(),
      event_time_ms: event_time_ms.into(),
    }
  }
}

impl TSerializable for UnfollowRetweetsEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UnfollowRetweetsEvent> {
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
    let ret = UnfollowRetweetsEvent {
      user_id: f_1,
      target_id: f_2,
      event_time_ms: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UnfollowRetweetsEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.target_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("target_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FavoriteEvent {
  pub user_id: Option<i64>,
  pub tweet_id: Option<i64>,
  pub tweet_user_id: Option<i64>,
  pub event_time_ms: Option<i64>,
  pub retweet_id: Option<i64>,
  pub user: Option<user::User>,
  pub tweet_user: Option<user::User>,
  pub tweet: Option<tweet::Tweet>,
  pub is_soft_user: Option<bool>,
  pub in_reply_to_tweet_id: Option<i64>,
  pub quoted_tweet_id: Option<i64>,
  pub migration_mode: Option<MigrationMode>,
}

impl FavoriteEvent {
  pub fn new<F1, F2, F3, F4, F5, F6, F8, F11, F12, F15, F16, F100>(user_id: F1, tweet_id: F2, tweet_user_id: F3, event_time_ms: F4, retweet_id: F5, user: F6, tweet_user: F8, tweet: F11, is_soft_user: F12, in_reply_to_tweet_id: F15, quoted_tweet_id: F16, migration_mode: F100) -> FavoriteEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<i64>>, F6: Into<Option<user::User>>, F8: Into<Option<user::User>>, F11: Into<Option<tweet::Tweet>>, F12: Into<Option<bool>>, F15: Into<Option<i64>>, F16: Into<Option<i64>>, F100: Into<Option<MigrationMode>> {
    FavoriteEvent {
      user_id: user_id.into(),
      tweet_id: tweet_id.into(),
      tweet_user_id: tweet_user_id.into(),
      event_time_ms: event_time_ms.into(),
      retweet_id: retweet_id.into(),
      user: user.into(),
      tweet_user: tweet_user.into(),
      tweet: tweet.into(),
      is_soft_user: is_soft_user.into(),
      in_reply_to_tweet_id: in_reply_to_tweet_id.into(),
      quoted_tweet_id: quoted_tweet_id.into(),
      migration_mode: migration_mode.into(),
    }
  }
}

impl TSerializable for FavoriteEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FavoriteEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<user::User> = None;
    let mut f_8: Option<user::User> = None;
    let mut f_11: Option<tweet::Tweet> = None;
    let mut f_12: Option<bool> = None;
    let mut f_15: Option<i64> = None;
    let mut f_16: Option<i64> = None;
    let mut f_100: Option<MigrationMode> = None;
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
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_6 = Some(val);
        },
        8 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_8 = Some(val);
        },
        11 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_bool()?;
          f_12 = Some(val);
        },
        15 => {
          let val = i_prot.read_i64()?;
          f_15 = Some(val);
        },
        16 => {
          let val = i_prot.read_i64()?;
          f_16 = Some(val);
        },
        100 => {
          let val = MigrationMode::read_from_in_protocol(i_prot)?;
          f_100 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = FavoriteEvent {
      user_id: f_1,
      tweet_id: f_2,
      tweet_user_id: f_3,
      event_time_ms: f_4,
      retweet_id: f_5,
      user: f_6,
      tweet_user: f_8,
      tweet: f_11,
      is_soft_user: f_12,
      in_reply_to_tweet_id: f_15,
      quoted_tweet_id: f_16,
      migration_mode: f_100,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FavoriteEvent");
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
    if let Some(fld_var) = self.tweet_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.retweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweet_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user {
      o_prot.write_field_begin(&TFieldIdentifier::new("user", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.tweet_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user", TType::Struct, 8))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 11))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_soft_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_soft_user", TType::Bool, 12))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.in_reply_to_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("in_reply_to_tweet_id", TType::I64, 15))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoted_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet_id", TType::I64, 16))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.migration_mode {
      o_prot.write_field_begin(&TFieldIdentifier::new("migration_mode", TType::I32, 100))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnfavoriteEvent {
  pub user_id: Option<i64>,
  pub tweet_id: Option<i64>,
  pub tweet_user_id: Option<i64>,
  pub event_time_ms: Option<i64>,
  pub retweet_id: Option<i64>,
  pub user: Option<user::User>,
  pub tweet_user: Option<user::User>,
  pub tweet: Option<tweet::Tweet>,
  pub is_soft_user: Option<bool>,
  pub in_reply_to_tweet_id: Option<i64>,
  pub quoted_tweet_id: Option<i64>,
  pub migration_mode: Option<MigrationMode>,
}

impl UnfavoriteEvent {
  pub fn new<F1, F2, F3, F4, F5, F6, F8, F10, F11, F13, F14, F100>(user_id: F1, tweet_id: F2, tweet_user_id: F3, event_time_ms: F4, retweet_id: F5, user: F6, tweet_user: F8, tweet: F10, is_soft_user: F11, in_reply_to_tweet_id: F13, quoted_tweet_id: F14, migration_mode: F100) -> UnfavoriteEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<i64>>, F6: Into<Option<user::User>>, F8: Into<Option<user::User>>, F10: Into<Option<tweet::Tweet>>, F11: Into<Option<bool>>, F13: Into<Option<i64>>, F14: Into<Option<i64>>, F100: Into<Option<MigrationMode>> {
    UnfavoriteEvent {
      user_id: user_id.into(),
      tweet_id: tweet_id.into(),
      tweet_user_id: tweet_user_id.into(),
      event_time_ms: event_time_ms.into(),
      retweet_id: retweet_id.into(),
      user: user.into(),
      tweet_user: tweet_user.into(),
      tweet: tweet.into(),
      is_soft_user: is_soft_user.into(),
      in_reply_to_tweet_id: in_reply_to_tweet_id.into(),
      quoted_tweet_id: quoted_tweet_id.into(),
      migration_mode: migration_mode.into(),
    }
  }
}

impl TSerializable for UnfavoriteEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UnfavoriteEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<user::User> = None;
    let mut f_8: Option<user::User> = None;
    let mut f_10: Option<tweet::Tweet> = None;
    let mut f_11: Option<bool> = None;
    let mut f_13: Option<i64> = None;
    let mut f_14: Option<i64> = None;
    let mut f_100: Option<MigrationMode> = None;
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
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_6 = Some(val);
        },
        8 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_8 = Some(val);
        },
        10 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_bool()?;
          f_11 = Some(val);
        },
        13 => {
          let val = i_prot.read_i64()?;
          f_13 = Some(val);
        },
        14 => {
          let val = i_prot.read_i64()?;
          f_14 = Some(val);
        },
        100 => {
          let val = MigrationMode::read_from_in_protocol(i_prot)?;
          f_100 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = UnfavoriteEvent {
      user_id: f_1,
      tweet_id: f_2,
      tweet_user_id: f_3,
      event_time_ms: f_4,
      retweet_id: f_5,
      user: f_6,
      tweet_user: f_8,
      tweet: f_10,
      is_soft_user: f_11,
      in_reply_to_tweet_id: f_13,
      quoted_tweet_id: f_14,
      migration_mode: f_100,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UnfavoriteEvent");
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
    if let Some(fld_var) = self.tweet_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.retweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweet_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user {
      o_prot.write_field_begin(&TFieldIdentifier::new("user", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.tweet_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user", TType::Struct, 8))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 10))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_soft_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_soft_user", TType::Bool, 11))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.in_reply_to_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("in_reply_to_tweet_id", TType::I64, 13))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoted_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet_id", TType::I64, 14))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.migration_mode {
      o_prot.write_field_begin(&TFieldIdentifier::new("migration_mode", TType::I32, 100))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FavoriteArchivalEvent {
  pub favoriter_id: i64,
  pub tweet_id: i64,
  pub timestamp_ms: i64,
  pub is_archiving_action: Option<bool>,
  pub source_tweet_id: Option<i64>,
  pub tweet_user_id: Option<i64>,
}

impl FavoriteArchivalEvent {
  pub fn new<F4, F5, F6>(favoriter_id: i64, tweet_id: i64, timestamp_ms: i64, is_archiving_action: F4, source_tweet_id: F5, tweet_user_id: F6) -> FavoriteArchivalEvent where F4: Into<Option<bool>>, F5: Into<Option<i64>>, F6: Into<Option<i64>> {
    FavoriteArchivalEvent {
      favoriter_id,
      tweet_id,
      timestamp_ms,
      is_archiving_action: is_archiving_action.into(),
      source_tweet_id: source_tweet_id.into(),
      tweet_user_id: tweet_user_id.into(),
    }
  }
}

impl TSerializable for FavoriteArchivalEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FavoriteArchivalEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
    let mut f_4: Option<bool> = None;
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<i64> = None;
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
          let val = i_prot.read_bool()?;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("FavoriteArchivalEvent.favoriter_id", &f_1)?;
    verify_required_field_exists("FavoriteArchivalEvent.tweet_id", &f_2)?;
    verify_required_field_exists("FavoriteArchivalEvent.timestamp_ms", &f_3)?;
    let ret = FavoriteArchivalEvent {
      favoriter_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      tweet_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      timestamp_ms: f_3.expect("auto-generated code should have checked for presence of required fields"),
      is_archiving_action: f_4,
      source_tweet_id: f_5,
      tweet_user_id: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FavoriteArchivalEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("favoriter_id", TType::I64, 1))?;
    o_prot.write_i64(self.favoriter_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("tweet_id", TType::I64, 2))?;
    o_prot.write_i64(self.tweet_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("timestamp_ms", TType::I64, 3))?;
    o_prot.write_i64(self.timestamp_ms)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.is_archiving_action {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_archiving_action", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.source_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_tweet_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.tweet_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user_id", TType::I64, 6))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClearCachesEvent {
}

impl ClearCachesEvent {
  pub fn new() -> ClearCachesEvent {
    ClearCachesEvent {}
  }
}

impl TSerializable for ClearCachesEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ClearCachesEvent> {
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
    let ret = ClearCachesEvent {};
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ClearCachesEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClearUnreadCountCachesEvent {
}

impl ClearUnreadCountCachesEvent {
  pub fn new() -> ClearUnreadCountCachesEvent {
    ClearUnreadCountCachesEvent {}
  }
}

impl TSerializable for ClearUnreadCountCachesEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ClearUnreadCountCachesEvent> {
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
    let ret = ClearUnreadCountCachesEvent {};
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ClearUnreadCountCachesEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeleteUserEvent {
  pub user_id: Option<i64>,
}

impl DeleteUserEvent {
  pub fn new<F1>(user_id: F1) -> DeleteUserEvent where F1: Into<Option<i64>> {
    DeleteUserEvent {
      user_id: user_id.into(),
    }
  }
}

impl TSerializable for DeleteUserEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DeleteUserEvent> {
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
    let ret = DeleteUserEvent {
      user_id: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DeleteUserEvent");
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


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClearUserCachesEvent {
  pub user_id: Option<i64>,
}

impl ClearUserCachesEvent {
  pub fn new<F1>(user_id: F1) -> ClearUserCachesEvent where F1: Into<Option<i64>> {
    ClearUserCachesEvent {
      user_id: user_id.into(),
    }
  }
}

impl TSerializable for ClearUserCachesEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ClearUserCachesEvent> {
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
    let ret = ClearUserCachesEvent {
      user_id: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ClearUserCachesEvent");
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


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Retweet {
  pub source_user_id: Option<i64>,
  pub source_tweet_id: Option<i64>,
  pub parent_tweet_id: Option<i64>,
}

impl Retweet {
  pub fn new<F1, F2, F3>(source_user_id: F1, source_tweet_id: F2, parent_tweet_id: F3) -> Retweet where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>> {
    Retweet {
      source_user_id: source_user_id.into(),
      source_tweet_id: source_tweet_id.into(),
      parent_tweet_id: parent_tweet_id.into(),
    }
  }
}

impl TSerializable for Retweet {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Retweet> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = None;
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
    let ret = Retweet {
      source_user_id: f_1,
      source_tweet_id: f_2,
      parent_tweet_id: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Retweet");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.source_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.source_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_tweet_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.parent_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("parent_tweet_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Quote {
  pub quoted_user_id: Option<i64>,
  pub quoted_tweet_id: Option<i64>,
}

impl Quote {
  pub fn new<F1, F2>(quoted_user_id: F1, quoted_tweet_id: F2) -> Quote where F1: Into<Option<i64>>, F2: Into<Option<i64>> {
    Quote {
      quoted_user_id: quoted_user_id.into(),
      quoted_tweet_id: quoted_tweet_id.into(),
    }
  }
}

impl TSerializable for Quote {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Quote> {
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
    let ret = Quote {
      quoted_user_id: f_1,
      quoted_tweet_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Quote");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.quoted_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoted_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Reply {
  pub in_reply_to_user_id: Option<i64>,
  pub in_reply_to_tweet_id: Option<i64>,
}

impl Reply {
  pub fn new<F1, F2>(in_reply_to_user_id: F1, in_reply_to_tweet_id: F2) -> Reply where F1: Into<Option<i64>>, F2: Into<Option<i64>> {
    Reply {
      in_reply_to_user_id: in_reply_to_user_id.into(),
      in_reply_to_tweet_id: in_reply_to_tweet_id.into(),
    }
  }
}

impl TSerializable for Reply {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Reply> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = None;
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
      in_reply_to_user_id: f_1,
      in_reply_to_tweet_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Reply");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.in_reply_to_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("in_reply_to_user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.in_reply_to_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("in_reply_to_tweet_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FullTweet {
  pub user_id: Option<i64>,
  pub tweet_id: Option<i64>,
  pub conversation_id: Option<i64>,
  pub reply: Option<Reply>,
  pub retweet: Option<Retweet>,
  pub mentioned_user_ids: Option<BTreeSet<i64>>,
  pub is_nullcasted: Option<bool>,
  pub narrowcast_geos: Option<BTreeSet<String>>,
  pub has_media: Option<bool>,
  pub created_at_ms: Option<i64>,
  pub directed_at_user_id: Option<i64>,
  pub quote: Option<Quote>,
  pub media_tags: Option<tweet::TweetMediaTags>,
  pub text: Option<String>,
  pub community_ids: Option<Vec<i64>>,
  pub trusted_friends_control: Option<tweet::TrustedFriendsControl>,
}

impl FullTweet {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F17>(user_id: F1, tweet_id: F2, conversation_id: F3, reply: F4, retweet: F5, mentioned_user_ids: F6, is_nullcasted: F7, narrowcast_geos: F8, has_media: F9, created_at_ms: F10, directed_at_user_id: F11, quote: F12, media_tags: F13, text: F14, community_ids: F15, trusted_friends_control: F17) -> FullTweet where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<Reply>>, F5: Into<Option<Retweet>>, F6: Into<Option<BTreeSet<i64>>>, F7: Into<Option<bool>>, F8: Into<Option<BTreeSet<String>>>, F9: Into<Option<bool>>, F10: Into<Option<i64>>, F11: Into<Option<i64>>, F12: Into<Option<Quote>>, F13: Into<Option<tweet::TweetMediaTags>>, F14: Into<Option<String>>, F15: Into<Option<Vec<i64>>>, F17: Into<Option<tweet::TrustedFriendsControl>> {
    FullTweet {
      user_id: user_id.into(),
      tweet_id: tweet_id.into(),
      conversation_id: conversation_id.into(),
      reply: reply.into(),
      retweet: retweet.into(),
      mentioned_user_ids: mentioned_user_ids.into(),
      is_nullcasted: is_nullcasted.into(),
      narrowcast_geos: narrowcast_geos.into(),
      has_media: has_media.into(),
      created_at_ms: created_at_ms.into(),
      directed_at_user_id: directed_at_user_id.into(),
      quote: quote.into(),
      media_tags: media_tags.into(),
      text: text.into(),
      community_ids: community_ids.into(),
      trusted_friends_control: trusted_friends_control.into(),
    }
  }
}

impl TSerializable for FullTweet {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FullTweet> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<Reply> = None;
    let mut f_5: Option<Retweet> = None;
    let mut f_6: Option<BTreeSet<i64>> = Some(BTreeSet::new());
    let mut f_7: Option<bool> = Some(false);
    let mut f_8: Option<BTreeSet<String>> = Some(BTreeSet::new());
    let mut f_9: Option<bool> = Some(false);
    let mut f_10: Option<i64> = Some(0);
    let mut f_11: Option<i64> = None;
    let mut f_12: Option<Quote> = None;
    let mut f_13: Option<tweet::TweetMediaTags> = None;
    let mut f_14: Option<String> = None;
    let mut f_15: Option<Vec<i64>> = None;
    let mut f_17: Option<tweet::TrustedFriendsControl> = None;
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
          let val = Reply::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        5 => {
          let val = Retweet::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        6 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<i64> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_1 = i_prot.read_i64()?;
            val.insert(set_elem_1);
          }
          i_prot.read_set_end()?;
          f_6 = Some(val);
        },
        7 => {
          let val = i_prot.read_bool()?;
          f_7 = Some(val);
        },
        8 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<String> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_2 = i_prot.read_string()?;
            val.insert(set_elem_2);
          }
          i_prot.read_set_end()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_i64()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_i64()?;
          f_11 = Some(val);
        },
        12 => {
          let val = Quote::read_from_in_protocol(i_prot)?;
          f_12 = Some(val);
        },
        13 => {
          let val = tweet::TweetMediaTags::read_from_in_protocol(i_prot)?;
          f_13 = Some(val);
        },
        14 => {
          let val = i_prot.read_string()?;
          f_14 = Some(val);
        },
        15 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_3 = i_prot.read_i64()?;
            val.push(list_elem_3);
          }
          i_prot.read_list_end()?;
          f_15 = Some(val);
        },
        17 => {
          let val = tweet::TrustedFriendsControl::read_from_in_protocol(i_prot)?;
          f_17 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = FullTweet {
      user_id: f_1,
      tweet_id: f_2,
      conversation_id: f_3,
      reply: f_4,
      retweet: f_5,
      mentioned_user_ids: f_6,
      is_nullcasted: f_7,
      narrowcast_geos: f_8,
      has_media: f_9,
      created_at_ms: f_10,
      directed_at_user_id: f_11,
      quote: f_12,
      media_tags: f_13,
      text: f_14,
      community_ids: f_15,
      trusted_friends_control: f_17,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FullTweet");
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
    if let Some(fld_var) = self.conversation_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("conversation_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.reply {
      o_prot.write_field_begin(&TFieldIdentifier::new("reply", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.retweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweet", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.mentioned_user_ids {
      o_prot.write_field_begin(&TFieldIdentifier::new("mentioned_user_ids", TType::Set, 6))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::I64, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_i64(*e)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_nullcasted {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_nullcasted", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.narrowcast_geos {
      o_prot.write_field_begin(&TFieldIdentifier::new("narrowcast_geos", TType::Set, 8))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::String, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_string(e)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_media {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_media", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_ms", TType::I64, 10))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.directed_at_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("directed_at_user_id", TType::I64, 11))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.quote {
      o_prot.write_field_begin(&TFieldIdentifier::new("quote", TType::Struct, 12))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media_tags {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_tags", TType::Struct, 13))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.text {
      o_prot.write_field_begin(&TFieldIdentifier::new("text", TType::String, 14))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.community_ids {
      o_prot.write_field_begin(&TFieldIdentifier::new("communityIds", TType::List, 15))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::I64, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_i64(*e)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.trusted_friends_control {
      o_prot.write_field_begin(&TFieldIdentifier::new("trusted_friends_control", TType::Struct, 17))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FullTweetCreateEvent {
  pub tweet: Option<FullTweet>,
  pub event_time_ms: Option<i64>,
  pub sync_delivery: Option<bool>,
  pub fanoutservice_fanout: Option<bool>,
  pub hydrated_tweet: Option<tweet::Tweet>,
}

impl FullTweetCreateEvent {
  pub fn new<F1, F2, F4, F5, F7>(tweet: F1, event_time_ms: F2, sync_delivery: F4, fanoutservice_fanout: F5, hydrated_tweet: F7) -> FullTweetCreateEvent where F1: Into<Option<FullTweet>>, F2: Into<Option<i64>>, F4: Into<Option<bool>>, F5: Into<Option<bool>>, F7: Into<Option<tweet::Tweet>> {
    FullTweetCreateEvent {
      tweet: tweet.into(),
      event_time_ms: event_time_ms.into(),
      sync_delivery: sync_delivery.into(),
      fanoutservice_fanout: fanoutservice_fanout.into(),
      hydrated_tweet: hydrated_tweet.into(),
    }
  }
}

impl TSerializable for FullTweetCreateEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FullTweetCreateEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<FullTweet> = None;
    let mut f_2: Option<i64> = Some(0);
    let mut f_4: Option<bool> = Some(false);
    let mut f_5: Option<bool> = Some(false);
    let mut f_7: Option<tweet::Tweet> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = FullTweet::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        4 => {
          let val = i_prot.read_bool()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_bool()?;
          f_5 = Some(val);
        },
        7 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_7 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = FullTweetCreateEvent {
      tweet: f_1,
      event_time_ms: f_2,
      sync_delivery: f_4,
      fanoutservice_fanout: f_5,
      hydrated_tweet: f_7,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FullTweetCreateEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.sync_delivery {
      o_prot.write_field_begin(&TFieldIdentifier::new("syncDelivery", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.fanoutservice_fanout {
      o_prot.write_field_begin(&TFieldIdentifier::new("fanoutserviceFanout", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.hydrated_tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("hydratedTweet", TType::Struct, 7))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FullTweetRestoreEvent {
  pub tweet: Option<FullTweet>,
  pub delete_time_ms: Option<i64>,
}

impl FullTweetRestoreEvent {
  pub fn new<F1, F2>(tweet: F1, delete_time_ms: F2) -> FullTweetRestoreEvent where F1: Into<Option<FullTweet>>, F2: Into<Option<i64>> {
    FullTweetRestoreEvent {
      tweet: tweet.into(),
      delete_time_ms: delete_time_ms.into(),
    }
  }
}

impl TSerializable for FullTweetRestoreEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FullTweetRestoreEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<FullTweet> = None;
    let mut f_2: Option<i64> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = FullTweet::read_from_in_protocol(i_prot)?;
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
    let ret = FullTweetRestoreEvent {
      tweet: f_1,
      delete_time_ms: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FullTweetRestoreEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.delete_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("delete_time_ms", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FullTweetDeleteEvent {
  pub tweet: Option<FullTweet>,
  pub event_time_ms: Option<i64>,
  pub is_user_erasure: Option<bool>,
          pub is_bounce_delete: Option<bool>,
}

impl FullTweetDeleteEvent {
  pub fn new<F1, F2, F3, F4>(tweet: F1, event_time_ms: F2, is_user_erasure: F3, is_bounce_delete: F4) -> FullTweetDeleteEvent where F1: Into<Option<FullTweet>>, F2: Into<Option<i64>>, F3: Into<Option<bool>>, F4: Into<Option<bool>> {
    FullTweetDeleteEvent {
      tweet: tweet.into(),
      event_time_ms: event_time_ms.into(),
      is_user_erasure: is_user_erasure.into(),
      is_bounce_delete: is_bounce_delete.into(),
    }
  }
}

impl TSerializable for FullTweetDeleteEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FullTweetDeleteEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<FullTweet> = None;
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<bool> = None;
    let mut f_4: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = FullTweet::read_from_in_protocol(i_prot)?;
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
    let ret = FullTweetDeleteEvent {
      tweet: f_1,
      event_time_ms: f_2,
      is_user_erasure: f_3,
      is_bounce_delete: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FullTweetDeleteEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_user_erasure {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_user_erasure", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_bounce_delete {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_bounce_delete", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AddListMemberEvent {
  pub list_id: Option<i64>,
  pub target_id: Option<i64>,
  pub list_owner_id: Option<i64>,
  pub event_time_ms: Option<i64>,
  pub private_list: Option<bool>,
}

impl AddListMemberEvent {
  pub fn new<F1, F2, F3, F4, F5>(list_id: F1, target_id: F2, list_owner_id: F3, event_time_ms: F4, private_list: F5) -> AddListMemberEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<bool>> {
    AddListMemberEvent {
      list_id: list_id.into(),
      target_id: target_id.into(),
      list_owner_id: list_owner_id.into(),
      event_time_ms: event_time_ms.into(),
      private_list: private_list.into(),
    }
  }
}

impl TSerializable for AddListMemberEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AddListMemberEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<bool> = Some(false);
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
          let val = i_prot.read_bool()?;
          f_5 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = AddListMemberEvent {
      list_id: f_1,
      target_id: f_2,
      list_owner_id: f_3,
      event_time_ms: f_4,
      private_list: f_5,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AddListMemberEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.list_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.target_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("target_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.list_owner_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_owner_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.private_list {
      o_prot.write_field_begin(&TFieldIdentifier::new("private_list", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RemoveListMemberEvent {
  pub list_id: Option<i64>,
  pub target_id: Option<i64>,
  pub list_owner_id: Option<i64>,
  pub event_time_ms: Option<i64>,
}

impl RemoveListMemberEvent {
  pub fn new<F1, F2, F3, F4>(list_id: F1, target_id: F2, list_owner_id: F3, event_time_ms: F4) -> RemoveListMemberEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>> {
    RemoveListMemberEvent {
      list_id: list_id.into(),
      target_id: target_id.into(),
      list_owner_id: list_owner_id.into(),
      event_time_ms: event_time_ms.into(),
    }
  }
}

impl TSerializable for RemoveListMemberEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<RemoveListMemberEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<i64> = Some(0);
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = RemoveListMemberEvent {
      list_id: f_1,
      target_id: f_2,
      list_owner_id: f_3,
      event_time_ms: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("RemoveListMemberEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.list_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.target_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("target_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.list_owner_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_owner_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CreateListEvent {
  pub list_id: Option<i64>,
  pub list_owner_id: Option<i64>,
  pub event_time_ms: Option<i64>,
  pub private_list: Option<bool>,
}

impl CreateListEvent {
  pub fn new<F1, F2, F3, F4>(list_id: F1, list_owner_id: F2, event_time_ms: F3, private_list: F4) -> CreateListEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<bool>> {
    CreateListEvent {
      list_id: list_id.into(),
      list_owner_id: list_owner_id.into(),
      event_time_ms: event_time_ms.into(),
      private_list: private_list.into(),
    }
  }
}

impl TSerializable for CreateListEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CreateListEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<bool> = Some(false);
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
    let ret = CreateListEvent {
      list_id: f_1,
      list_owner_id: f_2,
      event_time_ms: f_3,
      private_list: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("CreateListEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.list_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.list_owner_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_owner_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.private_list {
      o_prot.write_field_begin(&TFieldIdentifier::new("private_list", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeleteListEvent {
  pub list_id: Option<i64>,
  pub list_owner_id: Option<i64>,
  pub event_time_ms: Option<i64>,
  pub private_list: Option<bool>,
}

impl DeleteListEvent {
  pub fn new<F1, F2, F3, F4>(list_id: F1, list_owner_id: F2, event_time_ms: F3, private_list: F4) -> DeleteListEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<bool>> {
    DeleteListEvent {
      list_id: list_id.into(),
      list_owner_id: list_owner_id.into(),
      event_time_ms: event_time_ms.into(),
      private_list: private_list.into(),
    }
  }
}

impl TSerializable for DeleteListEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DeleteListEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<bool> = Some(false);
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
    let ret = DeleteListEvent {
      list_id: f_1,
      list_owner_id: f_2,
      event_time_ms: f_3,
      private_list: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DeleteListEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.list_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.list_owner_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_owner_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.private_list {
      o_prot.write_field_begin(&TFieldIdentifier::new("private_list", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ListPrivacyChangeEvent {
  pub list_id: Option<i64>,
  pub list_owner_id: Option<i64>,
  pub private_list: Option<bool>,
  pub event_time_ms: Option<i64>,
}

impl ListPrivacyChangeEvent {
  pub fn new<F1, F2, F3, F4>(list_id: F1, list_owner_id: F2, private_list: F3, event_time_ms: F4) -> ListPrivacyChangeEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<bool>>, F4: Into<Option<i64>> {
    ListPrivacyChangeEvent {
      list_id: list_id.into(),
      list_owner_id: list_owner_id.into(),
      private_list: private_list.into(),
      event_time_ms: event_time_ms.into(),
    }
  }
}

impl TSerializable for ListPrivacyChangeEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ListPrivacyChangeEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<bool> = Some(false);
    let mut f_4: Option<i64> = Some(0);
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
          let val = i_prot.read_bool()?;
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
    let ret = ListPrivacyChangeEvent {
      list_id: f_1,
      list_owner_id: f_2,
      private_list: f_3,
      event_time_ms: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ListPrivacyChangeEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.list_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.list_owner_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("list_owner_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.private_list {
      o_prot.write_field_begin(&TFieldIdentifier::new("private_list", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CreateUserEvent {
  pub user_id: Option<i64>,
  pub event_time_ms: Option<i64>,
}

impl CreateUserEvent {
  pub fn new<F1, F2>(user_id: F1, event_time_ms: F2) -> CreateUserEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>> {
    CreateUserEvent {
      user_id: user_id.into(),
      event_time_ms: event_time_ms.into(),
    }
  }
}

impl TSerializable for CreateUserEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CreateUserEvent> {
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
    let ret = CreateUserEvent {
      user_id: f_1,
      event_time_ms: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("CreateUserEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserIdentifierConfirmedEvent {
  pub user_id: Option<i64>,
  pub identifier: Option<String>,
  pub event_time_ms: Option<i64>,
  pub identifier_type: Option<UserIdentifierType>,
}

impl UserIdentifierConfirmedEvent {
  pub fn new<F1, F2, F3, F4>(user_id: F1, identifier: F2, event_time_ms: F3, identifier_type: F4) -> UserIdentifierConfirmedEvent where F1: Into<Option<i64>>, F2: Into<Option<String>>, F3: Into<Option<i64>>, F4: Into<Option<UserIdentifierType>> {
    UserIdentifierConfirmedEvent {
      user_id: user_id.into(),
      identifier: identifier.into(),
      event_time_ms: event_time_ms.into(),
      identifier_type: identifier_type.into(),
    }
  }
}

impl TSerializable for UserIdentifierConfirmedEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UserIdentifierConfirmedEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<UserIdentifierType> = None;
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
          let val = i_prot.read_i64()?;
          f_3 = Some(val);
        },
        4 => {
          let val = UserIdentifierType::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = UserIdentifierConfirmedEvent {
      user_id: f_1,
      identifier: f_2,
      event_time_ms: f_3,
      identifier_type: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UserIdentifierConfirmedEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.identifier {
      o_prot.write_field_begin(&TFieldIdentifier::new("identifier", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.identifier_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("identifier_type", TType::I32, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReportTweetEvent {
  pub user_id: Option<i64>,
  pub tweet_id: Option<i64>,
  pub event_time_ms: Option<i64>,
}

impl ReportTweetEvent {
  pub fn new<F1, F2, F3>(user_id: F1, tweet_id: F2, event_time_ms: F3) -> ReportTweetEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>> {
    ReportTweetEvent {
      user_id: user_id.into(),
      tweet_id: tweet_id.into(),
      event_time_ms: event_time_ms.into(),
    }
  }
}

impl TSerializable for ReportTweetEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ReportTweetEvent> {
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
    let ret = ReportTweetEvent {
      user_id: f_1,
      tweet_id: f_2,
      event_time_ms: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ReportTweetEvent");
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
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DismissTweetEvent {
  pub user_id: Option<i64>,
  pub tweet_id: Option<i64>,
}

impl DismissTweetEvent {
  pub fn new<F1, F2>(user_id: F1, tweet_id: F2) -> DismissTweetEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>> {
    DismissTweetEvent {
      user_id: user_id.into(),
      tweet_id: tweet_id.into(),
    }
  }
}

impl TSerializable for DismissTweetEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DismissTweetEvent> {
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
    let ret = DismissTweetEvent {
      user_id: f_1,
      tweet_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DismissTweetEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("userId", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweetId", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FeedbackEvent {
  pub user_id: i64,
  pub ttl_ms: Option<i64>,
  pub feedback_text: Option<String>,
}

impl FeedbackEvent {
  pub fn new<F9, F10>(user_id: i64, ttl_ms: F9, feedback_text: F10) -> FeedbackEvent where F9: Into<Option<i64>>, F10: Into<Option<String>> {
    FeedbackEvent {
      user_id,
      ttl_ms: ttl_ms.into(),
      feedback_text: feedback_text.into(),
    }
  }
}

impl TSerializable for FeedbackEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FeedbackEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_9: Option<i64> = None;
    let mut f_10: Option<String> = None;
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
        9 => {
          let val = i_prot.read_i64()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_string()?;
          f_10 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("FeedbackEvent.user_id", &f_1)?;
    let ret = FeedbackEvent {
      user_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      ttl_ms: f_9,
      feedback_text: f_10,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FeedbackEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
    o_prot.write_i64(self.user_id)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.ttl_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("ttlMs", TType::I64, 9))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.feedback_text {
      o_prot.write_field_begin(&TFieldIdentifier::new("feedback_text", TType::String, 10))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GenericEvent {
  pub subject_id: i64,
  pub object_id: i64,
  pub indirect_object_key: i64,
  pub is_deleted: bool,
  pub event_time_ms: i64,
}

impl GenericEvent {
  pub fn new(subject_id: i64, object_id: i64, indirect_object_key: i64, is_deleted: bool, event_time_ms: i64) -> GenericEvent {
    GenericEvent {
      subject_id,
      object_id,
      indirect_object_key,
      is_deleted,
      event_time_ms,
    }
  }
}

impl TSerializable for GenericEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<GenericEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
    let mut f_4: Option<bool> = None;
    let mut f_5: Option<i64> = None;
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
          let val = i_prot.read_bool()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_i64()?;
          f_5 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("GenericEvent.subject_id", &f_1)?;
    verify_required_field_exists("GenericEvent.object_id", &f_2)?;
    verify_required_field_exists("GenericEvent.indirect_object_key", &f_3)?;
    verify_required_field_exists("GenericEvent.is_deleted", &f_4)?;
    verify_required_field_exists("GenericEvent.event_time_ms", &f_5)?;
    let ret = GenericEvent {
      subject_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      object_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      indirect_object_key: f_3.expect("auto-generated code should have checked for presence of required fields"),
      is_deleted: f_4.expect("auto-generated code should have checked for presence of required fields"),
      event_time_ms: f_5.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("GenericEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("subject_id", TType::I64, 1))?;
    o_prot.write_i64(self.subject_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("object_id", TType::I64, 2))?;
    o_prot.write_i64(self.object_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("indirect_object_key", TType::I64, 3))?;
    o_prot.write_i64(self.indirect_object_key)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("is_deleted", TType::Bool, 4))?;
    o_prot.write_bool(self.is_deleted)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 5))?;
    o_prot.write_i64(self.event_time_ms)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnmentionFromConversationEvent {
  pub user_id: i64,
  pub conversation_id: i64,
  pub event_time_ms: i64,
}

impl UnmentionFromConversationEvent {
  pub fn new(user_id: i64, conversation_id: i64, event_time_ms: i64) -> UnmentionFromConversationEvent {
    UnmentionFromConversationEvent {
      user_id,
      conversation_id,
      event_time_ms,
    }
  }
}

impl TSerializable for UnmentionFromConversationEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UnmentionFromConversationEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
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
    verify_required_field_exists("UnmentionFromConversationEvent.user_id", &f_1)?;
    verify_required_field_exists("UnmentionFromConversationEvent.conversation_id", &f_2)?;
    verify_required_field_exists("UnmentionFromConversationEvent.event_time_ms", &f_3)?;
    let ret = UnmentionFromConversationEvent {
      user_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      event_time_ms: f_3.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UnmentionFromConversationEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
    o_prot.write_i64(self.user_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
    o_prot.write_i64(self.event_time_ms)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UndoUnmentionFromConversationEvent {
  pub user_id: i64,
  pub conversation_id: i64,
  pub event_time_ms: i64,
}

impl UndoUnmentionFromConversationEvent {
  pub fn new(user_id: i64, conversation_id: i64, event_time_ms: i64) -> UndoUnmentionFromConversationEvent {
    UndoUnmentionFromConversationEvent {
      user_id,
      conversation_id,
      event_time_ms,
    }
  }
}

impl TSerializable for UndoUnmentionFromConversationEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UndoUnmentionFromConversationEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
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
    verify_required_field_exists("UndoUnmentionFromConversationEvent.user_id", &f_1)?;
    verify_required_field_exists("UndoUnmentionFromConversationEvent.conversation_id", &f_2)?;
    verify_required_field_exists("UndoUnmentionFromConversationEvent.event_time_ms", &f_3)?;
    let ret = UndoUnmentionFromConversationEvent {
      user_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      event_time_ms: f_3.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UndoUnmentionFromConversationEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
    o_prot.write_i64(self.user_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
    o_prot.write_i64(self.event_time_ms)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MuteConversationEvent {
  pub user_id: i64,
  pub conversation_id: i64,
  pub event_time_ms: i64,
}

impl MuteConversationEvent {
  pub fn new(user_id: i64, conversation_id: i64, event_time_ms: i64) -> MuteConversationEvent {
    MuteConversationEvent {
      user_id,
      conversation_id,
      event_time_ms,
    }
  }
}

impl TSerializable for MuteConversationEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MuteConversationEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
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
    verify_required_field_exists("MuteConversationEvent.user_id", &f_1)?;
    verify_required_field_exists("MuteConversationEvent.conversation_id", &f_2)?;
    verify_required_field_exists("MuteConversationEvent.event_time_ms", &f_3)?;
    let ret = MuteConversationEvent {
      user_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      event_time_ms: f_3.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MuteConversationEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
    o_prot.write_i64(self.user_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
    o_prot.write_i64(self.event_time_ms)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnmuteConversationEvent {
  pub user_id: i64,
  pub conversation_id: i64,
  pub event_time_ms: i64,
}

impl UnmuteConversationEvent {
  pub fn new(user_id: i64, conversation_id: i64, event_time_ms: i64) -> UnmuteConversationEvent {
    UnmuteConversationEvent {
      user_id,
      conversation_id,
      event_time_ms,
    }
  }
}

impl TSerializable for UnmuteConversationEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UnmuteConversationEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
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
    verify_required_field_exists("UnmuteConversationEvent.user_id", &f_1)?;
    verify_required_field_exists("UnmuteConversationEvent.conversation_id", &f_2)?;
    verify_required_field_exists("UnmuteConversationEvent.event_time_ms", &f_3)?;
    let ret = UnmuteConversationEvent {
      user_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      conversation_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      event_time_ms: f_3.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UnmuteConversationEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
    o_prot.write_i64(self.user_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("conversation_id", TType::I64, 2))?;
    o_prot.write_i64(self.conversation_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 3))?;
    o_prot.write_i64(self.event_time_ms)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeferredFavoriteEvent {
  pub favorite_event: FavoriteEvent,
}

impl DeferredFavoriteEvent {
  pub fn new(favorite_event: FavoriteEvent) -> DeferredFavoriteEvent {
    DeferredFavoriteEvent {
      favorite_event,
    }
  }
}

impl TSerializable for DeferredFavoriteEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DeferredFavoriteEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<FavoriteEvent> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = FavoriteEvent::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("DeferredFavoriteEvent.favorite_event", &f_1)?;
    let ret = DeferredFavoriteEvent {
      favorite_event: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DeferredFavoriteEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("favorite_event", TType::Struct, 1))?;
    self.favorite_event.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeferredUnfavoriteEvent {
  pub unfavorite_event: UnfavoriteEvent,
}

impl DeferredUnfavoriteEvent {
  pub fn new(unfavorite_event: UnfavoriteEvent) -> DeferredUnfavoriteEvent {
    DeferredUnfavoriteEvent {
      unfavorite_event,
    }
  }
}

impl TSerializable for DeferredUnfavoriteEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DeferredUnfavoriteEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<UnfavoriteEvent> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = UnfavoriteEvent::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("DeferredUnfavoriteEvent.unfavorite_event", &f_1)?;
    let ret = DeferredUnfavoriteEvent {
      unfavorite_event: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DeferredUnfavoriteEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("unfavorite_event", TType::Struct, 1))?;
    self.unfavorite_event.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RemediateFavoriteGraphEvent {
  pub user_id: i64,
  pub tweet_id: i64,
  pub tweet_user_id: i64,
  pub event_time_ms: i64,
  pub remediation_type: RemediateFavoriteGraphType,
}

impl RemediateFavoriteGraphEvent {
  pub fn new(user_id: i64, tweet_id: i64, tweet_user_id: i64, event_time_ms: i64, remediation_type: RemediateFavoriteGraphType) -> RemediateFavoriteGraphEvent {
    RemediateFavoriteGraphEvent {
      user_id,
      tweet_id,
      tweet_user_id,
      event_time_ms,
      remediation_type,
    }
  }
}

impl TSerializable for RemediateFavoriteGraphEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<RemediateFavoriteGraphEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
    let mut f_4: Option<i64> = None;
    let mut f_5: Option<RemediateFavoriteGraphType> = None;
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
          let val = RemediateFavoriteGraphType::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("RemediateFavoriteGraphEvent.user_id", &f_1)?;
    verify_required_field_exists("RemediateFavoriteGraphEvent.tweet_id", &f_2)?;
    verify_required_field_exists("RemediateFavoriteGraphEvent.tweet_user_id", &f_3)?;
    verify_required_field_exists("RemediateFavoriteGraphEvent.event_time_ms", &f_4)?;
    verify_required_field_exists("RemediateFavoriteGraphEvent.remediation_type", &f_5)?;
    let ret = RemediateFavoriteGraphEvent {
      user_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      tweet_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      tweet_user_id: f_3.expect("auto-generated code should have checked for presence of required fields"),
      event_time_ms: f_4.expect("auto-generated code should have checked for presence of required fields"),
      remediation_type: f_5.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("RemediateFavoriteGraphEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
    o_prot.write_i64(self.user_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("tweet_id", TType::I64, 2))?;
    o_prot.write_i64(self.tweet_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user_id", TType::I64, 3))?;
    o_prot.write_i64(self.tweet_user_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 4))?;
    o_prot.write_i64(self.event_time_ms)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("remediation_type", TType::I32, 5))?;
    self.remediation_type.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BookmarkEvent {
  pub user_id: Option<i64>,
  pub tweet_id: Option<i64>,
  pub tweet_user_id: Option<i64>,
  pub event_time_ms: Option<i64>,
  pub retweet_id: Option<i64>,
  pub user: Option<user::User>,
  pub tweet_user: Option<user::User>,
  pub tweet: Option<tweet::Tweet>,
  pub in_reply_to_tweet_id: Option<i64>,
  pub quoted_tweet_id: Option<i64>,
}

impl BookmarkEvent {
  pub fn new<F1, F2, F3, F4, F5, F6, F8, F11, F15, F16>(user_id: F1, tweet_id: F2, tweet_user_id: F3, event_time_ms: F4, retweet_id: F5, user: F6, tweet_user: F8, tweet: F11, in_reply_to_tweet_id: F15, quoted_tweet_id: F16) -> BookmarkEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<i64>>, F6: Into<Option<user::User>>, F8: Into<Option<user::User>>, F11: Into<Option<tweet::Tweet>>, F15: Into<Option<i64>>, F16: Into<Option<i64>> {
    BookmarkEvent {
      user_id: user_id.into(),
      tweet_id: tweet_id.into(),
      tweet_user_id: tweet_user_id.into(),
      event_time_ms: event_time_ms.into(),
      retweet_id: retweet_id.into(),
      user: user.into(),
      tweet_user: tweet_user.into(),
      tweet: tweet.into(),
      in_reply_to_tweet_id: in_reply_to_tweet_id.into(),
      quoted_tweet_id: quoted_tweet_id.into(),
    }
  }
}

impl TSerializable for BookmarkEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<BookmarkEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<user::User> = None;
    let mut f_8: Option<user::User> = None;
    let mut f_11: Option<tweet::Tweet> = None;
    let mut f_15: Option<i64> = None;
    let mut f_16: Option<i64> = None;
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
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_6 = Some(val);
        },
        8 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_8 = Some(val);
        },
        11 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_11 = Some(val);
        },
        15 => {
          let val = i_prot.read_i64()?;
          f_15 = Some(val);
        },
        16 => {
          let val = i_prot.read_i64()?;
          f_16 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = BookmarkEvent {
      user_id: f_1,
      tweet_id: f_2,
      tweet_user_id: f_3,
      event_time_ms: f_4,
      retweet_id: f_5,
      user: f_6,
      tweet_user: f_8,
      tweet: f_11,
      in_reply_to_tweet_id: f_15,
      quoted_tweet_id: f_16,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("BookmarkEvent");
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
    if let Some(fld_var) = self.tweet_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.retweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweet_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user {
      o_prot.write_field_begin(&TFieldIdentifier::new("user", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.tweet_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user", TType::Struct, 8))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 11))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.in_reply_to_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("in_reply_to_tweet_id", TType::I64, 15))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoted_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet_id", TType::I64, 16))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnbookmarkEvent {
  pub user_id: Option<i64>,
  pub tweet_id: Option<i64>,
  pub tweet_user_id: Option<i64>,
  pub event_time_ms: Option<i64>,
  pub retweet_id: Option<i64>,
  pub user: Option<user::User>,
  pub tweet_user: Option<user::User>,
  pub tweet: Option<tweet::Tweet>,
  pub in_reply_to_tweet_id: Option<i64>,
  pub quoted_tweet_id: Option<i64>,
}

impl UnbookmarkEvent {
  pub fn new<F1, F2, F3, F4, F5, F6, F8, F10, F13, F14>(user_id: F1, tweet_id: F2, tweet_user_id: F3, event_time_ms: F4, retweet_id: F5, user: F6, tweet_user: F8, tweet: F10, in_reply_to_tweet_id: F13, quoted_tweet_id: F14) -> UnbookmarkEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<i64>>, F6: Into<Option<user::User>>, F8: Into<Option<user::User>>, F10: Into<Option<tweet::Tweet>>, F13: Into<Option<i64>>, F14: Into<Option<i64>> {
    UnbookmarkEvent {
      user_id: user_id.into(),
      tweet_id: tweet_id.into(),
      tweet_user_id: tweet_user_id.into(),
      event_time_ms: event_time_ms.into(),
      retweet_id: retweet_id.into(),
      user: user.into(),
      tweet_user: tweet_user.into(),
      tweet: tweet.into(),
      in_reply_to_tweet_id: in_reply_to_tweet_id.into(),
      quoted_tweet_id: quoted_tweet_id.into(),
    }
  }
}

impl TSerializable for UnbookmarkEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UnbookmarkEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<user::User> = None;
    let mut f_8: Option<user::User> = None;
    let mut f_10: Option<tweet::Tweet> = None;
    let mut f_13: Option<i64> = None;
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
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_6 = Some(val);
        },
        8 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_8 = Some(val);
        },
        10 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_10 = Some(val);
        },
        13 => {
          let val = i_prot.read_i64()?;
          f_13 = Some(val);
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
    let ret = UnbookmarkEvent {
      user_id: f_1,
      tweet_id: f_2,
      tweet_user_id: f_3,
      event_time_ms: f_4,
      retweet_id: f_5,
      user: f_6,
      tweet_user: f_8,
      tweet: f_10,
      in_reply_to_tweet_id: f_13,
      quoted_tweet_id: f_14,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UnbookmarkEvent");
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
    if let Some(fld_var) = self.tweet_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.event_time_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("event_time_ms", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.retweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweet_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user {
      o_prot.write_field_begin(&TFieldIdentifier::new("user", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.tweet_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_user", TType::Struct, 8))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 10))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.in_reply_to_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("in_reply_to_tweet_id", TType::I64, 13))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoted_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet_id", TType::I64, 14))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeferredBookmarkEvent {
  pub bookmark_event: BookmarkEvent,
}

impl DeferredBookmarkEvent {
  pub fn new(bookmark_event: BookmarkEvent) -> DeferredBookmarkEvent {
    DeferredBookmarkEvent {
      bookmark_event,
    }
  }
}

impl TSerializable for DeferredBookmarkEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DeferredBookmarkEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<BookmarkEvent> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = BookmarkEvent::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("DeferredBookmarkEvent.bookmark_event", &f_1)?;
    let ret = DeferredBookmarkEvent {
      bookmark_event: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DeferredBookmarkEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("bookmark_event", TType::Struct, 1))?;
    self.bookmark_event.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeferredUnbookmarkEvent {
  pub unbookmark_event: UnbookmarkEvent,
}

impl DeferredUnbookmarkEvent {
  pub fn new(unbookmark_event: UnbookmarkEvent) -> DeferredUnbookmarkEvent {
    DeferredUnbookmarkEvent {
      unbookmark_event,
    }
  }
}

impl TSerializable for DeferredUnbookmarkEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DeferredUnbookmarkEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<UnbookmarkEvent> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = UnbookmarkEvent::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("DeferredUnbookmarkEvent.unbookmark_event", &f_1)?;
    let ret = DeferredUnbookmarkEvent {
      unbookmark_event: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DeferredUnbookmarkEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("unbookmark_event", TType::Struct, 1))?;
    self.unbookmark_event.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeleteAllBookmarksEvent {
  pub user_id: Option<i64>,
}

impl DeleteAllBookmarksEvent {
  pub fn new<F1>(user_id: F1) -> DeleteAllBookmarksEvent where F1: Into<Option<i64>> {
    DeleteAllBookmarksEvent {
      user_id: user_id.into(),
    }
  }
}

impl TSerializable for DeleteAllBookmarksEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DeleteAllBookmarksEvent> {
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
    let ret = DeleteAllBookmarksEvent {
      user_id: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DeleteAllBookmarksEvent");
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


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DropTweetsFromTimelineEvent {
  pub tweet_ids: Option<Vec<i64>>,
}

impl DropTweetsFromTimelineEvent {
  pub fn new<F1>(tweet_ids: F1) -> DropTweetsFromTimelineEvent where F1: Into<Option<Vec<i64>>> {
    DropTweetsFromTimelineEvent {
      tweet_ids: tweet_ids.into(),
    }
  }
}

impl TSerializable for DropTweetsFromTimelineEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DropTweetsFromTimelineEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<i64>> = Some(Vec::new());
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
            let list_elem_4 = i_prot.read_i64()?;
            val.push(list_elem_4);
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
    let ret = DropTweetsFromTimelineEvent {
      tweet_ids: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DropTweetsFromTimelineEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.tweet_ids {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet_ids", TType::List, 1))?;
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


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Event {
  Follow(FollowEvent),
  Unfollow(UnfollowEvent),
  FollowRetweets(FollowRetweetsEvent),
  UnfollowRetweets(UnfollowRetweetsEvent),
  Favorite(FavoriteEvent),
  Unfavorite(UnfavoriteEvent),
  ClearCaches(ClearCachesEvent),
  DeleteUser(DeleteUserEvent),
  ClearUserCaches(ClearUserCachesEvent),
  FullTweetCreate(FullTweetCreateEvent),
  FullTweetDelete(FullTweetDeleteEvent),
  AddListMember(AddListMemberEvent),
  RemoveListMember(RemoveListMemberEvent),
  CreateList(CreateListEvent),
  DeleteList(DeleteListEvent),
  ListPrivacyChange(ListPrivacyChangeEvent),
  CreateUser(CreateUserEvent),
  UserIdentifierConfirmation(UserIdentifierConfirmedEvent),
  ReportTweet(ReportTweetEvent),
  DismissTweet(DismissTweetEvent),
  ClearUnreadCountCaches(ClearUnreadCountCachesEvent),
  FullTweetRestore(FullTweetRestoreEvent),
  Feedback(FeedbackEvent),
  BatchFollow(BatchFollowEvent),
  GenericEvent(GenericEvent),
  MuteConversationEvent(MuteConversationEvent),
  UnmuteConversationEvent(UnmuteConversationEvent),
  DeferredFavoriteEvent(DeferredFavoriteEvent),
  DeferredUnfavoriteEvent(DeferredUnfavoriteEvent),
  UnmentionFromConversationEvent(UnmentionFromConversationEvent),
  UndoUnmentionFromConversationEvent(UndoUnmentionFromConversationEvent),
  SoftFollow(FollowEvent),
  BatchSoftFollow(BatchFollowEvent),
  SoftUnfollow(UnfollowEvent),
  RemediateFavoriteGraph(RemediateFavoriteGraphEvent),
  Bookmark(BookmarkEvent),
  Unbookmark(UnbookmarkEvent),
  DeferredBookmarkEvent(DeferredBookmarkEvent),
  DeferredUnbookmarkEvent(DeferredUnbookmarkEvent),
  DeleteAllBookmarksEvent(DeleteAllBookmarksEvent),
  DropTweetsFromTimelineEvent(DropTweetsFromTimelineEvent),
}

impl TSerializable for Event {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Event> {
    let mut ret: Option<Event> = None;
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
          let val = FollowEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::Follow(val));
          }
          received_field_count += 1;
        },
        2 => {
          let val = UnfollowEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::Unfollow(val));
          }
          received_field_count += 1;
        },
        5 => {
          let val = FollowRetweetsEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::FollowRetweets(val));
          }
          received_field_count += 1;
        },
        6 => {
          let val = UnfollowRetweetsEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::UnfollowRetweets(val));
          }
          received_field_count += 1;
        },
        7 => {
          let val = FavoriteEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::Favorite(val));
          }
          received_field_count += 1;
        },
        8 => {
          let val = UnfavoriteEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::Unfavorite(val));
          }
          received_field_count += 1;
        },
        9 => {
          let val = ClearCachesEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::ClearCaches(val));
          }
          received_field_count += 1;
        },
        10 => {
          let val = DeleteUserEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::DeleteUser(val));
          }
          received_field_count += 1;
        },
        11 => {
          let val = ClearUserCachesEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::ClearUserCaches(val));
          }
          received_field_count += 1;
        },
        12 => {
          let val = FullTweetCreateEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::FullTweetCreate(val));
          }
          received_field_count += 1;
        },
        13 => {
          let val = FullTweetDeleteEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::FullTweetDelete(val));
          }
          received_field_count += 1;
        },
        14 => {
          let val = AddListMemberEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::AddListMember(val));
          }
          received_field_count += 1;
        },
        15 => {
          let val = RemoveListMemberEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::RemoveListMember(val));
          }
          received_field_count += 1;
        },
        16 => {
          let val = CreateListEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::CreateList(val));
          }
          received_field_count += 1;
        },
        17 => {
          let val = DeleteListEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::DeleteList(val));
          }
          received_field_count += 1;
        },
        18 => {
          let val = ListPrivacyChangeEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::ListPrivacyChange(val));
          }
          received_field_count += 1;
        },
        19 => {
          let val = CreateUserEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::CreateUser(val));
          }
          received_field_count += 1;
        },
        20 => {
          let val = UserIdentifierConfirmedEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::UserIdentifierConfirmation(val));
          }
          received_field_count += 1;
        },
        21 => {
          let val = ReportTweetEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::ReportTweet(val));
          }
          received_field_count += 1;
        },
        24 => {
          let val = DismissTweetEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::DismissTweet(val));
          }
          received_field_count += 1;
        },
        25 => {
          let val = ClearUnreadCountCachesEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::ClearUnreadCountCaches(val));
          }
          received_field_count += 1;
        },
        26 => {
          let val = FullTweetRestoreEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::FullTweetRestore(val));
          }
          received_field_count += 1;
        },
        28 => {
          let val = FeedbackEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::Feedback(val));
          }
          received_field_count += 1;
        },
        31 => {
          let val = BatchFollowEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::BatchFollow(val));
          }
          received_field_count += 1;
        },
        32 => {
          let val = GenericEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::GenericEvent(val));
          }
          received_field_count += 1;
        },
        33 => {
          let val = MuteConversationEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::MuteConversationEvent(val));
          }
          received_field_count += 1;
        },
        34 => {
          let val = UnmuteConversationEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::UnmuteConversationEvent(val));
          }
          received_field_count += 1;
        },
        35 => {
          let val = DeferredFavoriteEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::DeferredFavoriteEvent(val));
          }
          received_field_count += 1;
        },
        36 => {
          let val = DeferredUnfavoriteEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::DeferredUnfavoriteEvent(val));
          }
          received_field_count += 1;
        },
        37 => {
          let val = UnmentionFromConversationEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::UnmentionFromConversationEvent(val));
          }
          received_field_count += 1;
        },
        38 => {
          let val = UndoUnmentionFromConversationEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::UndoUnmentionFromConversationEvent(val));
          }
          received_field_count += 1;
        },
        39 => {
          let val = FollowEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::SoftFollow(val));
          }
          received_field_count += 1;
        },
        40 => {
          let val = BatchFollowEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::BatchSoftFollow(val));
          }
          received_field_count += 1;
        },
        41 => {
          let val = UnfollowEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::SoftUnfollow(val));
          }
          received_field_count += 1;
        },
        42 => {
          let val = RemediateFavoriteGraphEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::RemediateFavoriteGraph(val));
          }
          received_field_count += 1;
        },
        43 => {
          let val = BookmarkEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::Bookmark(val));
          }
          received_field_count += 1;
        },
        44 => {
          let val = UnbookmarkEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::Unbookmark(val));
          }
          received_field_count += 1;
        },
        45 => {
          let val = DeferredBookmarkEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::DeferredBookmarkEvent(val));
          }
          received_field_count += 1;
        },
        46 => {
          let val = DeferredUnbookmarkEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::DeferredUnbookmarkEvent(val));
          }
          received_field_count += 1;
        },
        47 => {
          let val = DeleteAllBookmarksEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::DeleteAllBookmarksEvent(val));
          }
          received_field_count += 1;
        },
        48 => {
          let val = DropTweetsFromTimelineEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(Event::DropTweetsFromTimelineEvent(val));
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
            "received empty union from remote Event"
          )
        )
      )
    } else if received_field_count > 1 {
      Err(
        thrift::Error::Protocol(
          ProtocolError::new(
            ProtocolErrorKind::InvalidData,
            "received multiple fields for union from remote Event"
          )
        )
      )
    } else {
      Ok(ret.expect("return value should have been constructed"))
    }
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Event");
    o_prot.write_struct_begin(&struct_ident)?;
    match *self {
      Event::Follow(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("follow", TType::Struct, 1))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::Unfollow(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("unfollow", TType::Struct, 2))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::FollowRetweets(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("follow_retweets", TType::Struct, 5))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::UnfollowRetweets(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("unfollow_retweets", TType::Struct, 6))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::Favorite(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("favorite", TType::Struct, 7))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::Unfavorite(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("unfavorite", TType::Struct, 8))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::ClearCaches(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("clear_caches", TType::Struct, 9))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::DeleteUser(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("delete_user", TType::Struct, 10))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::ClearUserCaches(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("clear_user_caches", TType::Struct, 11))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::FullTweetCreate(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("full_tweet_create", TType::Struct, 12))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::FullTweetDelete(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("full_tweet_delete", TType::Struct, 13))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::AddListMember(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("add_list_member", TType::Struct, 14))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::RemoveListMember(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("remove_list_member", TType::Struct, 15))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::CreateList(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("create_list", TType::Struct, 16))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::DeleteList(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("delete_list", TType::Struct, 17))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::ListPrivacyChange(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("list_privacy_change", TType::Struct, 18))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::CreateUser(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("create_user", TType::Struct, 19))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::UserIdentifierConfirmation(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("user_identifier_confirmation", TType::Struct, 20))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::ReportTweet(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("report_tweet", TType::Struct, 21))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::DismissTweet(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("dismiss_tweet", TType::Struct, 24))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::ClearUnreadCountCaches(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("clear_unread_count_caches", TType::Struct, 25))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::FullTweetRestore(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("full_tweet_restore", TType::Struct, 26))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::Feedback(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("feedback", TType::Struct, 28))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::BatchFollow(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("batch_follow", TType::Struct, 31))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::GenericEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("generic_event", TType::Struct, 32))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::MuteConversationEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("mute_conversation_event", TType::Struct, 33))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::UnmuteConversationEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("unmute_conversation_event", TType::Struct, 34))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::DeferredFavoriteEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("deferred_favorite_event", TType::Struct, 35))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::DeferredUnfavoriteEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("deferred_unfavorite_event", TType::Struct, 36))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::UnmentionFromConversationEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("unmention_from_conversation_event", TType::Struct, 37))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::UndoUnmentionFromConversationEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("undo_unmention_from_conversation_event", TType::Struct, 38))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::SoftFollow(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("softFollow", TType::Struct, 39))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::BatchSoftFollow(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("batch_soft_follow", TType::Struct, 40))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::SoftUnfollow(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("softUnfollow", TType::Struct, 41))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::RemediateFavoriteGraph(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("remediateFavoriteGraph", TType::Struct, 42))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::Bookmark(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("bookmark", TType::Struct, 43))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::Unbookmark(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("unbookmark", TType::Struct, 44))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::DeferredBookmarkEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("deferred_bookmark_event", TType::Struct, 45))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::DeferredUnbookmarkEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("deferred_unbookmark_event", TType::Struct, 46))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::DeleteAllBookmarksEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("delete_all_bookmarks_event", TType::Struct, 47))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      Event::DropTweetsFromTimelineEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("drop_tweets_from_timeline_event", TType::Struct, 48))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

