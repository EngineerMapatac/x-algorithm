
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
pub struct SafetyType(pub i32);

impl SafetyType {
  pub const PRIVATE: SafetyType = SafetyType(0);
  pub const RESTRICTED: SafetyType = SafetyType(1);
  pub const PUBLIC: SafetyType = SafetyType(2);
  pub const RESERVED0: SafetyType = SafetyType(3);
  pub const RESERVED1: SafetyType = SafetyType(4);
  pub const RESERVED2: SafetyType = SafetyType(5);
  pub const RESERVED3: SafetyType = SafetyType(6);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PRIVATE,
    Self::RESTRICTED,
    Self::PUBLIC,
    Self::RESERVED0,
    Self::RESERVED1,
    Self::RESERVED2,
    Self::RESERVED3,
  ];
}

impl TSerializable for SafetyType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SafetyType> {
    let enum_value = i_prot.read_i32()?;
    Ok(SafetyType::from(enum_value))
  }
}

impl From<i32> for SafetyType {
  fn from(i: i32) -> Self {
    match i {
      0 => SafetyType::PRIVATE,
      1 => SafetyType::RESTRICTED,
      2 => SafetyType::PUBLIC,
      3 => SafetyType::RESERVED0,
      4 => SafetyType::RESERVED1,
      5 => SafetyType::RESERVED2,
      6 => SafetyType::RESERVED3,
      _ => SafetyType(i)
    }
  }
}

impl From<&i32> for SafetyType {
  fn from(i: &i32) -> Self {
    SafetyType::from(*i)
  }
}

impl From<SafetyType> for i32 {
  fn from(e: SafetyType) -> i32 {
    e.0
  }
}

