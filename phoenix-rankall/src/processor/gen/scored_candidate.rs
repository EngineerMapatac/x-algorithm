#[allow(dead_code)]
#[allow(unused_imports)]
#[allow(unused_extern_crates)]
#[allow(clippy::too_many_arguments, clippy::type_complexity, clippy::vec_box, clippy::wrong_self_convention)]
#[cfg_attr(rustfmt, rustfmt_skip)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::{From, TryFrom};
use std::default::Default;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use thrift::protocol::field_id;
use thrift::protocol::verify_expected_message_type;
use thrift::protocol::verify_expected_sequence_number;
use thrift::protocol::verify_expected_service_call;
use thrift::protocol::verify_required_field_exists;
use thrift::protocol::{
    TFieldIdentifier, TInputProtocol, TListIdentifier, TMapIdentifier, TMessageIdentifier,
    TMessageType, TOutputProtocol, TSerializable, TSetIdentifier, TStructIdentifier, TType,
};
use thrift::server::TProcessor;
use thrift::OrderedFloat;
use thrift::{
    ApplicationError, ApplicationErrorKind, ProtocolError, ProtocolErrorKind, TThriftClient,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScoredCandidate {
    pub tweet_id: i64,
    pub viewer_id: Option<i64>,
    pub author_id: Option<i64>,
    pub score: Option<OrderedFloat<f64>>,
    pub suggest_type: Option<String>,
    pub request_time_ms: Option<i64>,
    pub source_tweet_id: Option<i64>,
    pub prediction_scores: Option<BTreeSet<Box<PredictionScore>>>,
}

impl ScoredCandidate {
    pub fn new<F2, F3, F6, F7, F17, F22, F24>(
        tweet_id: i64,
        viewer_id: F2,
        author_id: F3,
        score: F6,
        suggest_type: F7,
        request_time_ms: F17,
        source_tweet_id: F22,
        prediction_scores: F24,
    ) -> ScoredCandidate
    where
        F2: Into<Option<i64>>,
        F3: Into<Option<i64>>,
        F6: Into<Option<OrderedFloat<f64>>>,
        F7: Into<Option<String>>,
        F17: Into<Option<i64>>,
        F22: Into<Option<i64>>,
        F24: Into<Option<BTreeSet<Box<PredictionScore>>>>,
    {
        ScoredCandidate {
            tweet_id,
            viewer_id: viewer_id.into(),
            author_id: author_id.into(),
            score: score.into(),
            suggest_type: suggest_type.into(),
            request_time_ms: request_time_ms.into(),
            source_tweet_id: source_tweet_id.into(),
            prediction_scores: prediction_scores.into(),
        }
    }
}

impl TSerializable for ScoredCandidate {
    fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ScoredCandidate> {
        i_prot.read_struct_begin()?;
        let mut f_1: Option<i64> = None;
        let mut f_2: Option<i64> = None;
        let mut f_3: Option<i64> = None;
        let mut f_6: Option<OrderedFloat<f64>> = None;
        let mut f_7: Option<String> = None;
        let mut f_17: Option<i64> = None;
        let mut f_22: Option<i64> = None;
        let mut f_24: Option<BTreeSet<Box<PredictionScore>>> = None;
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
                }
                2 => {
                    let val = i_prot.read_i64()?;
                    f_2 = Some(val);
                }
                3 => {
                    let val = i_prot.read_i64()?;
                    f_3 = Some(val);
                }
                6 => {
                    let val = OrderedFloat::from(i_prot.read_double()?);
                    f_6 = Some(val);
                }
                7 => {
                    let val = i_prot.read_string()?;
                    f_7 = Some(val);
                }
                17 => {
                    let val = i_prot.read_i64()?;
                    f_17 = Some(val);
                }
                22 => {
                    let val = i_prot.read_i64()?;
                    f_22 = Some(val);
                }
                24 => {
                    let set_ident = i_prot.read_set_begin()?;
                    let mut val: BTreeSet<Box<PredictionScore>> = BTreeSet::new();
                    for _ in 0..set_ident.size {
                        let set_elem_0 = Box::new(PredictionScore::read_from_in_protocol(i_prot)?);
                        val.insert(set_elem_0);
                    }
                    i_prot.read_set_end()?;
                    f_24 = Some(val);
                }
                _ => {
                    i_prot.skip(field_ident.field_type)?;
                }
            };
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        verify_required_field_exists("ScoredCandidate.tweet_id", &f_1)?;
        let ret = ScoredCandidate {
            tweet_id: f_1
                .expect("auto-generated code should have checked for presence of required fields"),
            viewer_id: f_2,
            author_id: f_3,
            score: f_6,
            suggest_type: f_7,
            request_time_ms: f_17,
            source_tweet_id: f_22,
            prediction_scores: f_24,
        };
        Ok(ret)
    }
    fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        let struct_ident = TStructIdentifier::new("ScoredCandidate");
        o_prot.write_struct_begin(&struct_ident)?;
        o_prot.write_field_begin(&TFieldIdentifier::new("tweetId", TType::I64, 1))?;
        o_prot.write_i64(self.tweet_id)?;
        o_prot.write_field_end()?;
        if let Some(fld_var) = self.viewer_id {
            o_prot.write_field_begin(&TFieldIdentifier::new("viewerId", TType::I64, 2))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.author_id {
            o_prot.write_field_begin(&TFieldIdentifier::new("authorId", TType::I64, 3))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.score {
            o_prot.write_field_begin(&TFieldIdentifier::new("score", TType::Double, 6))?;
            o_prot.write_double(fld_var.into())?;
            o_prot.write_field_end()?
        }
        if let Some(ref fld_var) = self.suggest_type {
            o_prot.write_field_begin(&TFieldIdentifier::new("suggestType", TType::String, 7))?;
            o_prot.write_string(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.request_time_ms {
            o_prot.write_field_begin(&TFieldIdentifier::new("requestTimeMs", TType::I64, 17))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.source_tweet_id {
            o_prot.write_field_begin(&TFieldIdentifier::new("sourceTweetId", TType::I64, 22))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(ref fld_var) = self.prediction_scores {
            o_prot.write_field_begin(&TFieldIdentifier::new("predictionScores", TType::Set, 24))?;
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
pub struct PredictionScore {
    pub name: Option<String>,
    pub score: Option<OrderedFloat<f64>>,
}

impl PredictionScore {
    pub fn new<F1, F2>(name: F1, score: F2) -> PredictionScore
    where
        F1: Into<Option<String>>,
        F2: Into<Option<OrderedFloat<f64>>>,
    {
        PredictionScore {
            name: name.into(),
            score: score.into(),
        }
    }
}

impl TSerializable for PredictionScore {
    fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PredictionScore> {
        i_prot.read_struct_begin()?;
        let mut f_1: Option<String> = None;
        let mut f_2: Option<OrderedFloat<f64>> = None;
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
                }
                2 => {
                    let val = OrderedFloat::from(i_prot.read_double()?);
                    f_2 = Some(val);
                }
                _ => {
                    i_prot.skip(field_ident.field_type)?;
                }
            };
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        let ret = PredictionScore {
            name: f_1,
            score: f_2,
        };
        Ok(ret)
    }
    fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        let struct_ident = TStructIdentifier::new("PredictionScore");
        o_prot.write_struct_begin(&struct_ident)?;
        if let Some(ref fld_var) = self.name {
            o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 1))?;
            o_prot.write_string(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.score {
            o_prot.write_field_begin(&TFieldIdentifier::new("score", TType::Double, 2))?;
            o_prot.write_double(fld_var.into())?;
            o_prot.write_field_end()?
        }
        o_prot.write_field_stop()?;
        o_prot.write_struct_end()
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ServedSearchCandidateList {
    pub tweet_ids: Vec<i64>,
    pub viewer_id: Option<i64>,
    pub request_time_ms: Option<i64>,
    pub served_request_id: Option<i64>,
    pub raw_query: Option<String>,
}

impl ServedSearchCandidateList {
    pub fn new<F2, F3, F4, F5>(
        tweet_ids: Vec<i64>,
        viewer_id: F2,
        request_time_ms: F3,
        served_request_id: F4,
        raw_query: F5,
    ) -> ServedSearchCandidateList
    where
        F2: Into<Option<i64>>,
        F3: Into<Option<i64>>,
        F4: Into<Option<i64>>,
        F5: Into<Option<String>>,
    {
        ServedSearchCandidateList {
            tweet_ids,
            viewer_id: viewer_id.into(),
            request_time_ms: request_time_ms.into(),
            served_request_id: served_request_id.into(),
            raw_query: raw_query.into(),
        }
    }
}

impl TSerializable for ServedSearchCandidateList {
    fn read_from_in_protocol(
        i_prot: &mut dyn TInputProtocol,
    ) -> thrift::Result<ServedSearchCandidateList> {
        i_prot.read_struct_begin()?;
        let mut f_1: Option<Vec<i64>> = None;
        let mut f_2: Option<i64> = None;
        let mut f_3: Option<i64> = None;
        let mut f_4: Option<i64> = None;
        let mut f_5: Option<String> = None;
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
                        let list_elem_1 = i_prot.read_i64()?;
                        val.push(list_elem_1);
                    }
                    i_prot.read_list_end()?;
                    f_1 = Some(val);
                }
                2 => {
                    let val = i_prot.read_i64()?;
                    f_2 = Some(val);
                }
                3 => {
                    let val = i_prot.read_i64()?;
                    f_3 = Some(val);
                }
                4 => {
                    let val = i_prot.read_i64()?;
                    f_4 = Some(val);
                }
                5 => {
                    let val = i_prot.read_string()?;
                    f_5 = Some(val);
                }
                _ => {
                    i_prot.skip(field_ident.field_type)?;
                }
            };
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        verify_required_field_exists("ServedSearchCandidateList.tweet_ids", &f_1)?;
        let ret = ServedSearchCandidateList {
            tweet_ids: f_1
                .expect("auto-generated code should have checked for presence of required fields"),
            viewer_id: f_2,
            request_time_ms: f_3,
            served_request_id: f_4,
            raw_query: f_5,
        };
        Ok(ret)
    }
    fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        let struct_ident = TStructIdentifier::new("ServedSearchCandidateList");
        o_prot.write_struct_begin(&struct_ident)?;
        o_prot.write_field_begin(&TFieldIdentifier::new("tweetIds", TType::List, 1))?;
        o_prot.write_list_begin(&TListIdentifier::new(
            TType::I64,
            self.tweet_ids.len() as i32,
        ))?;
        for e in &self.tweet_ids {
            o_prot.write_i64(*e)?;
        }
        o_prot.write_list_end()?;
        o_prot.write_field_end()?;
        if let Some(fld_var) = self.viewer_id {
            o_prot.write_field_begin(&TFieldIdentifier::new("viewerId", TType::I64, 2))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.request_time_ms {
            o_prot.write_field_begin(&TFieldIdentifier::new("requestTimeMs", TType::I64, 3))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.served_request_id {
            o_prot.write_field_begin(&TFieldIdentifier::new("servedRequestId", TType::I64, 4))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(ref fld_var) = self.raw_query {
            o_prot.write_field_begin(&TFieldIdentifier::new("rawQuery", TType::String, 5))?;
            o_prot.write_string(fld_var)?;
            o_prot.write_field_end()?
        }
        o_prot.write_field_stop()?;
        o_prot.write_struct_end()
    }
}
