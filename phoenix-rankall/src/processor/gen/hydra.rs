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

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PhoenixRankAllObject {
    pub post_id: Option<i64>,
    pub author_id: Option<i64>,
    pub index_name: Option<String>,
    pub topic_entity_ids: Option<Vec<i64>>,
}

impl PhoenixRankAllObject {
    pub fn new<F1, F2, F3, F100>(
        post_id: F1,
        author_id: F2,
        index_name: F3,
        topic_entity_ids: F100,
    ) -> PhoenixRankAllObject
    where
        F1: Into<Option<i64>>,
        F2: Into<Option<i64>>,
        F3: Into<Option<String>>,
        F100: Into<Option<Vec<i64>>>,
    {
        PhoenixRankAllObject {
            post_id: post_id.into(),
            author_id: author_id.into(),
            index_name: index_name.into(),
            topic_entity_ids: topic_entity_ids.into(),
        }
    }
}

impl TSerializable for PhoenixRankAllObject {
    fn read_from_in_protocol(
        i_prot: &mut dyn TInputProtocol,
    ) -> thrift::Result<PhoenixRankAllObject> {
        i_prot.read_struct_begin()?;
        let mut f_1: Option<i64> = Some(0);
        let mut f_2: Option<i64> = Some(0);
        let mut f_3: Option<String> = Some("".to_owned());
        let mut f_100: Option<Vec<i64>> = None;
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
                    let val = i_prot.read_string()?;
                    f_3 = Some(val);
                }
                100 => {
                    let list_ident = i_prot.read_list_begin()?;
                    let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
                    for _ in 0..list_ident.size {
                        let list_elem_0 = i_prot.read_i64()?;
                        val.push(list_elem_0);
                    }
                    i_prot.read_list_end()?;
                    f_100 = Some(val);
                }
                _ => {
                    i_prot.skip(field_ident.field_type)?;
                }
            };
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        let ret = PhoenixRankAllObject {
            post_id: f_1,
            author_id: f_2,
            index_name: f_3,
            topic_entity_ids: f_100,
        };
        Ok(ret)
    }
    fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        let struct_ident = TStructIdentifier::new("PhoenixRankAllObject");
        o_prot.write_struct_begin(&struct_ident)?;
        if let Some(fld_var) = self.post_id {
            o_prot.write_field_begin(&TFieldIdentifier::new("postId", TType::I64, 1))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.author_id {
            o_prot.write_field_begin(&TFieldIdentifier::new("authorId", TType::I64, 2))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(ref fld_var) = self.index_name {
            o_prot.write_field_begin(&TFieldIdentifier::new("indexName", TType::String, 3))?;
            o_prot.write_string(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(ref fld_var) = self.topic_entity_ids {
            o_prot.write_field_begin(&TFieldIdentifier::new("topicEntityIds", TType::List, 100))?;
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
pub struct EngagementCount {
    pub retweet_count: Option<i64>,
    pub reply_count: Option<i64>,
    pub favorite_count: Option<i64>,
    pub quote_count: Option<i64>,
    pub bookmark_count: Option<i64>,
    pub view_count: Option<i64>,
    pub not_interested_in_count: Option<i64>,
    pub report_count: Option<i64>,
    pub block_count: Option<i64>,
}

impl EngagementCount {
    pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9>(
        retweet_count: F1,
        reply_count: F2,
        favorite_count: F3,
        quote_count: F4,
        bookmark_count: F5,
        view_count: F6,
        not_interested_in_count: F7,
        report_count: F8,
        block_count: F9,
    ) -> EngagementCount
    where
        F1: Into<Option<i64>>,
        F2: Into<Option<i64>>,
        F3: Into<Option<i64>>,
        F4: Into<Option<i64>>,
        F5: Into<Option<i64>>,
        F6: Into<Option<i64>>,
        F7: Into<Option<i64>>,
        F8: Into<Option<i64>>,
        F9: Into<Option<i64>>,
    {
        EngagementCount {
            retweet_count: retweet_count.into(),
            reply_count: reply_count.into(),
            favorite_count: favorite_count.into(),
            quote_count: quote_count.into(),
            bookmark_count: bookmark_count.into(),
            view_count: view_count.into(),
            not_interested_in_count: not_interested_in_count.into(),
            report_count: report_count.into(),
            block_count: block_count.into(),
        }
    }
}

impl TSerializable for EngagementCount {
    fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<EngagementCount> {
        i_prot.read_struct_begin()?;
        let mut f_1: Option<i64> = None;
        let mut f_2: Option<i64> = None;
        let mut f_3: Option<i64> = None;
        let mut f_4: Option<i64> = None;
        let mut f_5: Option<i64> = None;
        let mut f_6: Option<i64> = None;
        let mut f_7: Option<i64> = None;
        let mut f_8: Option<i64> = None;
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
                    let val = i_prot.read_i64()?;
                    f_5 = Some(val);
                }
                6 => {
                    let val = i_prot.read_i64()?;
                    f_6 = Some(val);
                }
                7 => {
                    let val = i_prot.read_i64()?;
                    f_7 = Some(val);
                }
                8 => {
                    let val = i_prot.read_i64()?;
                    f_8 = Some(val);
                }
                9 => {
                    let val = i_prot.read_i64()?;
                    f_9 = Some(val);
                }
                _ => {
                    i_prot.skip(field_ident.field_type)?;
                }
            };
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        let ret = EngagementCount {
            retweet_count: f_1,
            reply_count: f_2,
            favorite_count: f_3,
            quote_count: f_4,
            bookmark_count: f_5,
            view_count: f_6,
            not_interested_in_count: f_7,
            report_count: f_8,
            block_count: f_9,
        };
        Ok(ret)
    }
    fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        let struct_ident = TStructIdentifier::new("EngagementCount");
        o_prot.write_struct_begin(&struct_ident)?;
        if let Some(fld_var) = self.retweet_count {
            o_prot.write_field_begin(&TFieldIdentifier::new("retweetCount", TType::I64, 1))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.reply_count {
            o_prot.write_field_begin(&TFieldIdentifier::new("replyCount", TType::I64, 2))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.favorite_count {
            o_prot.write_field_begin(&TFieldIdentifier::new("favoriteCount", TType::I64, 3))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.quote_count {
            o_prot.write_field_begin(&TFieldIdentifier::new("quoteCount", TType::I64, 4))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.bookmark_count {
            o_prot.write_field_begin(&TFieldIdentifier::new("bookmarkCount", TType::I64, 5))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.view_count {
            o_prot.write_field_begin(&TFieldIdentifier::new("viewCount", TType::I64, 6))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.not_interested_in_count {
            o_prot.write_field_begin(&TFieldIdentifier::new(
                "notInterestedInCount",
                TType::I64,
                7,
            ))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.report_count {
            o_prot.write_field_begin(&TFieldIdentifier::new("reportCount", TType::I64, 8))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.block_count {
            o_prot.write_field_begin(&TFieldIdentifier::new("blockCount", TType::I64, 9))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        o_prot.write_field_stop()?;
        o_prot.write_struct_end()
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MultiModalEmbeddings {
    pub mm_emb_v1: Option<Vec<OrderedFloat<f64>>>,
    pub mm_emb_v3: Option<Vec<OrderedFloat<f64>>>,
}

impl MultiModalEmbeddings {
    pub fn new<F1, F2>(mm_emb_v1: F1, mm_emb_v3: F2) -> MultiModalEmbeddings
    where
        F1: Into<Option<Vec<OrderedFloat<f64>>>>,
        F2: Into<Option<Vec<OrderedFloat<f64>>>>,
    {
        MultiModalEmbeddings {
            mm_emb_v1: mm_emb_v1.into(),
            mm_emb_v3: mm_emb_v3.into(),
        }
    }
}

impl TSerializable for MultiModalEmbeddings {
    fn read_from_in_protocol(
        i_prot: &mut dyn TInputProtocol,
    ) -> thrift::Result<MultiModalEmbeddings> {
        i_prot.read_struct_begin()?;
        let mut f_1: Option<Vec<OrderedFloat<f64>>> = None;
        let mut f_2: Option<Vec<OrderedFloat<f64>>> = None;
        loop {
            let field_ident = i_prot.read_field_begin()?;
            if field_ident.field_type == TType::Stop {
                break;
            }
            let field_id = field_id(&field_ident)?;
            match field_id {
                1 => {
                    let list_ident = i_prot.read_list_begin()?;
                    let mut val: Vec<OrderedFloat<f64>> =
                        Vec::with_capacity(list_ident.size as usize);
                    for _ in 0..list_ident.size {
                        let list_elem_1 = OrderedFloat::from(i_prot.read_double()?);
                        val.push(list_elem_1);
                    }
                    i_prot.read_list_end()?;
                    f_1 = Some(val);
                }
                2 => {
                    let list_ident = i_prot.read_list_begin()?;
                    let mut val: Vec<OrderedFloat<f64>> =
                        Vec::with_capacity(list_ident.size as usize);
                    for _ in 0..list_ident.size {
                        let list_elem_2 = OrderedFloat::from(i_prot.read_double()?);
                        val.push(list_elem_2);
                    }
                    i_prot.read_list_end()?;
                    f_2 = Some(val);
                }
                _ => {
                    i_prot.skip(field_ident.field_type)?;
                }
            };
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        let ret = MultiModalEmbeddings {
            mm_emb_v1: f_1,
            mm_emb_v3: f_2,
        };
        Ok(ret)
    }
    fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        let struct_ident = TStructIdentifier::new("MultiModalEmbeddings");
        o_prot.write_struct_begin(&struct_ident)?;
        if let Some(ref fld_var) = self.mm_emb_v1 {
            o_prot.write_field_begin(&TFieldIdentifier::new("mmEmbV1", TType::List, 1))?;
            o_prot.write_list_begin(&TListIdentifier::new(TType::Double, fld_var.len() as i32))?;
            for e in fld_var {
                o_prot.write_double((*e).into())?;
            }
            o_prot.write_list_end()?;
            o_prot.write_field_end()?
        }
        if let Some(ref fld_var) = self.mm_emb_v3 {
            o_prot.write_field_begin(&TFieldIdentifier::new("mmEmbV3", TType::List, 2))?;
            o_prot.write_list_begin(&TListIdentifier::new(TType::Double, fld_var.len() as i32))?;
            for e in fld_var {
                o_prot.write_double((*e).into())?;
            }
            o_prot.write_list_end()?;
            o_prot.write_field_end()?
        }
        o_prot.write_field_stop()?;
        o_prot.write_struct_end()
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PhoenixRankAllCandidateFlat {
    pub post_id: Option<i64>,
    pub author_id: Option<i64>,
    pub has_video: Option<bool>,
    pub has_image: Option<bool>,
    pub author_followers_count: Option<i64>,
    pub engagement_count: Option<EngagementCount>,
    pub video_duration_ms: Option<i64>,
    pub mm_embs: Option<MultiModalEmbeddings>,
    pub index_name: Option<String>,
}

impl PhoenixRankAllCandidateFlat {
    pub fn new<F1, F2, F3, F4, F5, F6, F8, F9, F100>(
        post_id: F1,
        author_id: F2,
        has_video: F3,
        has_image: F4,
        author_followers_count: F5,
        engagement_count: F6,
        video_duration_ms: F8,
        mm_embs: F9,
        index_name: F100,
    ) -> PhoenixRankAllCandidateFlat
    where
        F1: Into<Option<i64>>,
        F2: Into<Option<i64>>,
        F3: Into<Option<bool>>,
        F4: Into<Option<bool>>,
        F5: Into<Option<i64>>,
        F6: Into<Option<EngagementCount>>,
        F8: Into<Option<i64>>,
        F9: Into<Option<MultiModalEmbeddings>>,
        F100: Into<Option<String>>,
    {
        PhoenixRankAllCandidateFlat {
            post_id: post_id.into(),
            author_id: author_id.into(),
            has_video: has_video.into(),
            has_image: has_image.into(),
            author_followers_count: author_followers_count.into(),
            engagement_count: engagement_count.into(),
            video_duration_ms: video_duration_ms.into(),
            mm_embs: mm_embs.into(),
            index_name: index_name.into(),
        }
    }
}

impl TSerializable for PhoenixRankAllCandidateFlat {
    fn read_from_in_protocol(
        i_prot: &mut dyn TInputProtocol,
    ) -> thrift::Result<PhoenixRankAllCandidateFlat> {
        i_prot.read_struct_begin()?;
        let mut f_1: Option<i64> = Some(0);
        let mut f_2: Option<i64> = Some(0);
        let mut f_3: Option<bool> = Some(false);
        let mut f_4: Option<bool> = Some(false);
        let mut f_5: Option<i64> = Some(0);
        let mut f_6: Option<EngagementCount> = None;
        let mut f_8: Option<i64> = None;
        let mut f_9: Option<MultiModalEmbeddings> = None;
        let mut f_100: Option<String> = None;
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
                    let val = i_prot.read_bool()?;
                    f_3 = Some(val);
                }
                4 => {
                    let val = i_prot.read_bool()?;
                    f_4 = Some(val);
                }
                5 => {
                    let val = i_prot.read_i64()?;
                    f_5 = Some(val);
                }
                6 => {
                    let val = EngagementCount::read_from_in_protocol(i_prot)?;
                    f_6 = Some(val);
                }
                8 => {
                    let val = i_prot.read_i64()?;
                    f_8 = Some(val);
                }
                9 => {
                    let val = MultiModalEmbeddings::read_from_in_protocol(i_prot)?;
                    f_9 = Some(val);
                }
                100 => {
                    let val = i_prot.read_string()?;
                    f_100 = Some(val);
                }
                _ => {
                    i_prot.skip(field_ident.field_type)?;
                }
            };
            i_prot.read_field_end()?;
        }
        i_prot.read_struct_end()?;
        let ret = PhoenixRankAllCandidateFlat {
            post_id: f_1,
            author_id: f_2,
            has_video: f_3,
            has_image: f_4,
            author_followers_count: f_5,
            engagement_count: f_6,
            video_duration_ms: f_8,
            mm_embs: f_9,
            index_name: f_100,
        };
        Ok(ret)
    }
    fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        let struct_ident = TStructIdentifier::new("PhoenixRankAllCandidateFlat");
        o_prot.write_struct_begin(&struct_ident)?;
        if let Some(fld_var) = self.post_id {
            o_prot.write_field_begin(&TFieldIdentifier::new("postId", TType::I64, 1))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.author_id {
            o_prot.write_field_begin(&TFieldIdentifier::new("authorId", TType::I64, 2))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.has_video {
            o_prot.write_field_begin(&TFieldIdentifier::new("hasVideo", TType::Bool, 3))?;
            o_prot.write_bool(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.has_image {
            o_prot.write_field_begin(&TFieldIdentifier::new("hasImage", TType::Bool, 4))?;
            o_prot.write_bool(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.author_followers_count {
            o_prot.write_field_begin(&TFieldIdentifier::new(
                "authorFollowersCount",
                TType::I64,
                5,
            ))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(ref fld_var) = self.engagement_count {
            o_prot.write_field_begin(&TFieldIdentifier::new(
                "engagementCount",
                TType::Struct,
                6,
            ))?;
            fld_var.write_to_out_protocol(o_prot)?;
            o_prot.write_field_end()?
        }
        if let Some(fld_var) = self.video_duration_ms {
            o_prot.write_field_begin(&TFieldIdentifier::new("videoDurationMs", TType::I64, 8))?;
            o_prot.write_i64(fld_var)?;
            o_prot.write_field_end()?
        }
        if let Some(ref fld_var) = self.mm_embs {
            o_prot.write_field_begin(&TFieldIdentifier::new("mmEmbs", TType::Struct, 9))?;
            fld_var.write_to_out_protocol(o_prot)?;
            o_prot.write_field_end()?
        }
        if let Some(ref fld_var) = self.index_name {
            o_prot.write_field_begin(&TFieldIdentifier::new("indexName", TType::String, 100))?;
            o_prot.write_string(fld_var)?;
            o_prot.write_field_end()?
        }
        o_prot.write_field_stop()?;
        o_prot.write_struct_end()
    }
}