impl From<&SafetyType> for i32 {
  fn from(e: &SafetyType) -> i32 {
    e.0
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetCreateEvent {
    pub tweet: Option<tweet::Tweet>,
    pub user: Option<user::User>,
    pub source_tweet: Option<tweet::Tweet>,
    pub source_user: Option<user::User>,
                pub retweet_parent_user_id: Option<i64>,
    pub quoted_tweet: Option<tweet::Tweet>,
    pub quoted_user: Option<user::User>,
          pub additional_context: Option<BTreeMap<tweet::TweetCreateContextKey, String>>,
        pub quoter_has_already_quoted_tweet: Option<bool>,
}

impl TweetCreateEvent {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F10>(tweet: F1, user: F2, source_tweet: F3, source_user: F4, retweet_parent_user_id: F5, quoted_tweet: F6, quoted_user: F7, additional_context: F8, quoter_has_already_quoted_tweet: F10) -> TweetCreateEvent where F1: Into<Option<tweet::Tweet>>, F2: Into<Option<user::User>>, F3: Into<Option<tweet::Tweet>>, F4: Into<Option<user::User>>, F5: Into<Option<i64>>, F6: Into<Option<tweet::Tweet>>, F7: Into<Option<user::User>>, F8: Into<Option<BTreeMap<tweet::TweetCreateContextKey, String>>>, F10: Into<Option<bool>> {
    TweetCreateEvent {
      tweet: tweet.into(),
      user: user.into(),
      source_tweet: source_tweet.into(),
      source_user: source_user.into(),
      retweet_parent_user_id: retweet_parent_user_id.into(),
      quoted_tweet: quoted_tweet.into(),
      quoted_user: quoted_user.into(),
      additional_context: additional_context.into(),
      quoter_has_already_quoted_tweet: quoter_has_already_quoted_tweet.into(),
    }
  }
}

impl TSerializable for TweetCreateEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetCreateEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<tweet::Tweet> = None;
    let mut f_2: Option<user::User> = None;
    let mut f_3: Option<tweet::Tweet> = None;
    let mut f_4: Option<user::User> = None;
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<tweet::Tweet> = None;
    let mut f_7: Option<user::User> = None;
    let mut f_8: Option<BTreeMap<tweet::TweetCreateContextKey, String>> = None;
    let mut f_10: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_i64()?;
          f_5 = Some(val);
        },
        6 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_6 = Some(val);
        },
        7 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_7 = Some(val);
        },
        8 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<tweet::TweetCreateContextKey, String> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_0 = tweet::TweetCreateContextKey::read_from_in_protocol(i_prot)?;
            let map_val_1 = i_prot.read_string()?;
            val.insert(map_key_0, map_val_1);
          }
          i_prot.read_map_end()?;
          f_8 = Some(val);
        },
        10 => {
          let val = i_prot.read_bool()?;
          f_10 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TweetCreateEvent {
      tweet: f_1,
      user: f_2,
      source_tweet: f_3,
      source_user: f_4,
      retweet_parent_user_id: f_5,
      quoted_tweet: f_6,
      quoted_user: f_7,
      additional_context: f_8,
      quoter_has_already_quoted_tweet: f_10,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetCreateEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user {
      o_prot.write_field_begin(&TFieldIdentifier::new("user", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.source_tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_tweet", TType::Struct, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.source_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_user", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.retweet_parent_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweet_parent_user_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.quoted_tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.quoted_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_user", TType::Struct, 7))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.additional_context {
      o_prot.write_field_begin(&TFieldIdentifier::new("additional_context", TType::Map, 8))?;
      o_prot.write_map_begin(&TMapIdentifier::new(TType::I32, TType::String, fld_var.len() as i32))?;
      for (k, v) in fld_var {
        k.write_to_out_protocol(o_prot)?;
        o_prot.write_string(v)?;
      }
      o_prot.write_map_end()?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoter_has_already_quoted_tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoter_has_already_quoted_tweet", TType::Bool, 10))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetDeleteEvent {
    pub tweet: Option<tweet::Tweet>,
    pub user: Option<user::User>,
            pub is_user_erasure: Option<bool>,
        pub by_user_id: Option<i64>,
        pub is_admin_delete: Option<bool>,
}

impl TweetDeleteEvent {
  pub fn new<F1, F2, F3, F5, F6>(tweet: F1, user: F2, is_user_erasure: F3, by_user_id: F5, is_admin_delete: F6) -> TweetDeleteEvent where F1: Into<Option<tweet::Tweet>>, F2: Into<Option<user::User>>, F3: Into<Option<bool>>, F5: Into<Option<i64>>, F6: Into<Option<bool>> {
    TweetDeleteEvent {
      tweet: tweet.into(),
      user: user.into(),
      is_user_erasure: is_user_erasure.into(),
      by_user_id: by_user_id.into(),
      is_admin_delete: is_admin_delete.into(),
    }
  }
}

impl TSerializable for TweetDeleteEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetDeleteEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<tweet::Tweet> = None;
    let mut f_2: Option<user::User> = None;
    let mut f_3: Option<bool> = None;
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
          f_3 = Some(val);
        },
        5 => {
          let val = i_prot.read_i64()?;
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
    let ret = TweetDeleteEvent {
      tweet: f_1,
      user: f_2,
      is_user_erasure: f_3,
      by_user_id: f_5,
      is_admin_delete: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetDeleteEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user {
      o_prot.write_field_begin(&TFieldIdentifier::new("user", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_user_erasure {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_user_erasure", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.by_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("by_user_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_admin_delete {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_admin_delete", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetUndeleteEvent {
  pub tweet: Option<tweet::Tweet>,
  pub user: Option<user::User>,
  pub source_tweet: Option<tweet::Tweet>,
  pub source_user: Option<user::User>,
  pub retweet_parent_user_id: Option<i64>,
  pub quoted_tweet: Option<tweet::Tweet>,
  pub quoted_user: Option<user::User>,
  pub deleted_at_msec: Option<i64>,
}

impl TweetUndeleteEvent {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8>(tweet: F1, user: F2, source_tweet: F3, source_user: F4, retweet_parent_user_id: F5, quoted_tweet: F6, quoted_user: F7, deleted_at_msec: F8) -> TweetUndeleteEvent where F1: Into<Option<tweet::Tweet>>, F2: Into<Option<user::User>>, F3: Into<Option<tweet::Tweet>>, F4: Into<Option<user::User>>, F5: Into<Option<i64>>, F6: Into<Option<tweet::Tweet>>, F7: Into<Option<user::User>>, F8: Into<Option<i64>> {
    TweetUndeleteEvent {
      tweet: tweet.into(),
      user: user.into(),
      source_tweet: source_tweet.into(),
      source_user: source_user.into(),
      retweet_parent_user_id: retweet_parent_user_id.into(),
      quoted_tweet: quoted_tweet.into(),
      quoted_user: quoted_user.into(),
      deleted_at_msec: deleted_at_msec.into(),
    }
  }
}

impl TSerializable for TweetUndeleteEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetUndeleteEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<tweet::Tweet> = None;
    let mut f_2: Option<user::User> = None;
    let mut f_3: Option<tweet::Tweet> = None;
    let mut f_4: Option<user::User> = None;
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<tweet::Tweet> = None;
    let mut f_7: Option<user::User> = None;
    let mut f_8: Option<i64> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_i64()?;
          f_5 = Some(val);
        },
        6 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
          f_6 = Some(val);
        },
        7 => {
          let val = user::User::read_from_in_protocol(i_prot)?;
          f_7 = Some(val);
        },
        8 => {
          let val = i_prot.read_i64()?;
          f_8 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TweetUndeleteEvent {
      tweet: f_1,
      user: f_2,
      source_tweet: f_3,
      source_user: f_4,
      retweet_parent_user_id: f_5,
      quoted_tweet: f_6,
      quoted_user: f_7,
      deleted_at_msec: f_8,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetUndeleteEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweet", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user {
      o_prot.write_field_begin(&TFieldIdentifier::new("user", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.source_tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_tweet", TType::Struct, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.source_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("source_user", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.retweet_parent_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("retweet_parent_user_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.quoted_tweet {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.quoted_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_user", TType::Struct, 7))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.deleted_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("deleted_at_msec", TType::I64, 8))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetScrubGeoEvent {
  pub tweet_id: Option<i64>,
  pub user_id: Option<i64>,
}

impl TweetScrubGeoEvent {
  pub fn new<F1, F2>(tweet_id: F1, user_id: F2) -> TweetScrubGeoEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>> {
    TweetScrubGeoEvent {
      tweet_id: tweet_id.into(),
      user_id: user_id.into(),
    }
  }
}

impl TSerializable for TweetScrubGeoEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetScrubGeoEvent> {
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
    let ret = TweetScrubGeoEvent {
      tweet_id: f_1,
      user_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetScrubGeoEvent");
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
pub struct UserScrubGeoEvent {
  pub user_id: Option<i64>,
  pub max_tweet_id: Option<i64>,
}

impl UserScrubGeoEvent {
  pub fn new<F1, F2>(user_id: F1, max_tweet_id: F2) -> UserScrubGeoEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>> {
    UserScrubGeoEvent {
      user_id: user_id.into(),
      max_tweet_id: max_tweet_id.into(),
    }
  }
}

impl TSerializable for UserScrubGeoEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UserScrubGeoEvent> {
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
    let ret = UserScrubGeoEvent {
      user_id: f_1,
      max_tweet_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UserScrubGeoEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.max_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("max_tweet_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetTakedownEvent {
  pub tweet_id: Option<i64>,
  pub user_id: Option<i64>,
  pub takedown_country_codes: Option<Vec<String>>,
}

impl TweetTakedownEvent {
  pub fn new<F1, F2, F3>(tweet_id: F1, user_id: F2, takedown_country_codes: F3) -> TweetTakedownEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<Vec<String>>> {
    TweetTakedownEvent {
      tweet_id: tweet_id.into(),
      user_id: user_id.into(),
      takedown_country_codes: takedown_country_codes.into(),
    }
  }
}

impl TSerializable for TweetTakedownEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetTakedownEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<Vec<String>> = Some(Vec::new());
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
          let mut val: Vec<String> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_2 = i_prot.read_string()?;
            val.push(list_elem_2);
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
    let ret = TweetTakedownEvent {
      tweet_id: f_1,
      user_id: f_2,
      takedown_country_codes: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetTakedownEvent");
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
    if let Some(ref fld_var) = self.takedown_country_codes {
      o_prot.write_field_begin(&TFieldIdentifier::new("takedown_country_codes", TType::List, 3))?;
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
pub struct AdditionalFieldUpdateEvent {
  pub updated_fields: Option<tweet::Tweet>,
  pub user_id: Option<i64>,
}

impl AdditionalFieldUpdateEvent {
  pub fn new<F1, F2>(updated_fields: F1, user_id: F2) -> AdditionalFieldUpdateEvent where F1: Into<Option<tweet::Tweet>>, F2: Into<Option<i64>> {
    AdditionalFieldUpdateEvent {
      updated_fields: updated_fields.into(),
      user_id: user_id.into(),
    }
  }
}

impl TSerializable for AdditionalFieldUpdateEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AdditionalFieldUpdateEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<tweet::Tweet> = None;
    let mut f_2: Option<i64> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = tweet::Tweet::read_from_in_protocol(i_prot)?;
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
    let ret = AdditionalFieldUpdateEvent {
      updated_fields: f_1,
      user_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AdditionalFieldUpdateEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.updated_fields {
      o_prot.write_field_begin(&TFieldIdentifier::new("updated_fields", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
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
pub struct AdditionalFieldDeleteEvent {
  pub deleted_fields: Option<BTreeMap<i64, Vec<i16>>>,
  pub user_id: Option<i64>,
}

impl AdditionalFieldDeleteEvent {
  pub fn new<F1, F2>(deleted_fields: F1, user_id: F2) -> AdditionalFieldDeleteEvent where F1: Into<Option<BTreeMap<i64, Vec<i16>>>>, F2: Into<Option<i64>> {
    AdditionalFieldDeleteEvent {
      deleted_fields: deleted_fields.into(),
      user_id: user_id.into(),
    }
  }
}

impl TSerializable for AdditionalFieldDeleteEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AdditionalFieldDeleteEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<BTreeMap<i64, Vec<i16>>> = Some(BTreeMap::new());
    let mut f_2: Option<i64> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<i64, Vec<i16>> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_3 = i_prot.read_i64()?;
            let list_ident = i_prot.read_list_begin()?;
            let mut map_val_4: Vec<i16> = Vec::with_capacity(list_ident.size as usize);
            for _ in 0..list_ident.size {
              let list_elem_5 = i_prot.read_i16()?;
              map_val_4.push(list_elem_5);
            }
            i_prot.read_list_end()?;
            val.insert(map_key_3, map_val_4);
          }
          i_prot.read_map_end()?;
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
    let ret = AdditionalFieldDeleteEvent {
      deleted_fields: f_1,
      user_id: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AdditionalFieldDeleteEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.deleted_fields {
      o_prot.write_field_begin(&TFieldIdentifier::new("deleted_fields", TType::Map, 1))?;
      o_prot.write_map_begin(&TMapIdentifier::new(TType::I64, TType::List, fld_var.len() as i32))?;
      for (k, v) in fld_var {
        o_prot.write_i64(*k)?;
        o_prot.write_list_begin(&TListIdentifier::new(TType::I16, v.len() as i32))?;
        for e in v {
          o_prot.write_i16(*e)?;
        }
        o_prot.write_list_end()?;
      }
      o_prot.write_map_end()?;
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
pub struct TweetMediaTagEvent {
  pub tweet_id: Option<i64>,
  pub user_id: Option<i64>,
  pub tagged_user_ids: Option<BTreeSet<i64>>,
  pub timestamp_ms: Option<i64>,
}

impl TweetMediaTagEvent {
  pub fn new<F1, F2, F3, F4>(tweet_id: F1, user_id: F2, tagged_user_ids: F3, timestamp_ms: F4) -> TweetMediaTagEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<BTreeSet<i64>>>, F4: Into<Option<i64>> {
    TweetMediaTagEvent {
      tweet_id: tweet_id.into(),
      user_id: user_id.into(),
      tagged_user_ids: tagged_user_ids.into(),
      timestamp_ms: timestamp_ms.into(),
    }
  }
}

impl TSerializable for TweetMediaTagEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetMediaTagEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<BTreeSet<i64>> = Some(BTreeSet::new());
    let mut f_4: Option<i64> = None;
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
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<i64> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_6 = i_prot.read_i64()?;
            val.insert(set_elem_6);
          }
          i_prot.read_set_end()?;
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
    let ret = TweetMediaTagEvent {
      tweet_id: f_1,
      user_id: f_2,
      tagged_user_ids: f_3,
      timestamp_ms: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetMediaTagEvent");
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
    if let Some(ref fld_var) = self.tagged_user_ids {
      o_prot.write_field_begin(&TFieldIdentifier::new("tagged_user_ids", TType::Set, 3))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::I64, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_i64(*e)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.timestamp_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("timestamp_ms", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetPossiblySensitiveUpdateEvent {
  pub tweet_id: Option<i64>,
  pub user_id: Option<i64>,
  pub nsfw_admin: Option<bool>,
  pub nsfw_user: Option<bool>,
}

impl TweetPossiblySensitiveUpdateEvent {
  pub fn new<F1, F2, F3, F4>(tweet_id: F1, user_id: F2, nsfw_admin: F3, nsfw_user: F4) -> TweetPossiblySensitiveUpdateEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<bool>>, F4: Into<Option<bool>> {
    TweetPossiblySensitiveUpdateEvent {
      tweet_id: tweet_id.into(),
      user_id: user_id.into(),
      nsfw_admin: nsfw_admin.into(),
      nsfw_user: nsfw_user.into(),
    }
  }
}

impl TSerializable for TweetPossiblySensitiveUpdateEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetPossiblySensitiveUpdateEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<bool> = Some(false);
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
    let ret = TweetPossiblySensitiveUpdateEvent {
      tweet_id: f_1,
      user_id: f_2,
      nsfw_admin: f_3,
      nsfw_user: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetPossiblySensitiveUpdateEvent");
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
    if let Some(fld_var) = self.nsfw_admin {
      o_prot.write_field_begin(&TFieldIdentifier::new("nsfw_admin", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.nsfw_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("nsfw_user", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QuotedTweetDeleteEvent {
  pub quoting_tweet_id: Option<i64>,
  pub quoting_user_id: Option<i64>,
  pub quoted_tweet_id: Option<i64>,
  pub quoted_user_id: Option<i64>,
}

impl QuotedTweetDeleteEvent {
  pub fn new<F1, F2, F3, F4>(quoting_tweet_id: F1, quoting_user_id: F2, quoted_tweet_id: F3, quoted_user_id: F4) -> QuotedTweetDeleteEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>> {
    QuotedTweetDeleteEvent {
      quoting_tweet_id: quoting_tweet_id.into(),
      quoting_user_id: quoting_user_id.into(),
      quoted_tweet_id: quoted_tweet_id.into(),
      quoted_user_id: quoted_user_id.into(),
    }
  }
}

impl TSerializable for QuotedTweetDeleteEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<QuotedTweetDeleteEvent> {
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
    let ret = QuotedTweetDeleteEvent {
      quoting_tweet_id: f_1,
      quoting_user_id: f_2,
      quoted_tweet_id: f_3,
      quoted_user_id: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("QuotedTweetDeleteEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.quoting_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoting_tweet_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoting_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoting_user_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoted_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoted_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_user_id", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QuotedTweetTakedownEvent {
  pub quoting_tweet_id: Option<i64>,
  pub quoting_user_id: Option<i64>,
  pub quoted_tweet_id: Option<i64>,
  pub quoted_user_id: Option<i64>,
  pub takedown_country_codes: Option<Vec<String>>,
}

impl QuotedTweetTakedownEvent {
  pub fn new<F1, F2, F3, F4, F5>(quoting_tweet_id: F1, quoting_user_id: F2, quoted_tweet_id: F3, quoted_user_id: F4, takedown_country_codes: F5) -> QuotedTweetTakedownEvent where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<Vec<String>>> {
    QuotedTweetTakedownEvent {
      quoting_tweet_id: quoting_tweet_id.into(),
      quoting_user_id: quoting_user_id.into(),
      quoted_tweet_id: quoted_tweet_id.into(),
      quoted_user_id: quoted_user_id.into(),
      takedown_country_codes: takedown_country_codes.into(),
    }
  }
}

impl TSerializable for QuotedTweetTakedownEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<QuotedTweetTakedownEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<Vec<String>> = Some(Vec::new());
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
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<String> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_7 = i_prot.read_string()?;
            val.push(list_elem_7);
          }
          i_prot.read_list_end()?;
          f_5 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = QuotedTweetTakedownEvent {
      quoting_tweet_id: f_1,
      quoting_user_id: f_2,
      quoted_tweet_id: f_3,
      quoted_user_id: f_4,
      takedown_country_codes: f_5,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("QuotedTweetTakedownEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.quoting_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoting_tweet_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoting_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoting_user_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoted_tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet_id", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.quoted_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("quoted_user_id", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.takedown_country_codes {
      o_prot.write_field_begin(&TFieldIdentifier::new("takedown_country_codes", TType::List, 5))?;
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


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TweetEventData {
  TweetCreateEvent(TweetCreateEvent),
  TweetDeleteEvent(TweetDeleteEvent),
  AdditionalFieldUpdateEvent(AdditionalFieldUpdateEvent),
  AdditionalFieldDeleteEvent(AdditionalFieldDeleteEvent),
  TweetUndeleteEvent(TweetUndeleteEvent),
  TweetScrubGeoEvent(TweetScrubGeoEvent),
  TweetTakedownEvent(TweetTakedownEvent),
  UserScrubGeoEvent(UserScrubGeoEvent),
  TweetPossiblySensitiveUpdateEvent(TweetPossiblySensitiveUpdateEvent),
  QuotedTweetDeleteEvent(QuotedTweetDeleteEvent),
  QuotedTweetTakedownEvent(QuotedTweetTakedownEvent),
}

impl TSerializable for TweetEventData {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetEventData> {
    let mut ret: Option<TweetEventData> = None;
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
          let val = TweetCreateEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::TweetCreateEvent(val));
          }
          received_field_count += 1;
        },
        2 => {
          let val = TweetDeleteEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::TweetDeleteEvent(val));
          }
          received_field_count += 1;
        },
        3 => {
          let val = AdditionalFieldUpdateEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::AdditionalFieldUpdateEvent(val));
          }
          received_field_count += 1;
        },
        4 => {
          let val = AdditionalFieldDeleteEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::AdditionalFieldDeleteEvent(val));
          }
          received_field_count += 1;
        },
        5 => {
          let val = TweetUndeleteEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::TweetUndeleteEvent(val));
          }
          received_field_count += 1;
        },
        6 => {
          let val = TweetScrubGeoEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::TweetScrubGeoEvent(val));
          }
          received_field_count += 1;
        },
        7 => {
          let val = TweetTakedownEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::TweetTakedownEvent(val));
          }
          received_field_count += 1;
        },
        8 => {
          let val = UserScrubGeoEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::UserScrubGeoEvent(val));
          }
          received_field_count += 1;
        },
        9 => {
          let val = TweetPossiblySensitiveUpdateEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::TweetPossiblySensitiveUpdateEvent(val));
          }
          received_field_count += 1;
        },
        10 => {
          let val = QuotedTweetDeleteEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::QuotedTweetDeleteEvent(val));
          }
          received_field_count += 1;
        },
        11 => {
          let val = QuotedTweetTakedownEvent::read_from_in_protocol(i_prot)?;
          if ret.is_none() {
            ret = Some(TweetEventData::QuotedTweetTakedownEvent(val));
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
            "received empty union from remote TweetEventData"
          )
        )
      )
    } else if received_field_count > 1 {
      Err(
        thrift::Error::Protocol(
          ProtocolError::new(
            ProtocolErrorKind::InvalidData,
            "received multiple fields for union from remote TweetEventData"
          )
        )
      )
    } else {
      Ok(ret.expect("return value should have been constructed"))
    }
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetEventData");
    o_prot.write_struct_begin(&struct_ident)?;
    match *self {
      TweetEventData::TweetCreateEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("tweet_create_event", TType::Struct, 1))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::TweetDeleteEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("tweet_delete_event", TType::Struct, 2))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::AdditionalFieldUpdateEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("additional_field_update_event", TType::Struct, 3))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::AdditionalFieldDeleteEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("additional_field_delete_event", TType::Struct, 4))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::TweetUndeleteEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("tweet_undelete_event", TType::Struct, 5))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::TweetScrubGeoEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("tweet_scrub_geo_event", TType::Struct, 6))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::TweetTakedownEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("tweet_takedown_event", TType::Struct, 7))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::UserScrubGeoEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("user_scrub_geo_event", TType::Struct, 8))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::TweetPossiblySensitiveUpdateEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("tweet_possibly_sensitive_update_event", TType::Struct, 9))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::QuotedTweetDeleteEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet_delete_event", TType::Struct, 10))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
      TweetEventData::QuotedTweetTakedownEvent(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("quoted_tweet_takedown_event", TType::Struct, 11))?;
        f.write_to_out_protocol(o_prot)?;
        o_prot.write_field_end()?;
      },
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Checksum {
  pub checksum: Option<i32>,
}

impl Checksum {
  pub fn new<F1>(checksum: F1) -> Checksum where F1: Into<Option<i32>> {
    Checksum {
      checksum: checksum.into(),
    }
  }
}

impl TSerializable for Checksum {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Checksum> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i32> = Some(0);
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Checksum {
      checksum: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Checksum");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.checksum {
      o_prot.write_field_begin(&TFieldIdentifier::new("checksum", TType::I32, 1))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetEventFlags {
    pub unused1: Option<Vec<String>>,
  pub timestamp_ms: Option<i64>,
  pub safety_type: Option<SafetyType>,
    pub unused4: Option<Checksum>,
}

impl TweetEventFlags {
  pub fn new<F1, F2, F3, F4>(unused1: F1, timestamp_ms: F2, safety_type: F3, unused4: F4) -> TweetEventFlags where F1: Into<Option<Vec<String>>>, F2: Into<Option<i64>>, F3: Into<Option<SafetyType>>, F4: Into<Option<Checksum>> {
    TweetEventFlags {
      unused1: unused1.into(),
      timestamp_ms: timestamp_ms.into(),
      safety_type: safety_type.into(),
      unused4: unused4.into(),
    }
  }
}

impl TSerializable for TweetEventFlags {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetEventFlags> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<String>> = Some(Vec::new());
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<SafetyType> = None;
    let mut f_4: Option<Checksum> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<String> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_8 = i_prot.read_string()?;
            val.push(list_elem_8);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        3 => {
          let val = SafetyType::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = Checksum::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TweetEventFlags {
      unused1: f_1,
      timestamp_ms: f_2,
      safety_type: f_3,
      unused4: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetEventFlags");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.unused1 {
      o_prot.write_field_begin(&TFieldIdentifier::new("unused1", TType::List, 1))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::String, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_string(e)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.timestamp_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("timestamp_ms", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.safety_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("safety_type", TType::I32, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.unused4 {
      o_prot.write_field_begin(&TFieldIdentifier::new("unused4", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetEvent {
  pub data: Option<TweetEventData>,
  pub flags: Option<TweetEventFlags>,
}

impl TweetEvent {
  pub fn new<F1, F2>(data: F1, flags: F2) -> TweetEvent where F1: Into<Option<TweetEventData>>, F2: Into<Option<TweetEventFlags>> {
    TweetEvent {
      data: data.into(),
      flags: flags.into(),
    }
  }
}

impl TSerializable for TweetEvent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetEvent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<TweetEventData> = None;
    let mut f_2: Option<TweetEventFlags> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = TweetEventData::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = TweetEventFlags::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TweetEvent {
      data: f_1,
      flags: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetEvent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.data {
      o_prot.write_field_begin(&TFieldIdentifier::new("data", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.flags {
      o_prot.write_field_begin(&TFieldIdentifier::new("flags", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

