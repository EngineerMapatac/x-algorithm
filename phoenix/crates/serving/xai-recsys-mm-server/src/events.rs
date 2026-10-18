// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.

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
pub struct Experiment(pub i32);

impl Experiment {
  pub const PROD: Experiment = Experiment(0);
  pub const TEST: Experiment = Experiment(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PROD,
    Self::TEST,
  ];
}

impl TSerializable for Experiment {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Experiment> {
    let enum_value = i_prot.read_i32()?;
    Ok(Experiment::from(enum_value))
  }
}

impl From<i32> for Experiment {
  fn from(i: i32) -> Self {
    match i {
      0 => Experiment::PROD,
      1 => Experiment::TEST,
      _ => Experiment(i)
    }
  }
}

impl From<&i32> for Experiment {
  fn from(i: &i32) -> Self {
    Experiment::from(*i)
  }
}

impl From<Experiment> for i32 {
  fn from(e: Experiment) -> i32 {
    e.0
  }
}

impl From<&Experiment> for i32 {
  fn from(e: &Experiment) -> i32 {
    e.0
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GrokAnalyzeTaskEntity {
  pub post_id: i64,
  pub created_at_secs: i64,
}

impl GrokAnalyzeTaskEntity {
  pub fn new(post_id: i64, created_at_secs: i64) -> GrokAnalyzeTaskEntity {
    GrokAnalyzeTaskEntity {
      post_id,
      created_at_secs,
    }
  }
}

impl TSerializable for GrokAnalyzeTaskEntity {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<GrokAnalyzeTaskEntity> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
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
    verify_required_field_exists("GrokAnalyzeTaskEntity.post_id", &f_1)?;
    verify_required_field_exists("GrokAnalyzeTaskEntity.created_at_secs", &f_2)?;
    let ret = GrokAnalyzeTaskEntity {
      post_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      created_at_secs: f_2.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("GrokAnalyzeTaskEntity");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("post_id", TType::I64, 1))?;
    o_prot.write_i64(self.post_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("createdAtSecs", TType::I64, 2))?;
    o_prot.write_i64(self.created_at_secs)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticCoreData {
  pub event: i64,
}

impl SemanticCoreData {
  pub fn new(event: i64) -> SemanticCoreData {
    SemanticCoreData {
      event,
    }
  }
}

impl TSerializable for SemanticCoreData {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SemanticCoreData> {
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
    verify_required_field_exists("SemanticCoreData.event", &f_1)?;
    let ret = SemanticCoreData {
      event: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SemanticCoreData");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("event", TType::I64, 1))?;
    o_prot.write_i64(self.event)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TrendFlattened {
  pub id: i64,
  pub title: String,
  pub post_ids: Vec<i64>,
  pub cluster_embedding: Vec<OrderedFloat<f64>>,
  pub summary: String,
  pub engagement_score: Option<OrderedFloat<f64>>,
  pub is_new_trend: Option<bool>,
  pub semantic_core_data: Option<SemanticCoreData>,
  pub experiment_id: Option<Experiment>,
  pub hook: Option<String>,
  pub created_at_ms: Option<i64>,
}

impl TrendFlattened {
  pub fn new<F6, F7, F12, F13, F14, F15>(id: i64, title: String, post_ids: Vec<i64>, cluster_embedding: Vec<OrderedFloat<f64>>, summary: String, engagement_score: F6, is_new_trend: F7, semantic_core_data: F12, experiment_id: F13, hook: F14, created_at_ms: F15) -> TrendFlattened where F6: Into<Option<OrderedFloat<f64>>>, F7: Into<Option<bool>>, F12: Into<Option<SemanticCoreData>>, F13: Into<Option<Experiment>>, F14: Into<Option<String>>, F15: Into<Option<i64>> {
    TrendFlattened {
      id,
      title,
      post_ids,
      cluster_embedding,
      summary,
      engagement_score: engagement_score.into(),
      is_new_trend: is_new_trend.into(),
      semantic_core_data: semantic_core_data.into(),
      experiment_id: experiment_id.into(),
      hook: hook.into(),
      created_at_ms: created_at_ms.into(),
    }
  }
}

impl TSerializable for TrendFlattened {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TrendFlattened> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<String> = None;
    let mut f_3: Option<Vec<i64>> = None;
    let mut f_4: Option<Vec<OrderedFloat<f64>>> = None;
    let mut f_5: Option<String> = None;
    let mut f_6: Option<OrderedFloat<f64>> = None;
    let mut f_7: Option<bool> = None;
    let mut f_12: Option<SemanticCoreData> = None;
    let mut f_13: Option<Experiment> = None;
    let mut f_14: Option<String> = None;
    let mut f_15: Option<i64> = None;
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
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_0 = i_prot.read_i64()?;
            val.push(list_elem_0);
          }
          i_prot.read_list_end()?;
          f_3 = Some(val);
        },
        4 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<OrderedFloat<f64>> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_1 = OrderedFloat::from(i_prot.read_double()?);
            val.push(list_elem_1);
          }
          i_prot.read_list_end()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_string()?;
          f_5 = Some(val);
        },
        6 => {
          let val = OrderedFloat::from(i_prot.read_double()?);
          f_6 = Some(val);
        },
        7 => {
          let val = i_prot.read_bool()?;
          f_7 = Some(val);
        },
        12 => {
          let val = SemanticCoreData::read_from_in_protocol(i_prot)?;
          f_12 = Some(val);
        },
        13 => {
          let val = Experiment::read_from_in_protocol(i_prot)?;
          f_13 = Some(val);
        },
        14 => {
          let val = i_prot.read_string()?;
          f_14 = Some(val);
        },
        15 => {
          let val = i_prot.read_i64()?;
          f_15 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("TrendFlattened.id", &f_1)?;
    verify_required_field_exists("TrendFlattened.title", &f_2)?;
    verify_required_field_exists("TrendFlattened.post_ids", &f_3)?;
    verify_required_field_exists("TrendFlattened.cluster_embedding", &f_4)?;
    verify_required_field_exists("TrendFlattened.summary", &f_5)?;
    let ret = TrendFlattened {
      id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      title: f_2.expect("auto-generated code should have checked for presence of required fields"),
      post_ids: f_3.expect("auto-generated code should have checked for presence of required fields"),
      cluster_embedding: f_4.expect("auto-generated code should have checked for presence of required fields"),
      summary: f_5.expect("auto-generated code should have checked for presence of required fields"),
      engagement_score: f_6,
      is_new_trend: f_7,
      semantic_core_data: f_12,
      experiment_id: f_13,
      hook: f_14,
      created_at_ms: f_15,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TrendFlattened");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
    o_prot.write_i64(self.id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("title", TType::String, 2))?;
    o_prot.write_string(&self.title)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("postIds", TType::List, 3))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::I64, self.post_ids.len() as i32))?;
    for e in &self.post_ids {
      o_prot.write_i64(*e)?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("clusterEmbedding", TType::List, 4))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::Double, self.cluster_embedding.len() as i32))?;
    for e in &self.cluster_embedding {
      o_prot.write_double((*e).into())?;
    }
    o_prot.write_list_end()?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("summary", TType::String, 5))?;
    o_prot.write_string(&self.summary)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.engagement_score {
      o_prot.write_field_begin(&TFieldIdentifier::new("engagementScore", TType::Double, 6))?;
      o_prot.write_double(fld_var.into())?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_new_trend {
      o_prot.write_field_begin(&TFieldIdentifier::new("isNewTrend", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.semantic_core_data {
      o_prot.write_field_begin(&TFieldIdentifier::new("semanticCoreData", TType::Struct, 12))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.experiment_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("experimentId", TType::I32, 13))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.hook {
      o_prot.write_field_begin(&TFieldIdentifier::new("hook", TType::String, 14))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("createdAtMs", TType::I64, 15))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StoryFlattened {
  pub cluster_id: i64,
  pub product_id: i64,
  pub title: String,
  pub engagement: i32,
  pub story_type: String,
  pub is_noteworthy: Option<String>,
  pub verified: Option<String>,
  pub topics: Option<String>,
  pub categories: Option<String>,
  pub created_at_ms: Option<i64>,
  pub last_updated_at_ms: Option<i64>,
  pub experiment_id: Option<Experiment>,
}

impl StoryFlattened {
  pub fn new<F6, F7, F8, F9, F10, F11, F13>(cluster_id: i64, product_id: i64, title: String, engagement: i32, story_type: String, is_noteworthy: F6, verified: F7, topics: F8, categories: F9, created_at_ms: F10, last_updated_at_ms: F11, experiment_id: F13) -> StoryFlattened where F6: Into<Option<String>>, F7: Into<Option<String>>, F8: Into<Option<String>>, F9: Into<Option<String>>, F10: Into<Option<i64>>, F11: Into<Option<i64>>, F13: Into<Option<Experiment>> {
    StoryFlattened {
      cluster_id,
      product_id,
      title,
      engagement,
      story_type,
      is_noteworthy: is_noteworthy.into(),
      verified: verified.into(),
      topics: topics.into(),
      categories: categories.into(),
      created_at_ms: created_at_ms.into(),
      last_updated_at_ms: last_updated_at_ms.into(),
      experiment_id: experiment_id.into(),
    }
  }
}

impl TSerializable for StoryFlattened {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<StoryFlattened> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<String> = None;
    let mut f_4: Option<i32> = None;
    let mut f_5: Option<String> = None;
    let mut f_6: Option<String> = None;
    let mut f_7: Option<String> = None;
    let mut f_8: Option<String> = None;
    let mut f_9: Option<String> = None;
    let mut f_10: Option<i64> = None;
    let mut f_11: Option<i64> = None;
    let mut f_13: Option<Experiment> = None;
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
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_i32()?;
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
          let val = i_prot.read_string()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_string()?;
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
        13 => {
          let val = Experiment::read_from_in_protocol(i_prot)?;
          f_13 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("StoryFlattened.cluster_id", &f_1)?;
    verify_required_field_exists("StoryFlattened.product_id", &f_2)?;
    verify_required_field_exists("StoryFlattened.title", &f_3)?;
    verify_required_field_exists("StoryFlattened.engagement", &f_4)?;
    verify_required_field_exists("StoryFlattened.story_type", &f_5)?;
    let ret = StoryFlattened {
      cluster_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      product_id: f_2.expect("auto-generated code should have checked for presence of required fields"),
      title: f_3.expect("auto-generated code should have checked for presence of required fields"),
      engagement: f_4.expect("auto-generated code should have checked for presence of required fields"),
      story_type: f_5.expect("auto-generated code should have checked for presence of required fields"),
      is_noteworthy: f_6,
      verified: f_7,
      topics: f_8,
      categories: f_9,
      created_at_ms: f_10,
      last_updated_at_ms: f_11,
      experiment_id: f_13,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("StoryFlattened");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("cluster_id", TType::I64, 1))?;
    o_prot.write_i64(self.cluster_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("product_id", TType::I64, 2))?;
    o_prot.write_i64(self.product_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("title", TType::String, 3))?;
    o_prot.write_string(&self.title)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("engagement", TType::I32, 4))?;
    o_prot.write_i32(self.engagement)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("storyType", TType::String, 5))?;
    o_prot.write_string(&self.story_type)?;
    o_prot.write_field_end()?;
    if let Some(ref fld_var) = self.is_noteworthy {
      o_prot.write_field_begin(&TFieldIdentifier::new("isNoteworthy", TType::String, 6))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.verified {
      o_prot.write_field_begin(&TFieldIdentifier::new("verified", TType::String, 7))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.topics {
      o_prot.write_field_begin(&TFieldIdentifier::new("topics", TType::String, 8))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.categories {
      o_prot.write_field_begin(&TFieldIdentifier::new("categories", TType::String, 9))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("createdAtMs", TType::I64, 10))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.last_updated_at_ms {
      o_prot.write_field_begin(&TFieldIdentifier::new("lastUpdatedAtMs", TType::I64, 11))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.experiment_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("experimentId", TType::I32, 13))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArticleCrawlerEventEntity {
  pub post_id: i64,
  pub created_at_secs: i64,
}

impl ArticleCrawlerEventEntity {
  pub fn new(post_id: i64, created_at_secs: i64) -> ArticleCrawlerEventEntity {
    ArticleCrawlerEventEntity {
      post_id,
      created_at_secs,
    }
  }
}

impl TSerializable for ArticleCrawlerEventEntity {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ArticleCrawlerEventEntity> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
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
    verify_required_field_exists("ArticleCrawlerEventEntity.post_id", &f_1)?;
    verify_required_field_exists("ArticleCrawlerEventEntity.created_at_secs", &f_2)?;
    let ret = ArticleCrawlerEventEntity {
      post_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      created_at_secs: f_2.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ArticleCrawlerEventEntity");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("post_id", TType::I64, 1))?;
    o_prot.write_i64(self.post_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("createdAtSecs", TType::I64, 2))?;
    o_prot.write_i64(self.created_at_secs)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TweetEmbedding {
  pub tweet_id: Option<i64>,
  pub embedding1: Option<Vec<OrderedFloat<f64>>>,
  pub user_id: Option<i64>,
  pub user_credibility: Option<i16>,
  pub followers: Option<i64>,
  pub text: Option<String>,
  pub url_card_title: Option<String>,
  pub user_name: Option<String>,
  pub user_screen_name: Option<String>,
  pub qt_text: Option<String>,
  pub reply_to_text: Option<String>,
  pub qt_followers: Option<i64>,
  pub reply_to_followers: Option<i64>,
  pub qt_text_raw: Option<String>,
  pub reply_to_text_raw: Option<String>,
  pub media_caption: Option<String>,
}

impl TweetEmbedding {
  pub fn new<F1, F2, F3, F5, F7, F102, F104, F900, F901, F902, F903, F906, F907, F908, F909, F910>(tweet_id: F1, embedding1: F2, user_id: F3, user_credibility: F5, followers: F7, text: F102, url_card_title: F104, user_name: F900, user_screen_name: F901, qt_text: F902, reply_to_text: F903, qt_followers: F906, reply_to_followers: F907, qt_text_raw: F908, reply_to_text_raw: F909, media_caption: F910) -> TweetEmbedding where F1: Into<Option<i64>>, F2: Into<Option<Vec<OrderedFloat<f64>>>>, F3: Into<Option<i64>>, F5: Into<Option<i16>>, F7: Into<Option<i64>>, F102: Into<Option<String>>, F104: Into<Option<String>>, F900: Into<Option<String>>, F901: Into<Option<String>>, F902: Into<Option<String>>, F903: Into<Option<String>>, F906: Into<Option<i64>>, F907: Into<Option<i64>>, F908: Into<Option<String>>, F909: Into<Option<String>>, F910: Into<Option<String>> {
    TweetEmbedding {
      tweet_id: tweet_id.into(),
      embedding1: embedding1.into(),
      user_id: user_id.into(),
      user_credibility: user_credibility.into(),
      followers: followers.into(),
      text: text.into(),
      url_card_title: url_card_title.into(),
      user_name: user_name.into(),
      user_screen_name: user_screen_name.into(),
      qt_text: qt_text.into(),
      reply_to_text: reply_to_text.into(),
      qt_followers: qt_followers.into(),
      reply_to_followers: reply_to_followers.into(),
      qt_text_raw: qt_text_raw.into(),
      reply_to_text_raw: reply_to_text_raw.into(),
      media_caption: media_caption.into(),
    }
  }
}

impl TSerializable for TweetEmbedding {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TweetEmbedding> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<Vec<OrderedFloat<f64>>> = None;
    let mut f_3: Option<i64> = None;
    let mut f_5: Option<i16> = None;
    let mut f_7: Option<i64> = None;
    let mut f_102: Option<String> = None;
    let mut f_104: Option<String> = None;
    let mut f_900: Option<String> = None;
    let mut f_901: Option<String> = None;
    let mut f_902: Option<String> = None;
    let mut f_903: Option<String> = None;
    let mut f_906: Option<i64> = None;
    let mut f_907: Option<i64> = None;
    let mut f_908: Option<String> = None;
    let mut f_909: Option<String> = None;
    let mut f_910: Option<String> = None;
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
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<OrderedFloat<f64>> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_2 = OrderedFloat::from(i_prot.read_double()?);
            val.push(list_elem_2);
          }
          i_prot.read_list_end()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i64()?;
          f_3 = Some(val);
        },
        5 => {
          let val = i_prot.read_i16()?;
          f_5 = Some(val);
        },
        7 => {
          let val = i_prot.read_i64()?;
          f_7 = Some(val);
        },
        102 => {
          let val = i_prot.read_string()?;
          f_102 = Some(val);
        },
        104 => {
          let val = i_prot.read_string()?;
          f_104 = Some(val);
        },
        900 => {
          let val = i_prot.read_string()?;
          f_900 = Some(val);
        },
        901 => {
          let val = i_prot.read_string()?;
          f_901 = Some(val);
        },
        902 => {
          let val = i_prot.read_string()?;
          f_902 = Some(val);
        },
        903 => {
          let val = i_prot.read_string()?;
          f_903 = Some(val);
        },
        906 => {
          let val = i_prot.read_i64()?;
          f_906 = Some(val);
        },
        907 => {
          let val = i_prot.read_i64()?;
          f_907 = Some(val);
        },
        908 => {
          let val = i_prot.read_string()?;
          f_908 = Some(val);
        },
        909 => {
          let val = i_prot.read_string()?;
          f_909 = Some(val);
        },
        910 => {
          let val = i_prot.read_string()?;
          f_910 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = TweetEmbedding {
      tweet_id: f_1,
      embedding1: f_2,
      user_id: f_3,
      user_credibility: f_5,
      followers: f_7,
      text: f_102,
      url_card_title: f_104,
      user_name: f_900,
      user_screen_name: f_901,
      qt_text: f_902,
      reply_to_text: f_903,
      qt_followers: f_906,
      reply_to_followers: f_907,
      qt_text_raw: f_908,
      reply_to_text_raw: f_909,
      media_caption: f_910,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TweetEmbedding");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.tweet_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweetId", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.embedding1 {
      o_prot.write_field_begin(&TFieldIdentifier::new("embedding1", TType::List, 2))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Double, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_double((*e).into())?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("userId", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.user_credibility {
      o_prot.write_field_begin(&TFieldIdentifier::new("userCredibility", TType::I16, 5))?;
      o_prot.write_i16(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.followers {
      o_prot.write_field_begin(&TFieldIdentifier::new("followers", TType::I64, 7))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.text {
      o_prot.write_field_begin(&TFieldIdentifier::new("text", TType::String, 102))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.url_card_title {
      o_prot.write_field_begin(&TFieldIdentifier::new("urlCardTitle", TType::String, 104))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("userName", TType::String, 900))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user_screen_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("userScreenName", TType::String, 901))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.qt_text {
      o_prot.write_field_begin(&TFieldIdentifier::new("qtText", TType::String, 902))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.reply_to_text {
      o_prot.write_field_begin(&TFieldIdentifier::new("replyToText", TType::String, 903))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.qt_followers {
      o_prot.write_field_begin(&TFieldIdentifier::new("qtFollowers", TType::I64, 906))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.reply_to_followers {
      o_prot.write_field_begin(&TFieldIdentifier::new("replyToFollowers", TType::I64, 907))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.qt_text_raw {
      o_prot.write_field_begin(&TFieldIdentifier::new("qtTextRaw", TType::String, 908))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.reply_to_text_raw {
      o_prot.write_field_begin(&TFieldIdentifier::new("replyToTextRaw", TType::String, 909))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media_caption {
      o_prot.write_field_begin(&TFieldIdentifier::new("mediaCaption", TType::String, 910))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

