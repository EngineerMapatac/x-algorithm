
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

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProfileType(pub i32);

impl ProfileType {
  pub const DEFAULT: ProfileType = ProfileType(0);
  pub const THEME: ProfileType = ProfileType(1);
  pub const CUSTOM: ProfileType = ProfileType(2);
  pub const DEFAULT_WITHOUT_BACKGROUND: ProfileType = ProfileType(3);
  pub const CUSTOM_WITHOUT_THEME: ProfileType = ProfileType(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::DEFAULT,
    Self::THEME,
    Self::CUSTOM,
    Self::DEFAULT_WITHOUT_BACKGROUND,
    Self::CUSTOM_WITHOUT_THEME,
  ];
}

impl TSerializable for ProfileType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProfileType> {
    let enum_value = i_prot.read_i32()?;
    Ok(ProfileType::from(enum_value))
  }
}

impl From<i32> for ProfileType {
  fn from(i: i32) -> Self {
    match i {
      0 => ProfileType::DEFAULT,
      1 => ProfileType::THEME,
      2 => ProfileType::CUSTOM,
      3 => ProfileType::DEFAULT_WITHOUT_BACKGROUND,
      4 => ProfileType::CUSTOM_WITHOUT_THEME,
      _ => ProfileType(i)
    }
  }
}

impl From<&i32> for ProfileType {
  fn from(i: &i32) -> Self {
    ProfileType::from(*i)
  }
}

impl From<ProfileType> for i32 {
  fn from(e: ProfileType) -> i32 {
    e.0
  }
}

impl From<&ProfileType> for i32 {
  fn from(e: &ProfileType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BackgroundPosition(pub i32);

impl BackgroundPosition {
  pub const LEFT: BackgroundPosition = BackgroundPosition(0);
  pub const CENTER: BackgroundPosition = BackgroundPosition(1);
  pub const RIGHT: BackgroundPosition = BackgroundPosition(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::LEFT,
    Self::CENTER,
    Self::RIGHT,
  ];
}

impl TSerializable for BackgroundPosition {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<BackgroundPosition> {
    let enum_value = i_prot.read_i32()?;
    Ok(BackgroundPosition::from(enum_value))
  }
}

impl From<i32> for BackgroundPosition {
  fn from(i: i32) -> Self {
    match i {
      0 => BackgroundPosition::LEFT,
      1 => BackgroundPosition::CENTER,
      2 => BackgroundPosition::RIGHT,
      _ => BackgroundPosition(i)
    }
  }
}

impl From<&i32> for BackgroundPosition {
  fn from(i: &i32) -> Self {
    BackgroundPosition::from(*i)
  }
}

impl From<BackgroundPosition> for i32 {
  fn from(e: BackgroundPosition) -> i32 {
    e.0
  }
}

impl From<&BackgroundPosition> for i32 {
  fn from(e: &BackgroundPosition) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BusinessProfileState(pub i32);

impl BusinessProfileState {
  pub const NONE: BusinessProfileState = BusinessProfileState(0);
  pub const ENABLED: BusinessProfileState = BusinessProfileState(1);
  pub const DISABLED: BusinessProfileState = BusinessProfileState(2);
  pub const RESERVED_1: BusinessProfileState = BusinessProfileState(3);
  pub const RESERVED_2: BusinessProfileState = BusinessProfileState(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NONE,
    Self::ENABLED,
    Self::DISABLED,
    Self::RESERVED_1,
    Self::RESERVED_2,
  ];
}

impl TSerializable for BusinessProfileState {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<BusinessProfileState> {
    let enum_value = i_prot.read_i32()?;
    Ok(BusinessProfileState::from(enum_value))
  }
}

impl From<i32> for BusinessProfileState {
  fn from(i: i32) -> Self {
    match i {
      0 => BusinessProfileState::NONE,
      1 => BusinessProfileState::ENABLED,
      2 => BusinessProfileState::DISABLED,
      3 => BusinessProfileState::RESERVED_1,
      4 => BusinessProfileState::RESERVED_2,
      _ => BusinessProfileState(i)
    }
  }
}

impl From<&i32> for BusinessProfileState {
  fn from(i: &i32) -> Self {
    BusinessProfileState::from(*i)
  }
}

impl From<BusinessProfileState> for i32 {
  fn from(e: BusinessProfileState) -> i32 {
    e.0
  }
}

impl From<&BusinessProfileState> for i32 {
  fn from(e: &BusinessProfileState) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TranslatorType(pub i32);

impl TranslatorType {
  pub const NONE: TranslatorType = TranslatorType(0);
  pub const REGULAR: TranslatorType = TranslatorType(1);
  pub const BADGED: TranslatorType = TranslatorType(2);
  pub const MODERATOR: TranslatorType = TranslatorType(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NONE,
    Self::REGULAR,
    Self::BADGED,
    Self::MODERATOR,
  ];
}

impl TSerializable for TranslatorType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TranslatorType> {
    let enum_value = i_prot.read_i32()?;
    Ok(TranslatorType::from(enum_value))
  }
}

impl From<i32> for TranslatorType {
  fn from(i: i32) -> Self {
    match i {
      0 => TranslatorType::NONE,
      1 => TranslatorType::REGULAR,
      2 => TranslatorType::BADGED,
      3 => TranslatorType::MODERATOR,
      _ => TranslatorType(i)
    }
  }
}

impl From<&i32> for TranslatorType {
  fn from(i: &i32) -> Self {
    TranslatorType::from(*i)
  }
}

impl From<TranslatorType> for i32 {
  fn from(e: TranslatorType) -> i32 {
    e.0
  }
}

impl From<&TranslatorType> for i32 {
  fn from(e: &TranslatorType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CustomerServiceState(pub i32);

impl CustomerServiceState {
  pub const NONE: CustomerServiceState = CustomerServiceState(0);
  pub const ENABLED: CustomerServiceState = CustomerServiceState(1);
  pub const DISABLED: CustomerServiceState = CustomerServiceState(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NONE,
    Self::ENABLED,
    Self::DISABLED,
  ];
}

impl TSerializable for CustomerServiceState {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CustomerServiceState> {
    let enum_value = i_prot.read_i32()?;
    Ok(CustomerServiceState::from(enum_value))
  }
}

impl From<i32> for CustomerServiceState {
  fn from(i: i32) -> Self {
    match i {
      0 => CustomerServiceState::NONE,
      1 => CustomerServiceState::ENABLED,
      2 => CustomerServiceState::DISABLED,
      _ => CustomerServiceState(i)
    }
  }
}

impl From<&i32> for CustomerServiceState {
  fn from(i: &i32) -> Self {
    CustomerServiceState::from(*i)
  }
}

impl From<CustomerServiceState> for i32 {
  fn from(e: CustomerServiceState) -> i32 {
    e.0
  }
}

impl From<&CustomerServiceState> for i32 {
  fn from(e: &CustomerServiceState) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MentionFilter(pub i32);

impl MentionFilter {
  pub const UNFILTERED: MentionFilter = MentionFilter(0);
  pub const FILTERED: MentionFilter = MentionFilter(1);
  pub const VERIFIED: MentionFilter = MentionFilter(2);
  pub const FOLLOWING: MentionFilter = MentionFilter(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::UNFILTERED,
    Self::FILTERED,
    Self::VERIFIED,
    Self::FOLLOWING,
  ];
}

impl TSerializable for MentionFilter {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MentionFilter> {
    let enum_value = i_prot.read_i32()?;
    Ok(MentionFilter::from(enum_value))
  }
}

impl From<i32> for MentionFilter {
  fn from(i: i32) -> Self {
    match i {
      0 => MentionFilter::UNFILTERED,
      1 => MentionFilter::FILTERED,
      2 => MentionFilter::VERIFIED,
      3 => MentionFilter::FOLLOWING,
      _ => MentionFilter(i)
    }
  }
}

impl From<&i32> for MentionFilter {
  fn from(i: &i32) -> Self {
    MentionFilter::from(*i)
  }
}

impl From<MentionFilter> for i32 {
  fn from(e: MentionFilter) -> i32 {
    e.0
  }
}

impl From<&MentionFilter> for i32 {
  fn from(e: &MentionFilter) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NotificationsFilterQuality(pub i32);

impl NotificationsFilterQuality {
  pub const UNFILTERED: NotificationsFilterQuality = NotificationsFilterQuality(0);
  pub const FILTERED: NotificationsFilterQuality = NotificationsFilterQuality(1);
  pub const RESERVED_2: NotificationsFilterQuality = NotificationsFilterQuality(2);
  pub const RESERVED_3: NotificationsFilterQuality = NotificationsFilterQuality(3);
  pub const RESERVED_4: NotificationsFilterQuality = NotificationsFilterQuality(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::UNFILTERED,
    Self::FILTERED,
    Self::RESERVED_2,
    Self::RESERVED_3,
    Self::RESERVED_4,
  ];
}

impl TSerializable for NotificationsFilterQuality {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<NotificationsFilterQuality> {
    let enum_value = i_prot.read_i32()?;
    Ok(NotificationsFilterQuality::from(enum_value))
  }
}

impl From<i32> for NotificationsFilterQuality {
  fn from(i: i32) -> Self {
    match i {
      0 => NotificationsFilterQuality::UNFILTERED,
      1 => NotificationsFilterQuality::FILTERED,
      2 => NotificationsFilterQuality::RESERVED_2,
      3 => NotificationsFilterQuality::RESERVED_3,
      4 => NotificationsFilterQuality::RESERVED_4,
      _ => NotificationsFilterQuality(i)
    }
  }
}

impl From<&i32> for NotificationsFilterQuality {
  fn from(i: &i32) -> Self {
    NotificationsFilterQuality::from(*i)
  }
}

impl From<NotificationsFilterQuality> for i32 {
  fn from(e: NotificationsFilterQuality) -> i32 {
    e.0
  }
}

impl From<&NotificationsFilterQuality> for i32 {
  fn from(e: &NotificationsFilterQuality) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NotificationsAbuseFilterQuality(pub i32);

impl NotificationsAbuseFilterQuality {
  pub const UNFILTERED: NotificationsAbuseFilterQuality = NotificationsAbuseFilterQuality(0);
  pub const FILTERED: NotificationsAbuseFilterQuality = NotificationsAbuseFilterQuality(1);
  pub const RESERVED_2: NotificationsAbuseFilterQuality = NotificationsAbuseFilterQuality(2);
  pub const RESERVED_3: NotificationsAbuseFilterQuality = NotificationsAbuseFilterQuality(3);
  pub const RESERVED_4: NotificationsAbuseFilterQuality = NotificationsAbuseFilterQuality(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::UNFILTERED,
    Self::FILTERED,
    Self::RESERVED_2,
    Self::RESERVED_3,
    Self::RESERVED_4,
  ];
}

impl TSerializable for NotificationsAbuseFilterQuality {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<NotificationsAbuseFilterQuality> {
    let enum_value = i_prot.read_i32()?;
    Ok(NotificationsAbuseFilterQuality::from(enum_value))
  }
}

impl From<i32> for NotificationsAbuseFilterQuality {
  fn from(i: i32) -> Self {
    match i {
      0 => NotificationsAbuseFilterQuality::UNFILTERED,
      1 => NotificationsAbuseFilterQuality::FILTERED,
      2 => NotificationsAbuseFilterQuality::RESERVED_2,
      3 => NotificationsAbuseFilterQuality::RESERVED_3,
      4 => NotificationsAbuseFilterQuality::RESERVED_4,
      _ => NotificationsAbuseFilterQuality(i)
    }
  }
}

impl From<&i32> for NotificationsAbuseFilterQuality {
  fn from(i: &i32) -> Self {
    NotificationsAbuseFilterQuality::from(*i)
  }
}

impl From<NotificationsAbuseFilterQuality> for i32 {
  fn from(e: NotificationsAbuseFilterQuality) -> i32 {
    e.0
  }
}

impl From<&NotificationsAbuseFilterQuality> for i32 {
  fn from(e: &NotificationsAbuseFilterQuality) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AllowMediaTagging(pub i32);

impl AllowMediaTagging {
  pub const DEFAULT_VALUE: AllowMediaTagging = AllowMediaTagging(1);
  pub const ALL: AllowMediaTagging = AllowMediaTagging(2);
  pub const FOLLOWING: AllowMediaTagging = AllowMediaTagging(3);
  pub const NONE: AllowMediaTagging = AllowMediaTagging(4);
  pub const RESERVED_1: AllowMediaTagging = AllowMediaTagging(5);
  pub const RESERVED_2: AllowMediaTagging = AllowMediaTagging(6);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::DEFAULT_VALUE,
    Self::ALL,
    Self::FOLLOWING,
    Self::NONE,
    Self::RESERVED_1,
    Self::RESERVED_2,
  ];
}

impl TSerializable for AllowMediaTagging {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AllowMediaTagging> {
    let enum_value = i_prot.read_i32()?;
    Ok(AllowMediaTagging::from(enum_value))
  }
}

impl From<i32> for AllowMediaTagging {
  fn from(i: i32) -> Self {
    match i {
      1 => AllowMediaTagging::DEFAULT_VALUE,
      2 => AllowMediaTagging::ALL,
      3 => AllowMediaTagging::FOLLOWING,
      4 => AllowMediaTagging::NONE,
      5 => AllowMediaTagging::RESERVED_1,
      6 => AllowMediaTagging::RESERVED_2,
      _ => AllowMediaTagging(i)
    }
  }
}

impl From<&i32> for AllowMediaTagging {
  fn from(i: &i32) -> Self {
    AllowMediaTagging::from(*i)
  }
}

impl From<AllowMediaTagging> for i32 {
  fn from(e: AllowMediaTagging) -> i32 {
    e.0
  }
}

impl From<&AllowMediaTagging> for i32 {
  fn from(e: &AllowMediaTagging) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PersonaMediaGenAccess(pub i32);

impl PersonaMediaGenAccess {
  pub const ALL: PersonaMediaGenAccess = PersonaMediaGenAccess(1);
  pub const FOLLOWING: PersonaMediaGenAccess = PersonaMediaGenAccess(2);
  pub const NONE: PersonaMediaGenAccess = PersonaMediaGenAccess(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::ALL,
    Self::FOLLOWING,
    Self::NONE,
  ];
}

impl TSerializable for PersonaMediaGenAccess {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PersonaMediaGenAccess> {
    let enum_value = i_prot.read_i32()?;
    Ok(PersonaMediaGenAccess::from(enum_value))
  }
}

impl From<i32> for PersonaMediaGenAccess {
  fn from(i: i32) -> Self {
    match i {
      1 => PersonaMediaGenAccess::ALL,
      2 => PersonaMediaGenAccess::FOLLOWING,
      3 => PersonaMediaGenAccess::NONE,
      _ => PersonaMediaGenAccess(i)
    }
  }
}

impl From<&i32> for PersonaMediaGenAccess {
  fn from(i: &i32) -> Self {
    PersonaMediaGenAccess::from(*i)
  }
}

impl From<PersonaMediaGenAccess> for i32 {
  fn from(e: PersonaMediaGenAccess) -> i32 {
    e.0
  }
}

impl From<&PersonaMediaGenAccess> for i32 {
  fn from(e: &PersonaMediaGenAccess) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AllowDmsFrom(pub i32);

impl AllowDmsFrom {
  pub const FOLLOWING: AllowDmsFrom = AllowDmsFrom(0);
  pub const ANYONE: AllowDmsFrom = AllowDmsFrom(1);
  pub const VERIFIED: AllowDmsFrom = AllowDmsFrom(2);
  pub const RESERVED_3: AllowDmsFrom = AllowDmsFrom(3);
  pub const RESERVED_4: AllowDmsFrom = AllowDmsFrom(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::FOLLOWING,
    Self::ANYONE,
    Self::VERIFIED,
    Self::RESERVED_3,
    Self::RESERVED_4,
  ];
}

impl TSerializable for AllowDmsFrom {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AllowDmsFrom> {
    let enum_value = i_prot.read_i32()?;
    Ok(AllowDmsFrom::from(enum_value))
  }
}

impl From<i32> for AllowDmsFrom {
  fn from(i: i32) -> Self {
    match i {
      0 => AllowDmsFrom::FOLLOWING,
      1 => AllowDmsFrom::ANYONE,
      2 => AllowDmsFrom::VERIFIED,
      3 => AllowDmsFrom::RESERVED_3,
      4 => AllowDmsFrom::RESERVED_4,
      _ => AllowDmsFrom(i)
    }
  }
}

impl From<&i32> for AllowDmsFrom {
  fn from(i: &i32) -> Self {
    AllowDmsFrom::from(*i)
  }
}

impl From<AllowDmsFrom> for i32 {
  fn from(e: AllowDmsFrom) -> i32 {
    e.0
  }
}

impl From<&AllowDmsFrom> for i32 {
  fn from(e: &AllowDmsFrom) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AllowDmGroupsFrom(pub i32);

impl AllowDmGroupsFrom {
  pub const FOLLOWING: AllowDmGroupsFrom = AllowDmGroupsFrom(0);
  pub const VIT_FOLLOWING: AllowDmGroupsFrom = AllowDmGroupsFrom(1);
  pub const NOBODY: AllowDmGroupsFrom = AllowDmGroupsFrom(2);
  pub const RESERVED_3: AllowDmGroupsFrom = AllowDmGroupsFrom(3);
  pub const RESERVED_4: AllowDmGroupsFrom = AllowDmGroupsFrom(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::FOLLOWING,
    Self::VIT_FOLLOWING,
    Self::NOBODY,
    Self::RESERVED_3,
    Self::RESERVED_4,
  ];
}

impl TSerializable for AllowDmGroupsFrom {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AllowDmGroupsFrom> {
    let enum_value = i_prot.read_i32()?;
    Ok(AllowDmGroupsFrom::from(enum_value))
  }
}

impl From<i32> for AllowDmGroupsFrom {
  fn from(i: i32) -> Self {
    match i {
      0 => AllowDmGroupsFrom::FOLLOWING,
      1 => AllowDmGroupsFrom::VIT_FOLLOWING,
      2 => AllowDmGroupsFrom::NOBODY,
      3 => AllowDmGroupsFrom::RESERVED_3,
      4 => AllowDmGroupsFrom::RESERVED_4,
      _ => AllowDmGroupsFrom(i)
    }
  }
}

impl From<&i32> for AllowDmGroupsFrom {
  fn from(i: &i32) -> Self {
    AllowDmGroupsFrom::from(*i)
  }
}

impl From<AllowDmGroupsFrom> for i32 {
  fn from(e: AllowDmGroupsFrom) -> i32 {
    e.0
  }
}

impl From<&AllowDmGroupsFrom> for i32 {
  fn from(e: &AllowDmGroupsFrom) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AbuseFilter(pub i32);

impl AbuseFilter {
  pub const UNFILTERED: AbuseFilter = AbuseFilter(0);
  pub const SUPPRESSED: AbuseFilter = AbuseFilter(1);
  pub const RESERVED_2: AbuseFilter = AbuseFilter(2);
  pub const RESERVED_3: AbuseFilter = AbuseFilter(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::UNFILTERED,
    Self::SUPPRESSED,
    Self::RESERVED_2,
    Self::RESERVED_3,
  ];
}

impl TSerializable for AbuseFilter {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AbuseFilter> {
    let enum_value = i_prot.read_i32()?;
    Ok(AbuseFilter::from(enum_value))
  }
}

impl From<i32> for AbuseFilter {
  fn from(i: i32) -> Self {
    match i {
      0 => AbuseFilter::UNFILTERED,
      1 => AbuseFilter::SUPPRESSED,
      2 => AbuseFilter::RESERVED_2,
      3 => AbuseFilter::RESERVED_3,
      _ => AbuseFilter(i)
    }
  }
}

impl From<&i32> for AbuseFilter {
  fn from(i: &i32) -> Self {
    AbuseFilter::from(*i)
  }
}

impl From<AbuseFilter> for i32 {
  fn from(e: AbuseFilter) -> i32 {
    e.0
  }
}

impl From<&AbuseFilter> for i32 {
  fn from(e: &AbuseFilter) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AllowContributorRequest(pub i32);

impl AllowContributorRequest {
  pub const DEFAULT_VALUE: AllowContributorRequest = AllowContributorRequest(0);
  pub const ALL: AllowContributorRequest = AllowContributorRequest(1);
  pub const FOLLOWING: AllowContributorRequest = AllowContributorRequest(2);
  pub const NONE: AllowContributorRequest = AllowContributorRequest(3);
  pub const RESERVED_4: AllowContributorRequest = AllowContributorRequest(4);
  pub const RESERVED_5: AllowContributorRequest = AllowContributorRequest(5);
  pub const RESERVED_6: AllowContributorRequest = AllowContributorRequest(6);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::DEFAULT_VALUE,
    Self::ALL,
    Self::FOLLOWING,
    Self::NONE,
    Self::RESERVED_4,
    Self::RESERVED_5,
    Self::RESERVED_6,
  ];
}

impl TSerializable for AllowContributorRequest {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AllowContributorRequest> {
    let enum_value = i_prot.read_i32()?;
    Ok(AllowContributorRequest::from(enum_value))
  }
}

impl From<i32> for AllowContributorRequest {
  fn from(i: i32) -> Self {
    match i {
      0 => AllowContributorRequest::DEFAULT_VALUE,
      1 => AllowContributorRequest::ALL,
      2 => AllowContributorRequest::FOLLOWING,
      3 => AllowContributorRequest::NONE,
      4 => AllowContributorRequest::RESERVED_4,
      5 => AllowContributorRequest::RESERVED_5,
      6 => AllowContributorRequest::RESERVED_6,
      _ => AllowContributorRequest(i)
    }
  }
}

impl From<&i32> for AllowContributorRequest {
  fn from(i: &i32) -> Self {
    AllowContributorRequest::from(*i)
  }
}

impl From<AllowContributorRequest> for i32 {
  fn from(e: AllowContributorRequest) -> i32 {
    e.0
  }
}

impl From<&AllowContributorRequest> for i32 {
  fn from(e: &AllowContributorRequest) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AnalyticsType(pub i32);

impl AnalyticsType {
  pub const DISABLED: AnalyticsType = AnalyticsType(0);
  pub const ENABLED: AnalyticsType = AnalyticsType(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::DISABLED,
    Self::ENABLED,
  ];
}

impl TSerializable for AnalyticsType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AnalyticsType> {
    let enum_value = i_prot.read_i32()?;
    Ok(AnalyticsType::from(enum_value))
  }
}

impl From<i32> for AnalyticsType {
  fn from(i: i32) -> Self {
    match i {
      0 => AnalyticsType::DISABLED,
      1 => AnalyticsType::ENABLED,
      _ => AnalyticsType(i)
    }
  }
}

impl From<&i32> for AnalyticsType {
  fn from(i: &i32) -> Self {
    AnalyticsType::from(*i)
  }
}

impl From<AnalyticsType> for i32 {
  fn from(e: AnalyticsType) -> i32 {
    e.0
  }
}

impl From<&AnalyticsType> for i32 {
  fn from(e: &AnalyticsType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DmReceiptSetting(pub i32);

impl DmReceiptSetting {
  pub const ALL_ENABLED: DmReceiptSetting = DmReceiptSetting(0);
  pub const ALL_DISABLED: DmReceiptSetting = DmReceiptSetting(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::ALL_ENABLED,
    Self::ALL_DISABLED,
  ];
}

impl TSerializable for DmReceiptSetting {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DmReceiptSetting> {
    let enum_value = i_prot.read_i32()?;
    Ok(DmReceiptSetting::from(enum_value))
  }
}

impl From<i32> for DmReceiptSetting {
  fn from(i: i32) -> Self {
    match i {
      0 => DmReceiptSetting::ALL_ENABLED,
      1 => DmReceiptSetting::ALL_DISABLED,
      _ => DmReceiptSetting(i)
    }
  }
}

impl From<&i32> for DmReceiptSetting {
  fn from(i: &i32) -> Self {
    DmReceiptSetting::from(*i)
  }
}

impl From<DmReceiptSetting> for i32 {
  fn from(e: DmReceiptSetting) -> i32 {
    e.0
  }
}

impl From<&DmReceiptSetting> for i32 {
  fn from(e: &DmReceiptSetting) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DmQualityFilterSetting(pub i32);

impl DmQualityFilterSetting {
  pub const ENABLED: DmQualityFilterSetting = DmQualityFilterSetting(0);
  pub const DISABLED: DmQualityFilterSetting = DmQualityFilterSetting(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::ENABLED,
    Self::DISABLED,
  ];
}

impl TSerializable for DmQualityFilterSetting {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DmQualityFilterSetting> {
    let enum_value = i_prot.read_i32()?;
    Ok(DmQualityFilterSetting::from(enum_value))
  }
}

impl From<i32> for DmQualityFilterSetting {
  fn from(i: i32) -> Self {
    match i {
      0 => DmQualityFilterSetting::ENABLED,
      1 => DmQualityFilterSetting::DISABLED,
      _ => DmQualityFilterSetting(i)
    }
  }
}

impl From<&i32> for DmQualityFilterSetting {
  fn from(i: &i32) -> Self {
    DmQualityFilterSetting::from(*i)
  }
}

impl From<DmQualityFilterSetting> for i32 {
  fn from(e: DmQualityFilterSetting) -> i32 {
    e.0
  }
}

impl From<&DmQualityFilterSetting> for i32 {
  fn from(e: &DmQualityFilterSetting) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SmartBlockState(pub i32);

impl SmartBlockState {
  pub const DISABLED: SmartBlockState = SmartBlockState(0);
  pub const SELECTED: SmartBlockState = SmartBlockState(1);
  pub const FULL: SmartBlockState = SmartBlockState(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::DISABLED,
    Self::SELECTED,
    Self::FULL,
  ];
}

impl TSerializable for SmartBlockState {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SmartBlockState> {
    let enum_value = i_prot.read_i32()?;
    Ok(SmartBlockState::from(enum_value))
  }
}

impl From<i32> for SmartBlockState {
  fn from(i: i32) -> Self {
    match i {
      0 => SmartBlockState::DISABLED,
      1 => SmartBlockState::SELECTED,
      2 => SmartBlockState::FULL,
      _ => SmartBlockState(i)
    }
  }
}

impl From<&i32> for SmartBlockState {
  fn from(i: &i32) -> Self {
    SmartBlockState::from(*i)
  }
}

impl From<SmartBlockState> for i32 {
  fn from(e: SmartBlockState) -> i32 {
    e.0
  }
}

impl From<&SmartBlockState> for i32 {
  fn from(e: &SmartBlockState) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PCFLabel(pub i32);

impl PCFLabel {
  pub const NONE: PCFLabel = PCFLabel(0);
  pub const PARODY: PCFLabel = PCFLabel(1);
  pub const COMMENTARY: PCFLabel = PCFLabel(2);
  pub const FAN: PCFLabel = PCFLabel(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NONE,
    Self::PARODY,
    Self::COMMENTARY,
    Self::FAN,
  ];
}

impl TSerializable for PCFLabel {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PCFLabel> {
    let enum_value = i_prot.read_i32()?;
    Ok(PCFLabel::from(enum_value))
  }
}

impl From<i32> for PCFLabel {
  fn from(i: i32) -> Self {
    match i {
      0 => PCFLabel::NONE,
      1 => PCFLabel::PARODY,
      2 => PCFLabel::COMMENTARY,
      3 => PCFLabel::FAN,
      _ => PCFLabel(i)
    }
  }
}

impl From<&i32> for PCFLabel {
  fn from(i: &i32) -> Self {
    PCFLabel::from(*i)
  }
}

impl From<PCFLabel> for i32 {
  fn from(e: PCFLabel) -> i32 {
    e.0
  }
}

impl From<&PCFLabel> for i32 {
  fn from(e: &PCFLabel) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InferredLocationResolution(pub i32);

impl InferredLocationResolution {
  pub const REGION: InferredLocationResolution = InferredLocationResolution(0);
  pub const COUNTRY: InferredLocationResolution = InferredLocationResolution(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::REGION,
    Self::COUNTRY,
  ];
}

impl TSerializable for InferredLocationResolution {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<InferredLocationResolution> {
    let enum_value = i_prot.read_i32()?;
    Ok(InferredLocationResolution::from(enum_value))
  }
}

impl From<i32> for InferredLocationResolution {
  fn from(i: i32) -> Self {
    match i {
      0 => InferredLocationResolution::REGION,
      1 => InferredLocationResolution::COUNTRY,
      _ => InferredLocationResolution(i)
    }
  }
}

impl From<&i32> for InferredLocationResolution {
  fn from(i: &i32) -> Self {
    InferredLocationResolution::from(*i)
  }
}

impl From<InferredLocationResolution> for i32 {
  fn from(e: InferredLocationResolution) -> i32 {
    e.0
  }
}

impl From<&InferredLocationResolution> for i32 {
  fn from(e: &InferredLocationResolution) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GazeboFormStatus(pub i32);

impl GazeboFormStatus {
  pub const IN_PROGRESS: GazeboFormStatus = GazeboFormStatus(0);
  pub const DISMISSED: GazeboFormStatus = GazeboFormStatus(1);
  pub const COMPLETED: GazeboFormStatus = GazeboFormStatus(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::IN_PROGRESS,
    Self::DISMISSED,
    Self::COMPLETED,
  ];
}

impl TSerializable for GazeboFormStatus {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<GazeboFormStatus> {
    let enum_value = i_prot.read_i32()?;
    Ok(GazeboFormStatus::from(enum_value))
  }
}

impl From<i32> for GazeboFormStatus {
  fn from(i: i32) -> Self {
    match i {
      0 => GazeboFormStatus::IN_PROGRESS,
      1 => GazeboFormStatus::DISMISSED,
      2 => GazeboFormStatus::COMPLETED,
      _ => GazeboFormStatus(i)
    }
  }
}

impl From<&i32> for GazeboFormStatus {
  fn from(i: &i32) -> Self {
    GazeboFormStatus::from(*i)
  }
}

impl From<GazeboFormStatus> for i32 {
  fn from(e: GazeboFormStatus) -> i32 {
    e.0
  }
}

impl From<&GazeboFormStatus> for i32 {
  fn from(e: &GazeboFormStatus) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EmailSettingState(pub i32);

impl EmailSettingState {
  pub const NONE: EmailSettingState = EmailSettingState(0);
  pub const ALL: EmailSettingState = EmailSettingState(1);
  pub const FOLLOWINGS: EmailSettingState = EmailSettingState(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NONE,
    Self::ALL,
    Self::FOLLOWINGS,
  ];
}

impl TSerializable for EmailSettingState {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<EmailSettingState> {
    let enum_value = i_prot.read_i32()?;
    Ok(EmailSettingState::from(enum_value))
  }
}

impl From<i32> for EmailSettingState {
  fn from(i: i32) -> Self {
    match i {
      0 => EmailSettingState::NONE,
      1 => EmailSettingState::ALL,
      2 => EmailSettingState::FOLLOWINGS,
      _ => EmailSettingState(i)
    }
  }
}

impl From<&i32> for EmailSettingState {
  fn from(i: &i32) -> Self {
    EmailSettingState::from(*i)
  }
}

impl From<EmailSettingState> for i32 {
  fn from(e: EmailSettingState) -> i32 {
    e.0
  }
}

impl From<&EmailSettingState> for i32 {
  fn from(e: &EmailSettingState) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EmailConfirmedState(pub i32);

impl EmailConfirmedState {
  pub const GRANDFATHERED: EmailConfirmedState = EmailConfirmedState(0);
  pub const EXCUSED: EmailConfirmedState = EmailConfirmedState(1);
  pub const UNCONFIRMED: EmailConfirmedState = EmailConfirmedState(2);
  pub const CONFIRMED: EmailConfirmedState = EmailConfirmedState(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::GRANDFATHERED,
    Self::EXCUSED,
    Self::UNCONFIRMED,
    Self::CONFIRMED,
  ];
}

impl TSerializable for EmailConfirmedState {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<EmailConfirmedState> {
    let enum_value = i_prot.read_i32()?;
    Ok(EmailConfirmedState::from(enum_value))
  }
}

impl From<i32> for EmailConfirmedState {
  fn from(i: i32) -> Self {
    match i {
      0 => EmailConfirmedState::GRANDFATHERED,
      1 => EmailConfirmedState::EXCUSED,
      2 => EmailConfirmedState::UNCONFIRMED,
      3 => EmailConfirmedState::CONFIRMED,
      _ => EmailConfirmedState(i)
    }
  }
}

impl From<&i32> for EmailConfirmedState {
  fn from(i: &i32) -> Self {
    EmailConfirmedState::from(*i)
  }
}

impl From<EmailConfirmedState> for i32 {
  fn from(e: EmailConfirmedState) -> i32 {
    e.0
  }
}

impl From<&EmailConfirmedState> for i32 {
  fn from(e: &EmailConfirmedState) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DigestSchedule(pub i32);

impl DigestSchedule {
  pub const NONE: DigestSchedule = DigestSchedule(0);
  pub const DAILY: DigestSchedule = DigestSchedule(1);
  pub const TWO_DAYS: DigestSchedule = DigestSchedule(2);
  pub const WEEKLY: DigestSchedule = DigestSchedule(3);
  pub const PERIODICALLY: DigestSchedule = DigestSchedule(4);
  pub const RESERVED_5: DigestSchedule = DigestSchedule(5);
  pub const RESERVED_6: DigestSchedule = DigestSchedule(6);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NONE,
    Self::DAILY,
    Self::TWO_DAYS,
    Self::WEEKLY,
    Self::PERIODICALLY,
    Self::RESERVED_5,
    Self::RESERVED_6,
  ];
}

impl TSerializable for DigestSchedule {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DigestSchedule> {
    let enum_value = i_prot.read_i32()?;
    Ok(DigestSchedule::from(enum_value))
  }
}

impl From<i32> for DigestSchedule {
  fn from(i: i32) -> Self {
    match i {
      0 => DigestSchedule::NONE,
      1 => DigestSchedule::DAILY,
      2 => DigestSchedule::TWO_DAYS,
      3 => DigestSchedule::WEEKLY,
      4 => DigestSchedule::PERIODICALLY,
      5 => DigestSchedule::RESERVED_5,
      6 => DigestSchedule::RESERVED_6,
      _ => DigestSchedule(i)
    }
  }
}

impl From<&i32> for DigestSchedule {
  fn from(i: &i32) -> Self {
    DigestSchedule::from(*i)
  }
}

impl From<DigestSchedule> for i32 {
  fn from(e: DigestSchedule) -> i32 {
    e.0
  }
}

impl From<&DigestSchedule> for i32 {
  fn from(e: &DigestSchedule) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FrictionlessFollowerState(pub i32);

impl FrictionlessFollowerState {
  pub const NORMAL: FrictionlessFollowerState = FrictionlessFollowerState(0);
  pub const FRICTIONLESS: FrictionlessFollowerState = FrictionlessFollowerState(1);
  pub const CONVERTED: FrictionlessFollowerState = FrictionlessFollowerState(2);
  pub const MERGED: FrictionlessFollowerState = FrictionlessFollowerState(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NORMAL,
    Self::FRICTIONLESS,
    Self::CONVERTED,
    Self::MERGED,
  ];
}

impl TSerializable for FrictionlessFollowerState {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FrictionlessFollowerState> {
    let enum_value = i_prot.read_i32()?;
    Ok(FrictionlessFollowerState::from(enum_value))
  }
}

impl From<i32> for FrictionlessFollowerState {
  fn from(i: i32) -> Self {
    match i {
      0 => FrictionlessFollowerState::NORMAL,
      1 => FrictionlessFollowerState::FRICTIONLESS,
      2 => FrictionlessFollowerState::CONVERTED,
      3 => FrictionlessFollowerState::MERGED,
      _ => FrictionlessFollowerState(i)
    }
  }
}

impl From<&i32> for FrictionlessFollowerState {
  fn from(i: &i32) -> Self {
    FrictionlessFollowerState::from(*i)
  }
}

impl From<FrictionlessFollowerState> for i32 {
  fn from(e: FrictionlessFollowerState) -> i32 {
    e.0
  }
}

impl From<&FrictionlessFollowerState> for i32 {
  fn from(e: &FrictionlessFollowerState) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FrictionlessFollowerType(pub i32);

impl FrictionlessFollowerType {
  pub const SMS: FrictionlessFollowerType = FrictionlessFollowerType(0);
  pub const EMAIL: FrictionlessFollowerType = FrictionlessFollowerType(1);
  pub const SDK: FrictionlessFollowerType = FrictionlessFollowerType(2);
  pub const RESERVED_3: FrictionlessFollowerType = FrictionlessFollowerType(3);
  pub const RESERVED_4: FrictionlessFollowerType = FrictionlessFollowerType(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::SMS,
    Self::EMAIL,
    Self::SDK,
    Self::RESERVED_3,
    Self::RESERVED_4,
  ];
}

impl TSerializable for FrictionlessFollowerType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FrictionlessFollowerType> {
    let enum_value = i_prot.read_i32()?;
    Ok(FrictionlessFollowerType::from(enum_value))
  }
}

impl From<i32> for FrictionlessFollowerType {
  fn from(i: i32) -> Self {
    match i {
      0 => FrictionlessFollowerType::SMS,
      1 => FrictionlessFollowerType::EMAIL,
      2 => FrictionlessFollowerType::SDK,
      3 => FrictionlessFollowerType::RESERVED_3,
      4 => FrictionlessFollowerType::RESERVED_4,
      _ => FrictionlessFollowerType(i)
    }
  }
}

impl From<&i32> for FrictionlessFollowerType {
  fn from(i: &i32) -> Self {
    FrictionlessFollowerType::from(*i)
  }
}

impl From<FrictionlessFollowerType> for i32 {
  fn from(e: FrictionlessFollowerType) -> i32 {
    e.0
  }
}

impl From<&FrictionlessFollowerType> for i32 {
  fn from(e: &FrictionlessFollowerType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AccessPolicy(pub i32);

impl AccessPolicy {
  pub const NORMAL: AccessPolicy = AccessPolicy(0);
  pub const BOUNCE_ALL: AccessPolicy = AccessPolicy(1);
  pub const BOUNCE_ALL_WRITES_AND_NPCI: AccessPolicy = AccessPolicy(2);
  pub const BOUNCE_ALL_PUBLIC_WRITES: AccessPolicy = AccessPolicy(3);
  pub const BOUNCE_ON_UNSUSPENSION: AccessPolicy = AccessPolicy(4);
  pub const RESERVED_5: AccessPolicy = AccessPolicy(5);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NORMAL,
    Self::BOUNCE_ALL,
    Self::BOUNCE_ALL_WRITES_AND_NPCI,
    Self::BOUNCE_ALL_PUBLIC_WRITES,
    Self::BOUNCE_ON_UNSUSPENSION,
    Self::RESERVED_5,
  ];
}

impl TSerializable for AccessPolicy {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AccessPolicy> {
    let enum_value = i_prot.read_i32()?;
    Ok(AccessPolicy::from(enum_value))
  }
}

impl From<i32> for AccessPolicy {
  fn from(i: i32) -> Self {
    match i {
      0 => AccessPolicy::NORMAL,
      1 => AccessPolicy::BOUNCE_ALL,
      2 => AccessPolicy::BOUNCE_ALL_WRITES_AND_NPCI,
      3 => AccessPolicy::BOUNCE_ALL_PUBLIC_WRITES,
      4 => AccessPolicy::BOUNCE_ON_UNSUSPENSION,
      5 => AccessPolicy::RESERVED_5,
      _ => AccessPolicy(i)
    }
  }
}

impl From<&i32> for AccessPolicy {
  fn from(i: &i32) -> Self {
    AccessPolicy::from(*i)
  }
}

impl From<AccessPolicy> for i32 {
  fn from(e: AccessPolicy) -> i32 {
    e.0
  }
}

impl From<&AccessPolicy> for i32 {
  fn from(e: &AccessPolicy) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SignupCategory(pub i32);

impl SignupCategory {
  pub const GRANDFATHERED: SignupCategory = SignupCategory(0);
  pub const API_PHONE: SignupCategory = SignupCategory(1);
  pub const WEB_PHONE: SignupCategory = SignupCategory(2);
  pub const WEB_EMAIL: SignupCategory = SignupCategory(3);
  pub const API_EMAIL: SignupCategory = SignupCategory(4);
  pub const CONVERSION: SignupCategory = SignupCategory(5);
  pub const RESERVED_2: SignupCategory = SignupCategory(6);
  pub const RESERVED_3: SignupCategory = SignupCategory(7);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::GRANDFATHERED,
    Self::API_PHONE,
    Self::WEB_PHONE,
    Self::WEB_EMAIL,
    Self::API_EMAIL,
    Self::CONVERSION,
    Self::RESERVED_2,
    Self::RESERVED_3,
  ];
}

impl TSerializable for SignupCategory {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SignupCategory> {
    let enum_value = i_prot.read_i32()?;
    Ok(SignupCategory::from(enum_value))
  }
}

impl From<i32> for SignupCategory {
  fn from(i: i32) -> Self {
    match i {
      0 => SignupCategory::GRANDFATHERED,
      1 => SignupCategory::API_PHONE,
      2 => SignupCategory::WEB_PHONE,
      3 => SignupCategory::WEB_EMAIL,
      4 => SignupCategory::API_EMAIL,
      5 => SignupCategory::CONVERSION,
      6 => SignupCategory::RESERVED_2,
      7 => SignupCategory::RESERVED_3,
      _ => SignupCategory(i)
    }
  }
}

impl From<&i32> for SignupCategory {
  fn from(i: &i32) -> Self {
    SignupCategory::from(*i)
  }
}

impl From<SignupCategory> for i32 {
  fn from(e: SignupCategory) -> i32 {
    e.0
  }
}

impl From<&SignupCategory> for i32 {
  fn from(e: &SignupCategory) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SignupTrustLevel(pub i32);

impl SignupTrustLevel {
  pub const UNKNOWN: SignupTrustLevel = SignupTrustLevel(0);
  pub const FULL: SignupTrustLevel = SignupTrustLevel(1);
  pub const HIGH: SignupTrustLevel = SignupTrustLevel(2);
  pub const MEDIUM: SignupTrustLevel = SignupTrustLevel(3);
  pub const RESERVED_1: SignupTrustLevel = SignupTrustLevel(4);
  pub const RESERVED_2: SignupTrustLevel = SignupTrustLevel(5);
  pub const RESERVED_3: SignupTrustLevel = SignupTrustLevel(6);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::UNKNOWN,
    Self::FULL,
    Self::HIGH,
    Self::MEDIUM,
    Self::RESERVED_1,
    Self::RESERVED_2,
    Self::RESERVED_3,
  ];
}

impl TSerializable for SignupTrustLevel {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SignupTrustLevel> {
    let enum_value = i_prot.read_i32()?;
    Ok(SignupTrustLevel::from(enum_value))
  }
}

impl From<i32> for SignupTrustLevel {
  fn from(i: i32) -> Self {
    match i {
      0 => SignupTrustLevel::UNKNOWN,
      1 => SignupTrustLevel::FULL,
      2 => SignupTrustLevel::HIGH,
      3 => SignupTrustLevel::MEDIUM,
      4 => SignupTrustLevel::RESERVED_1,
      5 => SignupTrustLevel::RESERVED_2,
      6 => SignupTrustLevel::RESERVED_3,
      _ => SignupTrustLevel(i)
    }
  }
}

impl From<&i32> for SignupTrustLevel {
  fn from(i: &i32) -> Self {
    SignupTrustLevel::from(*i)
  }
}

impl From<SignupTrustLevel> for i32 {
  fn from(e: SignupTrustLevel) -> i32 {
    e.0
  }
}

impl From<&SignupTrustLevel> for i32 {
  fn from(e: &SignupTrustLevel) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UniversalQualityFiltering(pub i32);

impl UniversalQualityFiltering {
  pub const ENABLED: UniversalQualityFiltering = UniversalQualityFiltering(0);
  pub const DISABLED: UniversalQualityFiltering = UniversalQualityFiltering(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::ENABLED,
    Self::DISABLED,
  ];
}

impl TSerializable for UniversalQualityFiltering {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UniversalQualityFiltering> {
    let enum_value = i_prot.read_i32()?;
    Ok(UniversalQualityFiltering::from(enum_value))
  }
}

impl From<i32> for UniversalQualityFiltering {
  fn from(i: i32) -> Self {
    match i {
      0 => UniversalQualityFiltering::ENABLED,
      1 => UniversalQualityFiltering::DISABLED,
      _ => UniversalQualityFiltering(i)
    }
  }
}

impl From<&i32> for UniversalQualityFiltering {
  fn from(i: &i32) -> Self {
    UniversalQualityFiltering::from(*i)
  }
}

impl From<UniversalQualityFiltering> for i32 {
  fn from(e: UniversalQualityFiltering) -> i32 {
    e.0
  }
}

impl From<&UniversalQualityFiltering> for i32 {
  fn from(e: &UniversalQualityFiltering) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConsentFlow(pub i32);

impl ConsentFlow {
  pub const UNKNOWN: ConsentFlow = ConsentFlow(0);
  pub const SIGNUP: ConsentFlow = ConsentFlow(1);
  pub const EXISTING_USER: ConsentFlow = ConsentFlow(2);
  pub const COUNTRY_CHANGE: ConsentFlow = ConsentFlow(3);
  pub const BIRTHDATE_CHANGE: ConsentFlow = ConsentFlow(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::UNKNOWN,
    Self::SIGNUP,
    Self::EXISTING_USER,
    Self::COUNTRY_CHANGE,
    Self::BIRTHDATE_CHANGE,
  ];
}

impl TSerializable for ConsentFlow {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConsentFlow> {
    let enum_value = i_prot.read_i32()?;
    Ok(ConsentFlow::from(enum_value))
  }
}

impl From<i32> for ConsentFlow {
  fn from(i: i32) -> Self {
    match i {
      0 => ConsentFlow::UNKNOWN,
      1 => ConsentFlow::SIGNUP,
      2 => ConsentFlow::EXISTING_USER,
      3 => ConsentFlow::COUNTRY_CHANGE,
      4 => ConsentFlow::BIRTHDATE_CHANGE,
      _ => ConsentFlow(i)
    }
  }
}

impl From<&i32> for ConsentFlow {
  fn from(i: &i32) -> Self {
    ConsentFlow::from(*i)
  }
}

impl From<ConsentFlow> for i32 {
  fn from(e: ConsentFlow) -> i32 {
    e.0
  }
}

impl From<&ConsentFlow> for i32 {
  fn from(e: &ConsentFlow) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConsentResponseValue(pub i32);

impl ConsentResponseValue {
  pub const DECLINE: ConsentResponseValue = ConsentResponseValue(0);
  pub const ACCEPT: ConsentResponseValue = ConsentResponseValue(1);
  pub const DECLINE_PARENTAL: ConsentResponseValue = ConsentResponseValue(2);
  pub const ACCEPT_PARENTAL: ConsentResponseValue = ConsentResponseValue(3);
  pub const DECLINE_AGENT: ConsentResponseValue = ConsentResponseValue(4);
  pub const ACCEPT_SYSTEM: ConsentResponseValue = ConsentResponseValue(5);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::DECLINE,
    Self::ACCEPT,
    Self::DECLINE_PARENTAL,
    Self::ACCEPT_PARENTAL,
    Self::DECLINE_AGENT,
    Self::ACCEPT_SYSTEM,
  ];
}

impl TSerializable for ConsentResponseValue {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConsentResponseValue> {
    let enum_value = i_prot.read_i32()?;
    Ok(ConsentResponseValue::from(enum_value))
  }
}

impl From<i32> for ConsentResponseValue {
  fn from(i: i32) -> Self {
    match i {
      0 => ConsentResponseValue::DECLINE,
      1 => ConsentResponseValue::ACCEPT,
      2 => ConsentResponseValue::DECLINE_PARENTAL,
      3 => ConsentResponseValue::ACCEPT_PARENTAL,
      4 => ConsentResponseValue::DECLINE_AGENT,
      5 => ConsentResponseValue::ACCEPT_SYSTEM,
      _ => ConsentResponseValue(i)
    }
  }
}

impl From<&i32> for ConsentResponseValue {
  fn from(i: &i32) -> Self {
    ConsentResponseValue::from(*i)
  }
}

impl From<ConsentResponseValue> for i32 {
  fn from(e: ConsentResponseValue) -> i32 {
    e.0
  }
}

impl From<&ConsentResponseValue> for i32 {
  fn from(e: &ConsentResponseValue) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DerivedConsentIndicator(pub i32);

impl DerivedConsentIndicator {
  pub const NONE: DerivedConsentIndicator = DerivedConsentIndicator(0);
  pub const NO_BIRTHDATE: DerivedConsentIndicator = DerivedConsentIndicator(1);
  pub const UNDERAGE_WHEN_CREATED: DerivedConsentIndicator = DerivedConsentIndicator(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NONE,
    Self::NO_BIRTHDATE,
    Self::UNDERAGE_WHEN_CREATED,
  ];
}

impl TSerializable for DerivedConsentIndicator {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DerivedConsentIndicator> {
    let enum_value = i_prot.read_i32()?;
    Ok(DerivedConsentIndicator::from(enum_value))
  }
}

impl From<i32> for DerivedConsentIndicator {
  fn from(i: i32) -> Self {
    match i {
      0 => DerivedConsentIndicator::NONE,
      1 => DerivedConsentIndicator::NO_BIRTHDATE,
      2 => DerivedConsentIndicator::UNDERAGE_WHEN_CREATED,
      _ => DerivedConsentIndicator(i)
    }
  }
}

impl From<&i32> for DerivedConsentIndicator {
  fn from(i: &i32) -> Self {
    DerivedConsentIndicator::from(*i)
  }
}

impl From<DerivedConsentIndicator> for i32 {
  fn from(e: DerivedConsentIndicator) -> i32 {
    e.0
  }
}

impl From<&DerivedConsentIndicator> for i32 {
  fn from(e: &DerivedConsentIndicator) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U13RemediationReason(pub i32);

impl U13RemediationReason {
  pub const UNDER_GLOBAL_MINIMUM_AGE: U13RemediationReason = U13RemediationReason(0);
  pub const PARENTAL_ATTEST: U13RemediationReason = U13RemediationReason(1);
  pub const INCORRECT_BIRTHDATE: U13RemediationReason = U13RemediationReason(2);
  pub const BUSINESS_ACCOUNT: U13RemediationReason = U13RemediationReason(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::UNDER_GLOBAL_MINIMUM_AGE,
    Self::PARENTAL_ATTEST,
    Self::INCORRECT_BIRTHDATE,
    Self::BUSINESS_ACCOUNT,
  ];
}

impl TSerializable for U13RemediationReason {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<U13RemediationReason> {
    let enum_value = i_prot.read_i32()?;
    Ok(U13RemediationReason::from(enum_value))
  }
}

impl From<i32> for U13RemediationReason {
  fn from(i: i32) -> Self {
    match i {
      0 => U13RemediationReason::UNDER_GLOBAL_MINIMUM_AGE,
      1 => U13RemediationReason::PARENTAL_ATTEST,
      2 => U13RemediationReason::INCORRECT_BIRTHDATE,
      3 => U13RemediationReason::BUSINESS_ACCOUNT,
      _ => U13RemediationReason(i)
    }
  }
}

impl From<&i32> for U13RemediationReason {
  fn from(i: &i32) -> Self {
    U13RemediationReason::from(*i)
  }
}

impl From<U13RemediationReason> for i32 {
  fn from(e: U13RemediationReason) -> i32 {
    e.0
  }
}

impl From<&U13RemediationReason> for i32 {
  fn from(e: &U13RemediationReason) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U13RemediationState(pub i32);

impl U13RemediationState {
  pub const REQUESTED: U13RemediationState = U13RemediationState(0);
  pub const CONFIRMED: U13RemediationState = U13RemediationState(1);
  pub const REJECTED: U13RemediationState = U13RemediationState(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::REQUESTED,
    Self::CONFIRMED,
    Self::REJECTED,
  ];
}

impl TSerializable for U13RemediationState {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<U13RemediationState> {
    let enum_value = i_prot.read_i32()?;
    Ok(U13RemediationState::from(enum_value))
  }
}

impl From<i32> for U13RemediationState {
  fn from(i: i32) -> Self {
    match i {
      0 => U13RemediationState::REQUESTED,
      1 => U13RemediationState::CONFIRMED,
      2 => U13RemediationState::REJECTED,
      _ => U13RemediationState(i)
    }
  }
}

impl From<&i32> for U13RemediationState {
  fn from(i: &i32) -> Self {
    U13RemediationState::from(*i)
  }
}

impl From<U13RemediationState> for i32 {
  fn from(e: U13RemediationState) -> i32 {
    e.0
  }
}

impl From<&U13RemediationState> for i32 {
  fn from(e: &U13RemediationState) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U13RestorationStatus(pub i32);

impl U13RestorationStatus {
  pub const STARTED: U13RestorationStatus = U13RestorationStatus(0);
  pub const IN_PROGRESS: U13RestorationStatus = U13RestorationStatus(1);
  pub const COMPLETED: U13RestorationStatus = U13RestorationStatus(2);
  pub const APPEAL_IN_PROGRESS: U13RestorationStatus = U13RestorationStatus(3);
  pub const APPEAL_DENIED: U13RestorationStatus = U13RestorationStatus(4);
  pub const APPEAL_SUCCEEDED: U13RestorationStatus = U13RestorationStatus(5);
  pub const APPEAL_COMPLETED: U13RestorationStatus = U13RestorationStatus(6);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::STARTED,
    Self::IN_PROGRESS,
    Self::COMPLETED,
    Self::APPEAL_IN_PROGRESS,
    Self::APPEAL_DENIED,
    Self::APPEAL_SUCCEEDED,
    Self::APPEAL_COMPLETED,
  ];
}

impl TSerializable for U13RestorationStatus {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<U13RestorationStatus> {
    let enum_value = i_prot.read_i32()?;
    Ok(U13RestorationStatus::from(enum_value))
  }
}

impl From<i32> for U13RestorationStatus {
  fn from(i: i32) -> Self {
    match i {
      0 => U13RestorationStatus::STARTED,
      1 => U13RestorationStatus::IN_PROGRESS,
      2 => U13RestorationStatus::COMPLETED,
      3 => U13RestorationStatus::APPEAL_IN_PROGRESS,
      4 => U13RestorationStatus::APPEAL_DENIED,
      5 => U13RestorationStatus::APPEAL_SUCCEEDED,
      6 => U13RestorationStatus::APPEAL_COMPLETED,
      _ => U13RestorationStatus(i)
    }
  }
}

impl From<&i32> for U13RestorationStatus {
  fn from(i: &i32) -> Self {
    U13RestorationStatus::from(*i)
  }
}

impl From<U13RestorationStatus> for i32 {
  fn from(e: U13RestorationStatus) -> i32 {
    e.0
  }
}

impl From<&U13RestorationStatus> for i32 {
  fn from(e: &U13RestorationStatus) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeactivationTimespan(pub i32);

impl DeactivationTimespan {
  pub const RETAIN_30_DAYS: DeactivationTimespan = DeactivationTimespan(0);
  pub const RETAIN_1_YEAR: DeactivationTimespan = DeactivationTimespan(1);
  pub const RETAIN_1_DAY: DeactivationTimespan = DeactivationTimespan(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::RETAIN_30_DAYS,
    Self::RETAIN_1_YEAR,
    Self::RETAIN_1_DAY,
  ];
}

impl TSerializable for DeactivationTimespan {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DeactivationTimespan> {
    let enum_value = i_prot.read_i32()?;
    Ok(DeactivationTimespan::from(enum_value))
  }
}

impl From<i32> for DeactivationTimespan {
  fn from(i: i32) -> Self {
    match i {
      0 => DeactivationTimespan::RETAIN_30_DAYS,
      1 => DeactivationTimespan::RETAIN_1_YEAR,
      2 => DeactivationTimespan::RETAIN_1_DAY,
      _ => DeactivationTimespan(i)
    }
  }
}

impl From<&i32> for DeactivationTimespan {
  fn from(i: &i32) -> Self {
    DeactivationTimespan::from(*i)
  }
}

impl From<DeactivationTimespan> for i32 {
  fn from(e: DeactivationTimespan) -> i32 {
    e.0
  }
}

impl From<&DeactivationTimespan> for i32 {
  fn from(e: &DeactivationTimespan) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CompromisedState(pub i32);

impl CompromisedState {
  pub const NOT_COMPROMISED: CompromisedState = CompromisedState(0);
  pub const RECENTLY_COMPROMISED: CompromisedState = CompromisedState(1);
  pub const ARCHIVED: CompromisedState = CompromisedState(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NOT_COMPROMISED,
    Self::RECENTLY_COMPROMISED,
    Self::ARCHIVED,
  ];
}

impl TSerializable for CompromisedState {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CompromisedState> {
    let enum_value = i_prot.read_i32()?;
    Ok(CompromisedState::from(enum_value))
  }
}

impl From<i32> for CompromisedState {
  fn from(i: i32) -> Self {
    match i {
      0 => CompromisedState::NOT_COMPROMISED,
      1 => CompromisedState::RECENTLY_COMPROMISED,
      2 => CompromisedState::ARCHIVED,
      _ => CompromisedState(i)
    }
  }
}

impl From<&i32> for CompromisedState {
  fn from(i: &i32) -> Self {
    CompromisedState::from(*i)
  }
}

impl From<CompromisedState> for i32 {
  fn from(e: CompromisedState) -> i32 {
    e.0
  }
}

impl From<&CompromisedState> for i32 {
  fn from(e: &CompromisedState) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VerifiedType(pub i32);

impl VerifiedType {
  pub const USER: VerifiedType = VerifiedType(1);
  pub const NOTABLE: VerifiedType = VerifiedType(2);
  pub const BUSINESS: VerifiedType = VerifiedType(3);
  pub const GOVERNMENT: VerifiedType = VerifiedType(4);
  pub const RESERVED_4: VerifiedType = VerifiedType(5);
  pub const RESERVED_5: VerifiedType = VerifiedType(6);
  pub const RESERVED_6: VerifiedType = VerifiedType(7);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::USER,
    Self::NOTABLE,
    Self::BUSINESS,
    Self::GOVERNMENT,
    Self::RESERVED_4,
    Self::RESERVED_5,
    Self::RESERVED_6,
  ];
}

impl TSerializable for VerifiedType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<VerifiedType> {
    let enum_value = i_prot.read_i32()?;
    Ok(VerifiedType::from(enum_value))
  }
}

impl From<i32> for VerifiedType {
  fn from(i: i32) -> Self {
    match i {
      1 => VerifiedType::USER,
      2 => VerifiedType::NOTABLE,
      3 => VerifiedType::BUSINESS,
      4 => VerifiedType::GOVERNMENT,
      5 => VerifiedType::RESERVED_4,
      6 => VerifiedType::RESERVED_5,
      7 => VerifiedType::RESERVED_6,
      _ => VerifiedType(i)
    }
  }
}

impl From<&i32> for VerifiedType {
  fn from(i: &i32) -> Self {
    VerifiedType::from(*i)
  }
}

impl From<VerifiedType> for i32 {
  fn from(e: VerifiedType) -> i32 {
    e.0
  }
}

impl From<&VerifiedType> for i32 {
  fn from(e: &VerifiedType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlueCheckmarkHiddenReason(pub i32);

impl BlueCheckmarkHiddenReason {
  pub const MANUALLY_HIDDEN: BlueCheckmarkHiddenReason = BlueCheckmarkHiddenReason(1);
  pub const UNDER_REVIEW: BlueCheckmarkHiddenReason = BlueCheckmarkHiddenReason(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::MANUALLY_HIDDEN,
    Self::UNDER_REVIEW,
  ];
}

impl TSerializable for BlueCheckmarkHiddenReason {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<BlueCheckmarkHiddenReason> {
    let enum_value = i_prot.read_i32()?;
    Ok(BlueCheckmarkHiddenReason::from(enum_value))
  }
}

impl From<i32> for BlueCheckmarkHiddenReason {
  fn from(i: i32) -> Self {
    match i {
      1 => BlueCheckmarkHiddenReason::MANUALLY_HIDDEN,
      2 => BlueCheckmarkHiddenReason::UNDER_REVIEW,
      _ => BlueCheckmarkHiddenReason(i)
    }
  }
}

impl From<&i32> for BlueCheckmarkHiddenReason {
  fn from(i: &i32) -> Self {
    BlueCheckmarkHiddenReason::from(*i)
  }
}

impl From<BlueCheckmarkHiddenReason> for i32 {
  fn from(e: BlueCheckmarkHiddenReason) -> i32 {
    e.0
  }
}

impl From<&BlueCheckmarkHiddenReason> for i32 {
  fn from(e: &BlueCheckmarkHiddenReason) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubscriptionLevel(pub i32);

impl SubscriptionLevel {
  pub const BASIC: SubscriptionLevel = SubscriptionLevel(1);
  pub const PREMIUM: SubscriptionLevel = SubscriptionLevel(2);
  pub const PREMIUM_PLUS: SubscriptionLevel = SubscriptionLevel(3);
  pub const RESERVED_4: SubscriptionLevel = SubscriptionLevel(4);
  pub const RESERVED_5: SubscriptionLevel = SubscriptionLevel(5);
  pub const RESERVED_6: SubscriptionLevel = SubscriptionLevel(6);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::BASIC,
    Self::PREMIUM,
    Self::PREMIUM_PLUS,
    Self::RESERVED_4,
    Self::RESERVED_5,
    Self::RESERVED_6,
  ];
}

impl TSerializable for SubscriptionLevel {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SubscriptionLevel> {
    let enum_value = i_prot.read_i32()?;
    Ok(SubscriptionLevel::from(enum_value))
  }
}

impl From<i32> for SubscriptionLevel {
  fn from(i: i32) -> Self {
    match i {
      1 => SubscriptionLevel::BASIC,
      2 => SubscriptionLevel::PREMIUM,
      3 => SubscriptionLevel::PREMIUM_PLUS,
      4 => SubscriptionLevel::RESERVED_4,
      5 => SubscriptionLevel::RESERVED_5,
      6 => SubscriptionLevel::RESERVED_6,
      _ => SubscriptionLevel(i)
    }
  }
}

impl From<&i32> for SubscriptionLevel {
  fn from(i: &i32) -> Self {
    SubscriptionLevel::from(*i)
  }
}

impl From<SubscriptionLevel> for i32 {
  fn from(e: SubscriptionLevel) -> i32 {
    e.0
  }
}

impl From<&SubscriptionLevel> for i32 {
  fn from(e: &SubscriptionLevel) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SignupCreationSource(pub i32);

impl SignupCreationSource {
  pub const OCF: SignupCreationSource = SignupCreationSource(1);
  pub const ONBOARD: SignupCreationSource = SignupCreationSource(2);
  pub const MARCH_MADNESS: SignupCreationSource = SignupCreationSource(3);
  pub const RESERVED_4: SignupCreationSource = SignupCreationSource(4);
  pub const RESERVED_5: SignupCreationSource = SignupCreationSource(5);
  pub const RESERVED_6: SignupCreationSource = SignupCreationSource(6);
  pub const RESERVED_7: SignupCreationSource = SignupCreationSource(7);
  pub const RESERVED_8: SignupCreationSource = SignupCreationSource(8);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::OCF,
    Self::ONBOARD,
    Self::MARCH_MADNESS,
    Self::RESERVED_4,
    Self::RESERVED_5,
    Self::RESERVED_6,
    Self::RESERVED_7,
    Self::RESERVED_8,
  ];
}

impl TSerializable for SignupCreationSource {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SignupCreationSource> {
    let enum_value = i_prot.read_i32()?;
    Ok(SignupCreationSource::from(enum_value))
  }
}

impl From<i32> for SignupCreationSource {
  fn from(i: i32) -> Self {
    match i {
      1 => SignupCreationSource::OCF,
      2 => SignupCreationSource::ONBOARD,
      3 => SignupCreationSource::MARCH_MADNESS,
      4 => SignupCreationSource::RESERVED_4,
      5 => SignupCreationSource::RESERVED_5,
      6 => SignupCreationSource::RESERVED_6,
      7 => SignupCreationSource::RESERVED_7,
      8 => SignupCreationSource::RESERVED_8,
      _ => SignupCreationSource(i)
    }
  }
}

impl From<&i32> for SignupCreationSource {
  fn from(i: &i32) -> Self {
    SignupCreationSource::from(*i)
  }
}

impl From<SignupCreationSource> for i32 {
  fn from(e: SignupCreationSource) -> i32 {
    e.0
  }
}

impl From<&SignupCreationSource> for i32 {
  fn from(e: &SignupCreationSource) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContributorAccessLevel(pub i32);

impl ContributorAccessLevel {
  pub const PARTIAL: ContributorAccessLevel = ContributorAccessLevel(0);
  pub const FULL: ContributorAccessLevel = ContributorAccessLevel(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PARTIAL,
    Self::FULL,
  ];
}

impl TSerializable for ContributorAccessLevel {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ContributorAccessLevel> {
    let enum_value = i_prot.read_i32()?;
    Ok(ContributorAccessLevel::from(enum_value))
  }
}

impl From<i32> for ContributorAccessLevel {
  fn from(i: i32) -> Self {
    match i {
      0 => ContributorAccessLevel::PARTIAL,
      1 => ContributorAccessLevel::FULL,
      _ => ContributorAccessLevel(i)
    }
  }
}

impl From<&i32> for ContributorAccessLevel {
  fn from(i: &i32) -> Self {
    ContributorAccessLevel::from(*i)
  }
}

impl From<ContributorAccessLevel> for i32 {
  fn from(e: ContributorAccessLevel) -> i32 {
    e.0
  }
}

impl From<&ContributorAccessLevel> for i32 {
  fn from(e: &ContributorAccessLevel) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FacebookConnectionType(pub i32);

impl FacebookConnectionType {
  pub const PROFILE: FacebookConnectionType = FacebookConnectionType(0);
  pub const PAGE: FacebookConnectionType = FacebookConnectionType(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PROFILE,
    Self::PAGE,
  ];
}

impl TSerializable for FacebookConnectionType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FacebookConnectionType> {
    let enum_value = i_prot.read_i32()?;
    Ok(FacebookConnectionType::from(enum_value))
  }
}

impl From<i32> for FacebookConnectionType {
  fn from(i: i32) -> Self {
    match i {
      0 => FacebookConnectionType::PROFILE,
      1 => FacebookConnectionType::PAGE,
      _ => FacebookConnectionType(i)
    }
  }
}

impl From<&i32> for FacebookConnectionType {
  fn from(i: &i32) -> Self {
    FacebookConnectionType::from(*i)
  }
}

impl From<FacebookConnectionType> for i32 {
  fn from(e: FacebookConnectionType) -> i32 {
    e.0
  }
}

impl From<&FacebookConnectionType> for i32 {
  fn from(e: &FacebookConnectionType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PeriscopeUserType(pub i32);

impl PeriscopeUserType {
  pub const NORMAL: PeriscopeUserType = PeriscopeUserType(0);
  pub const SHELL: PeriscopeUserType = PeriscopeUserType(1);
  pub const THIRD_PARTY_AUTH: PeriscopeUserType = PeriscopeUserType(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NORMAL,
    Self::SHELL,
    Self::THIRD_PARTY_AUTH,
  ];
}

impl TSerializable for PeriscopeUserType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PeriscopeUserType> {
    let enum_value = i_prot.read_i32()?;
    Ok(PeriscopeUserType::from(enum_value))
  }
}

impl From<i32> for PeriscopeUserType {
  fn from(i: i32) -> Self {
    match i {
      0 => PeriscopeUserType::NORMAL,
      1 => PeriscopeUserType::SHELL,
      2 => PeriscopeUserType::THIRD_PARTY_AUTH,
      _ => PeriscopeUserType(i)
    }
  }
}

impl From<&i32> for PeriscopeUserType {
  fn from(i: &i32) -> Self {
    PeriscopeUserType::from(*i)
  }
}

impl From<PeriscopeUserType> for i32 {
  fn from(e: PeriscopeUserType) -> i32 {
    e.0
  }
}

impl From<&PeriscopeUserType> for i32 {
  fn from(e: &PeriscopeUserType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MuteSurface(pub i32);

impl MuteSurface {
  pub const NOTIFICATIONS: MuteSurface = MuteSurface(0);
  pub const HOME_TIMELINE: MuteSurface = MuteSurface(1);
  pub const TWEET_REPLIES: MuteSurface = MuteSurface(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NOTIFICATIONS,
    Self::HOME_TIMELINE,
    Self::TWEET_REPLIES,
  ];
}

impl TSerializable for MuteSurface {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MuteSurface> {
    let enum_value = i_prot.read_i32()?;
    Ok(MuteSurface::from(enum_value))
  }
}

impl From<i32> for MuteSurface {
  fn from(i: i32) -> Self {
    match i {
      0 => MuteSurface::NOTIFICATIONS,
      1 => MuteSurface::HOME_TIMELINE,
      2 => MuteSurface::TWEET_REPLIES,
      _ => MuteSurface(i)
    }
  }
}

impl From<&i32> for MuteSurface {
  fn from(i: &i32) -> Self {
    MuteSurface::from(*i)
  }
}

impl From<MuteSurface> for i32 {
  fn from(e: MuteSurface) -> i32 {
    e.0
  }
}

impl From<&MuteSurface> for i32 {
  fn from(e: &MuteSurface) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MuteOption(pub i32);

impl MuteOption {
        pub const MUTE_BASED_ON_PROFILE: MuteOption = MuteOption(0);
      pub const EXCLUDE_FOLLOWING_ACCOUNTS: MuteOption = MuteOption(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::MUTE_BASED_ON_PROFILE,
    Self::EXCLUDE_FOLLOWING_ACCOUNTS,
  ];
}

impl TSerializable for MuteOption {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MuteOption> {
    let enum_value = i_prot.read_i32()?;
    Ok(MuteOption::from(enum_value))
  }
}

impl From<i32> for MuteOption {
  fn from(i: i32) -> Self {
    match i {
      0 => MuteOption::MUTE_BASED_ON_PROFILE,
      1 => MuteOption::EXCLUDE_FOLLOWING_ACCOUNTS,
      _ => MuteOption(i)
    }
  }
}

impl From<&i32> for MuteOption {
  fn from(i: &i32) -> Self {
    MuteOption::from(*i)
  }
}

impl From<MuteOption> for i32 {
  fn from(e: MuteOption) -> i32 {
    e.0
  }
}

impl From<&MuteOption> for i32 {
  fn from(e: &MuteOption) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MessagingDeviceType(pub i32);

impl MessagingDeviceType {
  pub const MESSAGING: MessagingDeviceType = MessagingDeviceType(0);
  pub const EMAIL: MessagingDeviceType = MessagingDeviceType(1);
  pub const KEITAI_MAIL: MessagingDeviceType = MessagingDeviceType(2);
  pub const PUSH: MessagingDeviceType = MessagingDeviceType(3);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::MESSAGING,
    Self::EMAIL,
    Self::KEITAI_MAIL,
    Self::PUSH,
  ];
}

impl TSerializable for MessagingDeviceType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MessagingDeviceType> {
    let enum_value = i_prot.read_i32()?;
    Ok(MessagingDeviceType::from(enum_value))
  }
}

impl From<i32> for MessagingDeviceType {
  fn from(i: i32) -> Self {
    match i {
      0 => MessagingDeviceType::MESSAGING,
      1 => MessagingDeviceType::EMAIL,
      2 => MessagingDeviceType::KEITAI_MAIL,
      3 => MessagingDeviceType::PUSH,
      _ => MessagingDeviceType(i)
    }
  }
}

impl From<&i32> for MessagingDeviceType {
  fn from(i: &i32) -> Self {
    MessagingDeviceType::from(*i)
  }
}

impl From<MessagingDeviceType> for i32 {
  fn from(e: MessagingDeviceType) -> i32 {
    e.0
  }
}

impl From<&MessagingDeviceType> for i32 {
  fn from(e: &MessagingDeviceType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AppPushDeviceType(pub i32);

impl AppPushDeviceType {
  pub const PUSH: AppPushDeviceType = AppPushDeviceType(0);
  pub const VOIP_PUSH: AppPushDeviceType = AppPushDeviceType(1);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PUSH,
    Self::VOIP_PUSH,
  ];
}

impl TSerializable for AppPushDeviceType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AppPushDeviceType> {
    let enum_value = i_prot.read_i32()?;
    Ok(AppPushDeviceType::from(enum_value))
  }
}

impl From<i32> for AppPushDeviceType {
  fn from(i: i32) -> Self {
    match i {
      0 => AppPushDeviceType::PUSH,
      1 => AppPushDeviceType::VOIP_PUSH,
      _ => AppPushDeviceType(i)
    }
  }
}

impl From<&i32> for AppPushDeviceType {
  fn from(i: &i32) -> Self {
    AppPushDeviceType::from(*i)
  }
}

impl From<AppPushDeviceType> for i32 {
  fn from(e: AppPushDeviceType) -> i32 {
    e.0
  }
}

impl From<&AppPushDeviceType> for i32 {
  fn from(e: &AppPushDeviceType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DerivedUserState(pub i32);

impl DerivedUserState {
  pub const CORE: DerivedUserState = DerivedUserState(0);
  pub const CASUAL: DerivedUserState = DerivedUserState(1);
  pub const NEW: DerivedUserState = DerivedUserState(2);
  pub const DORMANT: DerivedUserState = DerivedUserState(3);
  pub const SPAMMER: DerivedUserState = DerivedUserState(4);
  pub const DELETED: DerivedUserState = DerivedUserState(5);
  pub const UNKNOWN: DerivedUserState = DerivedUserState(6);
  pub const RESERVED_7: DerivedUserState = DerivedUserState(7);
  pub const RESERVED_8: DerivedUserState = DerivedUserState(8);
  pub const RESERVED_9: DerivedUserState = DerivedUserState(9);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::CORE,
    Self::CASUAL,
    Self::NEW,
    Self::DORMANT,
    Self::SPAMMER,
    Self::DELETED,
    Self::UNKNOWN,
    Self::RESERVED_7,
    Self::RESERVED_8,
    Self::RESERVED_9,
  ];
}

impl TSerializable for DerivedUserState {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DerivedUserState> {
    let enum_value = i_prot.read_i32()?;
    Ok(DerivedUserState::from(enum_value))
  }
}

impl From<i32> for DerivedUserState {
  fn from(i: i32) -> Self {
    match i {
      0 => DerivedUserState::CORE,
      1 => DerivedUserState::CASUAL,
      2 => DerivedUserState::NEW,
      3 => DerivedUserState::DORMANT,
      4 => DerivedUserState::SPAMMER,
      5 => DerivedUserState::DELETED,
      6 => DerivedUserState::UNKNOWN,
      7 => DerivedUserState::RESERVED_7,
      8 => DerivedUserState::RESERVED_8,
      9 => DerivedUserState::RESERVED_9,
      _ => DerivedUserState(i)
    }
  }
}

impl From<&i32> for DerivedUserState {
  fn from(i: &i32) -> Self {
    DerivedUserState::from(*i)
  }
}

impl From<DerivedUserState> for i32 {
  fn from(e: DerivedUserState) -> i32 {
    e.0
  }
}

impl From<&DerivedUserState> for i32 {
  fn from(e: &DerivedUserState) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserCohort(pub i32);

impl UserCohort {
  pub const PRODUCER_AND_CONSUMER: UserCohort = UserCohort(0);
  pub const CONSUMER: UserCohort = UserCohort(1);
  pub const IDLER: UserCohort = UserCohort(2);
  pub const DISCONNECTED: UserCohort = UserCohort(3);
  pub const SMALL_COMMUNITY_MEMBER: UserCohort = UserCohort(4);
  pub const RESERVED_5: UserCohort = UserCohort(5);
  pub const RESERVED_6: UserCohort = UserCohort(6);
  pub const RESERVED_7: UserCohort = UserCohort(7);
  pub const RESERVED_8: UserCohort = UserCohort(8);
  pub const RESERVED_9: UserCohort = UserCohort(9);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PRODUCER_AND_CONSUMER,
    Self::CONSUMER,
    Self::IDLER,
    Self::DISCONNECTED,
    Self::SMALL_COMMUNITY_MEMBER,
    Self::RESERVED_5,
    Self::RESERVED_6,
    Self::RESERVED_7,
    Self::RESERVED_8,
    Self::RESERVED_9,
  ];
}

impl TSerializable for UserCohort {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UserCohort> {
    let enum_value = i_prot.read_i32()?;
    Ok(UserCohort::from(enum_value))
  }
}

impl From<i32> for UserCohort {
  fn from(i: i32) -> Self {
    match i {
      0 => UserCohort::PRODUCER_AND_CONSUMER,
      1 => UserCohort::CONSUMER,
      2 => UserCohort::IDLER,
      3 => UserCohort::DISCONNECTED,
      4 => UserCohort::SMALL_COMMUNITY_MEMBER,
      5 => UserCohort::RESERVED_5,
      6 => UserCohort::RESERVED_6,
      7 => UserCohort::RESERVED_7,
      8 => UserCohort::RESERVED_8,
      9 => UserCohort::RESERVED_9,
      _ => UserCohort(i)
    }
  }
}

impl From<&i32> for UserCohort {
  fn from(i: &i32) -> Self {
    UserCohort::from(*i)
  }
}

impl From<UserCohort> for i32 {
  fn from(e: UserCohort) -> i32 {
    e.0
  }
}

impl From<&UserCohort> for i32 {
  fn from(e: &UserCohort) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdvertiserType(pub i32);

impl AdvertiserType {
  pub const PROMOTABLE_USER: AdvertiserType = AdvertiserType(0);
  pub const ACCOUNT_USER: AdvertiserType = AdvertiserType(1);
  pub const RESERVED_2: AdvertiserType = AdvertiserType(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::PROMOTABLE_USER,
    Self::ACCOUNT_USER,
    Self::RESERVED_2,
  ];
}

impl TSerializable for AdvertiserType {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AdvertiserType> {
    let enum_value = i_prot.read_i32()?;
    Ok(AdvertiserType::from(enum_value))
  }
}

impl From<i32> for AdvertiserType {
  fn from(i: i32) -> Self {
    match i {
      0 => AdvertiserType::PROMOTABLE_USER,
      1 => AdvertiserType::ACCOUNT_USER,
      2 => AdvertiserType::RESERVED_2,
      _ => AdvertiserType(i)
    }
  }
}

impl From<&i32> for AdvertiserType {
  fn from(i: &i32) -> Self {
    AdvertiserType::from(*i)
  }
}

impl From<AdvertiserType> for i32 {
  fn from(e: AdvertiserType) -> i32 {
    e.0
  }
}

impl From<&AdvertiserType> for i32 {
  fn from(e: &AdvertiserType) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProfileVisibility(pub i32);

impl ProfileVisibility {
  pub const SELF: ProfileVisibility = ProfileVisibility(0);
  pub const MUTUAL_FOLLOW: ProfileVisibility = ProfileVisibility(1);
  pub const FOLLOWING: ProfileVisibility = ProfileVisibility(2);
  pub const FOLLOWERS: ProfileVisibility = ProfileVisibility(3);
  pub const PUBLIC: ProfileVisibility = ProfileVisibility(4);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::SELF,
    Self::MUTUAL_FOLLOW,
    Self::FOLLOWING,
    Self::FOLLOWERS,
    Self::PUBLIC,
  ];
}

impl TSerializable for ProfileVisibility {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProfileVisibility> {
    let enum_value = i_prot.read_i32()?;
    Ok(ProfileVisibility::from(enum_value))
  }
}

impl From<i32> for ProfileVisibility {
  fn from(i: i32) -> Self {
    match i {
      0 => ProfileVisibility::SELF,
      1 => ProfileVisibility::MUTUAL_FOLLOW,
      2 => ProfileVisibility::FOLLOWING,
      3 => ProfileVisibility::FOLLOWERS,
      4 => ProfileVisibility::PUBLIC,
      _ => ProfileVisibility(i)
    }
  }
}

impl From<&i32> for ProfileVisibility {
  fn from(i: &i32) -> Self {
    ProfileVisibility::from(*i)
  }
}

impl From<ProfileVisibility> for i32 {
  fn from(e: ProfileVisibility) -> i32 {
    e.0
  }
}

impl From<&ProfileVisibility> for i32 {
  fn from(e: &ProfileVisibility) -> i32 {
    e.0
  }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TestUserStatus(pub i32);

impl TestUserStatus {
  pub const NONE: TestUserStatus = TestUserStatus(0);
  pub const TEMPORARY: TestUserStatus = TestUserStatus(1);
  pub const PERMANENT: TestUserStatus = TestUserStatus(2);
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NONE,
    Self::TEMPORARY,
    Self::PERMANENT,
  ];
}

impl TSerializable for TestUserStatus {
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(self.0)
  }
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TestUserStatus> {
    let enum_value = i_prot.read_i32()?;
    Ok(TestUserStatus::from(enum_value))
  }
}

impl From<i32> for TestUserStatus {
  fn from(i: i32) -> Self {
    match i {
      0 => TestUserStatus::NONE,
      1 => TestUserStatus::TEMPORARY,
      2 => TestUserStatus::PERMANENT,
      _ => TestUserStatus(i)
    }
  }
}

impl From<&i32> for TestUserStatus {
  fn from(i: &i32) -> Self {
    TestUserStatus::from(*i)
  }
}

impl From<TestUserStatus> for i32 {
  fn from(e: TestUserStatus) -> i32 {
    e.0
  }
}

impl From<&TestUserStatus> for i32 {
  fn from(e: &TestUserStatus) -> i32 {
    e.0
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ColorValue {
  pub red: Option<i8>,
  pub green: Option<i8>,
  pub blue: Option<i8>,
  pub alpha: Option<i8>,
}

impl ColorValue {
  pub fn new<F1, F2, F3, F4>(red: F1, green: F2, blue: F3, alpha: F4) -> ColorValue where F1: Into<Option<i8>>, F2: Into<Option<i8>>, F3: Into<Option<i8>>, F4: Into<Option<i8>> {
    ColorValue {
      red: red.into(),
      green: green.into(),
      blue: blue.into(),
      alpha: alpha.into(),
    }
  }
}

impl TSerializable for ColorValue {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ColorValue> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i8> = Some(0);
    let mut f_2: Option<i8> = Some(0);
    let mut f_3: Option<i8> = Some(0);
    let mut f_4: Option<i8> = None;
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
        4 => {
          let val = i_prot.read_i8()?;
          f_4 = Some(val);
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
      alpha: f_4,
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
    if let Some(fld_var) = self.alpha {
      o_prot.write_field_begin(&TFieldIdentifier::new("alpha", TType::I08, 4))?;
      o_prot.write_i8(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Image {
  pub id: Option<i64>,
  pub filename: Option<String>,
  pub extensions_reply: Option<Vec<u8>>,
}

impl Image {
  pub fn new<F1, F2, F3>(id: F1, filename: F2, extensions_reply: F3) -> Image where F1: Into<Option<i64>>, F2: Into<Option<String>>, F3: Into<Option<Vec<u8>>> {
    Image {
      id: id.into(),
      filename: filename.into(),
      extensions_reply: extensions_reply.into(),
    }
  }
}

impl TSerializable for Image {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Image> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<Vec<u8>> = None;
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
          let val = i_prot.read_bytes()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Image {
      id: f_1,
      filename: f_2,
      extensions_reply: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Image");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.filename {
      o_prot.write_field_begin(&TFieldIdentifier::new("filename", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.extensions_reply {
      o_prot.write_field_begin(&TFieldIdentifier::new("extensions_reply", TType::String, 3))?;
      o_prot.write_bytes(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageUrls {
  pub http_url: Option<String>,
  pub https_url: Option<String>,
}

impl ImageUrls {
  pub fn new<F1, F2>(http_url: F1, https_url: F2) -> ImageUrls where F1: Into<Option<String>>, F2: Into<Option<String>> {
    ImageUrls {
      http_url: http_url.into(),
      https_url: https_url.into(),
    }
  }
}

impl TSerializable for ImageUrls {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ImageUrls> {
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
    let ret = ImageUrls {
      http_url: f_1,
      https_url: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ImageUrls");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.http_url {
      o_prot.write_field_begin(&TFieldIdentifier::new("http_url", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.https_url {
      o_prot.write_field_begin(&TFieldIdentifier::new("https_url", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProfileImageUrls {
  pub original: Option<ImageUrls>,
  pub normal: Option<ImageUrls>,
  pub bigger: Option<ImageUrls>,
  pub mini: Option<ImageUrls>,
  pub flash_badge: Option<ImageUrls>,
  pub x96: Option<ImageUrls>,
  pub reasonably_small: Option<ImageUrls>,
  pub x200: Option<ImageUrls>,
  pub x400: Option<ImageUrls>,
}

impl ProfileImageUrls {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9>(original: F1, normal: F2, bigger: F3, mini: F4, flash_badge: F5, x96: F6, reasonably_small: F7, x200: F8, x400: F9) -> ProfileImageUrls where F1: Into<Option<ImageUrls>>, F2: Into<Option<ImageUrls>>, F3: Into<Option<ImageUrls>>, F4: Into<Option<ImageUrls>>, F5: Into<Option<ImageUrls>>, F6: Into<Option<ImageUrls>>, F7: Into<Option<ImageUrls>>, F8: Into<Option<ImageUrls>>, F9: Into<Option<ImageUrls>> {
    ProfileImageUrls {
      original: original.into(),
      normal: normal.into(),
      bigger: bigger.into(),
      mini: mini.into(),
      flash_badge: flash_badge.into(),
      x96: x96.into(),
      reasonably_small: reasonably_small.into(),
      x200: x200.into(),
      x400: x400.into(),
    }
  }
}

impl TSerializable for ProfileImageUrls {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProfileImageUrls> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<ImageUrls> = None;
    let mut f_2: Option<ImageUrls> = None;
    let mut f_3: Option<ImageUrls> = None;
    let mut f_4: Option<ImageUrls> = None;
    let mut f_5: Option<ImageUrls> = None;
    let mut f_6: Option<ImageUrls> = None;
    let mut f_7: Option<ImageUrls> = None;
    let mut f_8: Option<ImageUrls> = None;
    let mut f_9: Option<ImageUrls> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        5 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        6 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_6 = Some(val);
        },
        7 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_7 = Some(val);
        },
        8 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_8 = Some(val);
        },
        9 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_9 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ProfileImageUrls {
      original: f_1,
      normal: f_2,
      bigger: f_3,
      mini: f_4,
      flash_badge: f_5,
      x96: f_6,
      reasonably_small: f_7,
      x200: f_8,
      x400: f_9,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ProfileImageUrls");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.original {
      o_prot.write_field_begin(&TFieldIdentifier::new("original", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.normal {
      o_prot.write_field_begin(&TFieldIdentifier::new("normal", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.bigger {
      o_prot.write_field_begin(&TFieldIdentifier::new("bigger", TType::Struct, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.mini {
      o_prot.write_field_begin(&TFieldIdentifier::new("mini", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.flash_badge {
      o_prot.write_field_begin(&TFieldIdentifier::new("flash_badge", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.x96 {
      o_prot.write_field_begin(&TFieldIdentifier::new("x96", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.reasonably_small {
      o_prot.write_field_begin(&TFieldIdentifier::new("reasonably_small", TType::Struct, 7))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.x200 {
      o_prot.write_field_begin(&TFieldIdentifier::new("x200", TType::Struct, 8))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.x400 {
      o_prot.write_field_begin(&TFieldIdentifier::new("x400", TType::Struct, 9))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Design {
  pub background_color: Option<ColorValue>,
  pub text_color: Option<ColorValue>,
  pub link_color: Option<ColorValue>,
  pub sidebar_fill_color: Option<ColorValue>,
  pub sidebar_border_color: Option<ColorValue>,
  pub background_image_urls: Option<ImageUrls>,
  pub use_background_image: Option<bool>,
  pub background_tile: Option<bool>,
  pub background_image: Option<Image>,
  pub background_position: Option<BackgroundPosition>,
}

impl Design {
  pub fn new<F1, F2, F3, F4, F5, F7, F8, F9, F10, F11>(background_color: F1, text_color: F2, link_color: F3, sidebar_fill_color: F4, sidebar_border_color: F5, background_image_urls: F7, use_background_image: F8, background_tile: F9, background_image: F10, background_position: F11) -> Design where F1: Into<Option<ColorValue>>, F2: Into<Option<ColorValue>>, F3: Into<Option<ColorValue>>, F4: Into<Option<ColorValue>>, F5: Into<Option<ColorValue>>, F7: Into<Option<ImageUrls>>, F8: Into<Option<bool>>, F9: Into<Option<bool>>, F10: Into<Option<Image>>, F11: Into<Option<BackgroundPosition>> {
    Design {
      background_color: background_color.into(),
      text_color: text_color.into(),
      link_color: link_color.into(),
      sidebar_fill_color: sidebar_fill_color.into(),
      sidebar_border_color: sidebar_border_color.into(),
      background_image_urls: background_image_urls.into(),
      use_background_image: use_background_image.into(),
      background_tile: background_tile.into(),
      background_image: background_image.into(),
      background_position: background_position.into(),
    }
  }
}

impl TSerializable for Design {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Design> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<ColorValue> = None;
    let mut f_2: Option<ColorValue> = None;
    let mut f_3: Option<ColorValue> = None;
    let mut f_4: Option<ColorValue> = None;
    let mut f_5: Option<ColorValue> = None;
    let mut f_7: Option<ImageUrls> = None;
    let mut f_8: Option<bool> = Some(false);
    let mut f_9: Option<bool> = Some(false);
    let mut f_10: Option<Image> = None;
    let mut f_11: Option<BackgroundPosition> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = ColorValue::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = ColorValue::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = ColorValue::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = ColorValue::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        5 => {
          let val = ColorValue::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        7 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_7 = Some(val);
        },
        8 => {
          let val = i_prot.read_bool()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let val = Image::read_from_in_protocol(i_prot)?;
          f_10 = Some(val);
        },
        11 => {
          let val = BackgroundPosition::read_from_in_protocol(i_prot)?;
          f_11 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Design {
      background_color: f_1,
      text_color: f_2,
      link_color: f_3,
      sidebar_fill_color: f_4,
      sidebar_border_color: f_5,
      background_image_urls: f_7,
      use_background_image: f_8,
      background_tile: f_9,
      background_image: f_10,
      background_position: f_11,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Design");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.background_color {
      o_prot.write_field_begin(&TFieldIdentifier::new("background_color", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.text_color {
      o_prot.write_field_begin(&TFieldIdentifier::new("text_color", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.link_color {
      o_prot.write_field_begin(&TFieldIdentifier::new("link_color", TType::Struct, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.sidebar_fill_color {
      o_prot.write_field_begin(&TFieldIdentifier::new("sidebar_fill_color", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.sidebar_border_color {
      o_prot.write_field_begin(&TFieldIdentifier::new("sidebar_border_color", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.background_image_urls {
      o_prot.write_field_begin(&TFieldIdentifier::new("background_image_urls", TType::Struct, 7))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.use_background_image {
      o_prot.write_field_begin(&TFieldIdentifier::new("use_background_image", TType::Bool, 8))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.background_tile {
      o_prot.write_field_begin(&TFieldIdentifier::new("background_tile", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.background_image {
      o_prot.write_field_begin(&TFieldIdentifier::new("background_image", TType::Struct, 10))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.background_position {
      o_prot.write_field_begin(&TFieldIdentifier::new("background_position", TType::I32, 11))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProfileDesign {
  pub profile_type: Option<ProfileType>,
  pub theme_name: Option<String>,
  pub design: Option<Design>,
  pub pinned_tweet_ids: Option<Vec<i64>>,
  pub featured_list_ids: Option<Vec<i64>>,
}

impl ProfileDesign {
  pub fn new<F2, F3, F4, F5, F6>(profile_type: F2, theme_name: F3, design: F4, pinned_tweet_ids: F5, featured_list_ids: F6) -> ProfileDesign where F2: Into<Option<ProfileType>>, F3: Into<Option<String>>, F4: Into<Option<Design>>, F5: Into<Option<Vec<i64>>>, F6: Into<Option<Vec<i64>>> {
    ProfileDesign {
      profile_type: profile_type.into(),
      theme_name: theme_name.into(),
      design: design.into(),
      pinned_tweet_ids: pinned_tweet_ids.into(),
      featured_list_ids: featured_list_ids.into(),
    }
  }
}

impl TSerializable for ProfileDesign {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProfileDesign> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<ProfileType> = None;
    let mut f_3: Option<String> = None;
    let mut f_4: Option<Design> = None;
    let mut f_5: Option<Vec<i64>> = None;
    let mut f_6: Option<Vec<i64>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = ProfileType::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let val = Design::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        5 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_0 = i_prot.read_i64()?;
            val.push(list_elem_0);
          }
          i_prot.read_list_end()?;
          f_5 = Some(val);
        },
        6 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_1 = i_prot.read_i64()?;
            val.push(list_elem_1);
          }
          i_prot.read_list_end()?;
          f_6 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ProfileDesign {
      profile_type: f_2,
      theme_name: f_3,
      design: f_4,
      pinned_tweet_ids: f_5,
      featured_list_ids: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ProfileDesign");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.profile_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("profile_type", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.theme_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("theme_name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.design {
      o_prot.write_field_begin(&TFieldIdentifier::new("design", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.pinned_tweet_ids {
      o_prot.write_field_begin(&TFieldIdentifier::new("pinned_tweet_ids", TType::List, 5))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::I64, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_i64(*e)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.featured_list_ids {
      o_prot.write_field_begin(&TFieldIdentifier::new("featured_list_ids", TType::List, 6))?;
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
pub struct LoginVerify {
  pub public_key: Option<String>,
  pub last_seen_offline_code: Option<String>,
  pub iteration_count: Option<i32>,
}

impl LoginVerify {
  pub fn new<F1, F2, F3>(public_key: F1, last_seen_offline_code: F2, iteration_count: F3) -> LoginVerify where F1: Into<Option<String>>, F2: Into<Option<String>>, F3: Into<Option<i32>> {
    LoginVerify {
      public_key: public_key.into(),
      last_seen_offline_code: last_seen_offline_code.into(),
      iteration_count: iteration_count.into(),
    }
  }
}

impl TSerializable for LoginVerify {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<LoginVerify> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<i32> = Some(0);
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
    let ret = LoginVerify {
      public_key: f_1,
      last_seen_offline_code: f_2,
      iteration_count: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("LoginVerify");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.public_key {
      o_prot.write_field_begin(&TFieldIdentifier::new("public_key", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.last_seen_offline_code {
      o_prot.write_field_begin(&TFieldIdentifier::new("last_seen_offline_code", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.iteration_count {
      o_prot.write_field_begin(&TFieldIdentifier::new("iteration_count", TType::I32, 3))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserPasswordStatus {
  pub has_password_been_set_by_user: Option<bool>,
  pub last_password_change_by_user_timestamp: Option<i64>,
}

impl UserPasswordStatus {
  pub fn new<F1, F2>(has_password_been_set_by_user: F1, last_password_change_by_user_timestamp: F2) -> UserPasswordStatus where F1: Into<Option<bool>>, F2: Into<Option<i64>> {
    UserPasswordStatus {
      has_password_been_set_by_user: has_password_been_set_by_user.into(),
      last_password_change_by_user_timestamp: last_password_change_by_user_timestamp.into(),
    }
  }
}

impl TSerializable for UserPasswordStatus {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UserPasswordStatus> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = Some(false);
    let mut f_2: Option<i64> = None;
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
    let ret = UserPasswordStatus {
      has_password_been_set_by_user: f_1,
      last_password_change_by_user_timestamp: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UserPasswordStatus");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.has_password_been_set_by_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_password_been_set_by_user", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.last_password_change_by_user_timestamp {
      o_prot.write_field_begin(&TFieldIdentifier::new("last_password_change_by_user_timestamp", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Auth {
  pub login_email: Option<String>,
  pub auth_token: Option<String>,
  pub hashed_password: Option<String>,
  pub login_verification: Option<LoginVerify>,
  pub user_password_status: Option<UserPasswordStatus>,
}

impl Auth {
  pub fn new<F1, F2, F3, F4, F5>(login_email: F1, auth_token: F2, hashed_password: F3, login_verification: F4, user_password_status: F5) -> Auth where F1: Into<Option<String>>, F2: Into<Option<String>>, F3: Into<Option<String>>, F4: Into<Option<LoginVerify>>, F5: Into<Option<UserPasswordStatus>> {
    Auth {
      login_email: login_email.into(),
      auth_token: auth_token.into(),
      hashed_password: hashed_password.into(),
      login_verification: login_verification.into(),
      user_password_status: user_password_status.into(),
    }
  }
}

impl TSerializable for Auth {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Auth> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<LoginVerify> = None;
    let mut f_5: Option<UserPasswordStatus> = None;
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
        4 => {
          let val = LoginVerify::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        5 => {
          let val = UserPasswordStatus::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Auth {
      login_email: f_1,
      auth_token: f_2,
      hashed_password: f_3,
      login_verification: f_4,
      user_password_status: f_5,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Auth");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.login_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("login_email", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.auth_token {
      o_prot.write_field_begin(&TFieldIdentifier::new("auth_token", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.hashed_password {
      o_prot.write_field_begin(&TFieldIdentifier::new("hashed_password", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.login_verification {
      o_prot.write_field_begin(&TFieldIdentifier::new("login_verification", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.user_password_status {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_password_status", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProfileBanner {
  pub activated: Option<bool>,
  pub updated_at_msec: Option<i64>,
  pub urls: Option<ImageUrls>,
  pub extensions_reply: Option<Vec<u8>>,
}

impl ProfileBanner {
  pub fn new<F2, F3, F4, F5>(activated: F2, updated_at_msec: F3, urls: F4, extensions_reply: F5) -> ProfileBanner where F2: Into<Option<bool>>, F3: Into<Option<i64>>, F4: Into<Option<ImageUrls>>, F5: Into<Option<Vec<u8>>> {
    ProfileBanner {
      activated: activated.into(),
      updated_at_msec: updated_at_msec.into(),
      urls: urls.into(),
      extensions_reply: extensions_reply.into(),
    }
  }
}

impl TSerializable for ProfileBanner {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProfileBanner> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<bool> = Some(false);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<ImageUrls> = None;
    let mut f_5: Option<Vec<u8>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = i_prot.read_bool()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i64()?;
          f_3 = Some(val);
        },
        4 => {
          let val = ImageUrls::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_bytes()?;
          f_5 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ProfileBanner {
      activated: f_2,
      updated_at_msec: f_3,
      urls: f_4,
      extensions_reply: f_5,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ProfileBanner");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.activated {
      o_prot.write_field_begin(&TFieldIdentifier::new("activated", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.updated_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("updated_at_msec", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.urls {
      o_prot.write_field_begin(&TFieldIdentifier::new("urls", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.extensions_reply {
      o_prot.write_field_begin(&TFieldIdentifier::new("extensions_reply", TType::String, 5))?;
      o_prot.write_bytes(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BusinessProfile {
  pub business_profile_state: Option<BusinessProfileState>,
  pub customer_service_state: Option<CustomerServiceState>,
}

impl BusinessProfile {
  pub fn new<F1, F2>(business_profile_state: F1, customer_service_state: F2) -> BusinessProfile where F1: Into<Option<BusinessProfileState>>, F2: Into<Option<CustomerServiceState>> {
    BusinessProfile {
      business_profile_state: business_profile_state.into(),
      customer_service_state: customer_service_state.into(),
    }
  }
}

impl TSerializable for BusinessProfile {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<BusinessProfile> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<BusinessProfileState> = None;
    let mut f_2: Option<CustomerServiceState> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = BusinessProfileState::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = CustomerServiceState::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = BusinessProfile {
      business_profile_state: f_1,
      customer_service_state: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("BusinessProfile");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.business_profile_state {
      o_prot.write_field_begin(&TFieldIdentifier::new("business_profile_state", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.customer_service_state {
      o_prot.write_field_begin(&TFieldIdentifier::new("customer_service_state", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Language {
    pub language: String,
    pub confidence: Option<OrderedFloat<f64>>,
}

impl Language {
  pub fn new<F2>(language: String, confidence: F2) -> Language where F2: Into<Option<OrderedFloat<f64>>> {
    Language {
      language,
      confidence: confidence.into(),
    }
  }
}

impl TSerializable for Language {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Language> {
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
        },
        2 => {
          let val = OrderedFloat::from(i_prot.read_double()?);
          f_2 = Some(val);
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
      confidence: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Language");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("language", TType::String, 1))?;
    o_prot.write_string(&self.language)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.confidence {
      o_prot.write_field_begin(&TFieldIdentifier::new("confidence", TType::Double, 2))?;
      o_prot.write_double(fld_var.into())?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Profile {
  pub name: Option<String>,
  pub screen_name: Option<String>,
  pub profile_image_urls: Option<ProfileImageUrls>,
  pub auth: Option<Auth>,
  pub location: Option<String>,
  pub description: Option<String>,
  pub url: Option<String>,
  pub profile_image: Option<Image>,
  pub profile_banner: Option<ProfileBanner>,
  pub location_place_id: Option<String>,
  pub has_translator_badge: Option<bool>,
  pub business_profile_state: Option<BusinessProfileState>,
  pub vine_profile_visible: Option<bool>,
  pub translator_type: Option<TranslatorType>,
  pub business_profile: Option<BusinessProfile>,
  pub periscope_profile_visible: Option<bool>,
  pub description_language: Option<Language>,
}

impl Profile {
  pub fn new<F2, F3, F5, F6, F7, F8, F9, F10, F11, F12, F13, F15, F16, F17, F18, F19, F20>(name: F2, screen_name: F3, profile_image_urls: F5, auth: F6, location: F7, description: F8, url: F9, profile_image: F10, profile_banner: F11, location_place_id: F12, has_translator_badge: F13, business_profile_state: F15, vine_profile_visible: F16, translator_type: F17, business_profile: F18, periscope_profile_visible: F19, description_language: F20) -> Profile where F2: Into<Option<String>>, F3: Into<Option<String>>, F5: Into<Option<ProfileImageUrls>>, F6: Into<Option<Auth>>, F7: Into<Option<String>>, F8: Into<Option<String>>, F9: Into<Option<String>>, F10: Into<Option<Image>>, F11: Into<Option<ProfileBanner>>, F12: Into<Option<String>>, F13: Into<Option<bool>>, F15: Into<Option<BusinessProfileState>>, F16: Into<Option<bool>>, F17: Into<Option<TranslatorType>>, F18: Into<Option<BusinessProfile>>, F19: Into<Option<bool>>, F20: Into<Option<Language>> {
    Profile {
      name: name.into(),
      screen_name: screen_name.into(),
      profile_image_urls: profile_image_urls.into(),
      auth: auth.into(),
      location: location.into(),
      description: description.into(),
      url: url.into(),
      profile_image: profile_image.into(),
      profile_banner: profile_banner.into(),
      location_place_id: location_place_id.into(),
      has_translator_badge: has_translator_badge.into(),
      business_profile_state: business_profile_state.into(),
      vine_profile_visible: vine_profile_visible.into(),
      translator_type: translator_type.into(),
      business_profile: business_profile.into(),
      periscope_profile_visible: periscope_profile_visible.into(),
      description_language: description_language.into(),
    }
  }
}

impl TSerializable for Profile {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Profile> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_5: Option<ProfileImageUrls> = None;
    let mut f_6: Option<Auth> = None;
    let mut f_7: Option<String> = Some("".to_owned());
    let mut f_8: Option<String> = Some("".to_owned());
    let mut f_9: Option<String> = Some("".to_owned());
    let mut f_10: Option<Image> = None;
    let mut f_11: Option<ProfileBanner> = None;
    let mut f_12: Option<String> = None;
    let mut f_13: Option<bool> = None;
    let mut f_15: Option<BusinessProfileState> = None;
    let mut f_16: Option<bool> = None;
    let mut f_17: Option<TranslatorType> = None;
    let mut f_18: Option<BusinessProfile> = None;
    let mut f_19: Option<bool> = None;
    let mut f_20: Option<Language> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = i_prot.read_string()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        5 => {
          let val = ProfileImageUrls::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        6 => {
          let val = Auth::read_from_in_protocol(i_prot)?;
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
          let val = Image::read_from_in_protocol(i_prot)?;
          f_10 = Some(val);
        },
        11 => {
          let val = ProfileBanner::read_from_in_protocol(i_prot)?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_string()?;
          f_12 = Some(val);
        },
        13 => {
          let val = i_prot.read_bool()?;
          f_13 = Some(val);
        },
        15 => {
          let val = BusinessProfileState::read_from_in_protocol(i_prot)?;
          f_15 = Some(val);
        },
        16 => {
          let val = i_prot.read_bool()?;
          f_16 = Some(val);
        },
        17 => {
          let val = TranslatorType::read_from_in_protocol(i_prot)?;
          f_17 = Some(val);
        },
        18 => {
          let val = BusinessProfile::read_from_in_protocol(i_prot)?;
          f_18 = Some(val);
        },
        19 => {
          let val = i_prot.read_bool()?;
          f_19 = Some(val);
        },
        20 => {
          let val = Language::read_from_in_protocol(i_prot)?;
          f_20 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Profile {
      name: f_2,
      screen_name: f_3,
      profile_image_urls: f_5,
      auth: f_6,
      location: f_7,
      description: f_8,
      url: f_9,
      profile_image: f_10,
      profile_banner: f_11,
      location_place_id: f_12,
      has_translator_badge: f_13,
      business_profile_state: f_15,
      vine_profile_visible: f_16,
      translator_type: f_17,
      business_profile: f_18,
      periscope_profile_visible: f_19,
      description_language: f_20,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Profile");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.screen_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("screen_name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.profile_image_urls {
      o_prot.write_field_begin(&TFieldIdentifier::new("profile_image_urls", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.auth {
      o_prot.write_field_begin(&TFieldIdentifier::new("auth", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.location {
      o_prot.write_field_begin(&TFieldIdentifier::new("location", TType::String, 7))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.description {
      o_prot.write_field_begin(&TFieldIdentifier::new("description", TType::String, 8))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.url {
      o_prot.write_field_begin(&TFieldIdentifier::new("url", TType::String, 9))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.profile_image {
      o_prot.write_field_begin(&TFieldIdentifier::new("profile_image", TType::Struct, 10))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.profile_banner {
      o_prot.write_field_begin(&TFieldIdentifier::new("profile_banner", TType::Struct, 11))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.location_place_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("location_place_id", TType::String, 12))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_translator_badge {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_translator_badge", TType::Bool, 13))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.business_profile_state {
      o_prot.write_field_begin(&TFieldIdentifier::new("business_profile_state", TType::I32, 15))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.vine_profile_visible {
      o_prot.write_field_begin(&TFieldIdentifier::new("vine_profile_visible", TType::Bool, 16))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.translator_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("translator_type", TType::I32, 17))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.business_profile {
      o_prot.write_field_begin(&TFieldIdentifier::new("business_profile", TType::Struct, 18))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.periscope_profile_visible {
      o_prot.write_field_begin(&TFieldIdentifier::new("periscope_profile_visible", TType::Bool, 19))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.description_language {
      o_prot.write_field_begin(&TFieldIdentifier::new("description_language", TType::Struct, 20))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TimeZone {
  pub name: Option<String>,
  pub tz_name: Option<String>,
  pub utc_offset_seconds: Option<i32>,
}

impl TimeZone {
  pub fn new<F2, F3, F4>(name: F2, tz_name: F3, utc_offset_seconds: F4) -> TimeZone where F2: Into<Option<String>>, F3: Into<Option<String>>, F4: Into<Option<i32>> {
    TimeZone {
      name: name.into(),
      tz_name: tz_name.into(),
      utc_offset_seconds: utc_offset_seconds.into(),
    }
  }
}

impl TSerializable for TimeZone {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TimeZone> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<String> = None;
    let mut f_4: Option<i32> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = i_prot.read_string()?;
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
    let ret = TimeZone {
      name: f_2,
      tz_name: f_3,
      utc_offset_seconds: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TimeZone");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.tz_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("tz_name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.utc_offset_seconds {
      o_prot.write_field_begin(&TFieldIdentifier::new("utc_offset_seconds", TType::I32, 4))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Discoverability {
  pub discoverable_by_email: Option<bool>,
  pub discoverable_by_mobile_phone: Option<bool>,
}

impl Discoverability {
  pub fn new<F2, F4>(discoverable_by_email: F2, discoverable_by_mobile_phone: F4) -> Discoverability where F2: Into<Option<bool>>, F4: Into<Option<bool>> {
    Discoverability {
      discoverable_by_email: discoverable_by_email.into(),
      discoverable_by_mobile_phone: discoverable_by_mobile_phone.into(),
    }
  }
}

impl TSerializable for Discoverability {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Discoverability> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<bool> = Some(false);
    let mut f_4: Option<bool> = Some(false);
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = i_prot.read_bool()?;
          f_2 = Some(val);
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
    let ret = Discoverability {
      discoverable_by_email: f_2,
      discoverable_by_mobile_phone: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Discoverability");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.discoverable_by_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("discoverable_by_email", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.discoverable_by_mobile_phone {
      o_prot.write_field_begin(&TFieldIdentifier::new("discoverable_by_mobile_phone", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SmartBlock {
  pub state: SmartBlockState,
  pub at_mentions_enabled: Option<bool>,
  pub social_proof_enabled: Option<bool>,
}

impl SmartBlock {
  pub fn new<F2, F3>(state: SmartBlockState, at_mentions_enabled: F2, social_proof_enabled: F3) -> SmartBlock where F2: Into<Option<bool>>, F3: Into<Option<bool>> {
    SmartBlock {
      state,
      at_mentions_enabled: at_mentions_enabled.into(),
      social_proof_enabled: social_proof_enabled.into(),
    }
  }
}

impl TSerializable for SmartBlock {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SmartBlock> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<SmartBlockState> = None;
    let mut f_2: Option<bool> = None;
    let mut f_3: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = SmartBlockState::read_from_in_protocol(i_prot)?;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("SmartBlock.state", &f_1)?;
    let ret = SmartBlock {
      state: f_1.expect("auto-generated code should have checked for presence of required fields"),
      at_mentions_enabled: f_2,
      social_proof_enabled: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SmartBlock");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("state", TType::I32, 1))?;
    self.state.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.at_mentions_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("at_mentions_enabled", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.social_proof_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("social_proof_enabled", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AnnotationKey {
  pub key_namespace: String,
  pub name: Option<String>,
}

impl AnnotationKey {
  pub fn new<F2>(key_namespace: String, name: F2) -> AnnotationKey where F2: Into<Option<String>> {
    AnnotationKey {
      key_namespace,
      name: name.into(),
    }
  }
}

impl TSerializable for AnnotationKey {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AnnotationKey> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = None;
    let mut f_2: Option<String> = None;
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
    verify_required_field_exists("AnnotationKey.key_namespace", &f_1)?;
    let ret = AnnotationKey {
      key_namespace: f_1.expect("auto-generated code should have checked for presence of required fields"),
      name: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AnnotationKey");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("key_namespace", TType::String, 1))?;
    o_prot.write_string(&self.key_namespace)?;
    o_prot.write_field_end()?;
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AnnotationValue {
  ShortValue(i16),
  IntValue(i32),
  LongValue(i64),
  DoubleValue(OrderedFloat<f64>),
  BooleanValue(bool),
  StrValue(String),
  ShortSetValue(BTreeSet<i16>),
  IntSetValue(BTreeSet<i32>),
  LongSetValue(BTreeSet<i64>),
  StrSetValue(BTreeSet<String>),
  ShortListValue(Vec<i16>),
  IntListValue(Vec<i32>),
  LongListValue(Vec<i64>),
  DoubleListValue(Vec<OrderedFloat<f64>>),
  BooleanListValue(Vec<bool>),
  StrListValue(Vec<String>),
  ShortMapValue(BTreeMap<String, i16>),
  IntMapValue(BTreeMap<String, i32>),
  LongMapValue(BTreeMap<String, i64>),
  DoubleMapValue(BTreeMap<String, OrderedFloat<f64>>),
  BooleanMapValue(BTreeMap<String, bool>),
  StrMapValue(BTreeMap<String, String>),
  IntToIntMapValue(BTreeMap<i32, i32>),
  LongToIntMapValue(BTreeMap<i64, i32>),
  ShortToDoubleMapValue(BTreeMap<i16, OrderedFloat<f64>>),
  LongToDoubleMapValue(BTreeMap<i64, OrderedFloat<f64>>),
  LongToStringMapValue(BTreeMap<i64, String>),
  IntToDoubleMapValue(BTreeMap<i32, OrderedFloat<f64>>),
  BinaryValue(Vec<u8>),
  StrListMapValue(BTreeMap<String, Vec<String>>),
  LongListMapValue(BTreeMap<String, Vec<i64>>),
  IntListMapValue(BTreeMap<String, Vec<i32>>),
  BinaryListValue(Vec<Vec<u8>>),
}

impl TSerializable for AnnotationValue {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AnnotationValue> {
    let mut ret: Option<AnnotationValue> = None;
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
          let val = i_prot.read_i16()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::ShortValue(val));
          }
          received_field_count += 1;
        },
        2 => {
          let val = i_prot.read_i32()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::IntValue(val));
          }
          received_field_count += 1;
        },
        3 => {
          let val = i_prot.read_i64()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::LongValue(val));
          }
          received_field_count += 1;
        },
        4 => {
          let val = OrderedFloat::from(i_prot.read_double()?);
          if ret.is_none() {
            ret = Some(AnnotationValue::DoubleValue(val));
          }
          received_field_count += 1;
        },
        5 => {
          let val = i_prot.read_bool()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::BooleanValue(val));
          }
          received_field_count += 1;
        },
        6 => {
          let val = i_prot.read_string()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::StrValue(val));
          }
          received_field_count += 1;
        },
        7 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<i16> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_2 = i_prot.read_i16()?;
            val.insert(set_elem_2);
          }
          i_prot.read_set_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::ShortSetValue(val));
          }
          received_field_count += 1;
        },
        8 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<i32> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_3 = i_prot.read_i32()?;
            val.insert(set_elem_3);
          }
          i_prot.read_set_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::IntSetValue(val));
          }
          received_field_count += 1;
        },
        9 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<i64> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_4 = i_prot.read_i64()?;
            val.insert(set_elem_4);
          }
          i_prot.read_set_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::LongSetValue(val));
          }
          received_field_count += 1;
        },
        10 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<String> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_5 = i_prot.read_string()?;
            val.insert(set_elem_5);
          }
          i_prot.read_set_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::StrSetValue(val));
          }
          received_field_count += 1;
        },
        11 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i16> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_6 = i_prot.read_i16()?;
            val.push(list_elem_6);
          }
          i_prot.read_list_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::ShortListValue(val));
          }
          received_field_count += 1;
        },
        12 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i32> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_7 = i_prot.read_i32()?;
            val.push(list_elem_7);
          }
          i_prot.read_list_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::IntListValue(val));
          }
          received_field_count += 1;
        },
        13 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_8 = i_prot.read_i64()?;
            val.push(list_elem_8);
          }
          i_prot.read_list_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::LongListValue(val));
          }
          received_field_count += 1;
        },
        14 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<OrderedFloat<f64>> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_9 = OrderedFloat::from(i_prot.read_double()?);
            val.push(list_elem_9);
          }
          i_prot.read_list_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::DoubleListValue(val));
          }
          received_field_count += 1;
        },
        15 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<bool> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_10 = i_prot.read_bool()?;
            val.push(list_elem_10);
          }
          i_prot.read_list_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::BooleanListValue(val));
          }
          received_field_count += 1;
        },
        16 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<String> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_11 = i_prot.read_string()?;
            val.push(list_elem_11);
          }
          i_prot.read_list_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::StrListValue(val));
          }
          received_field_count += 1;
        },
        17 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, i16> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_12 = i_prot.read_string()?;
            let map_val_13 = i_prot.read_i16()?;
            val.insert(map_key_12, map_val_13);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::ShortMapValue(val));
          }
          received_field_count += 1;
        },
        18 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, i32> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_14 = i_prot.read_string()?;
            let map_val_15 = i_prot.read_i32()?;
            val.insert(map_key_14, map_val_15);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::IntMapValue(val));
          }
          received_field_count += 1;
        },
        19 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, i64> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_16 = i_prot.read_string()?;
            let map_val_17 = i_prot.read_i64()?;
            val.insert(map_key_16, map_val_17);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::LongMapValue(val));
          }
          received_field_count += 1;
        },
        20 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, OrderedFloat<f64>> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_18 = i_prot.read_string()?;
            let map_val_19 = OrderedFloat::from(i_prot.read_double()?);
            val.insert(map_key_18, map_val_19);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::DoubleMapValue(val));
          }
          received_field_count += 1;
        },
        21 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, bool> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_20 = i_prot.read_string()?;
            let map_val_21 = i_prot.read_bool()?;
            val.insert(map_key_20, map_val_21);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::BooleanMapValue(val));
          }
          received_field_count += 1;
        },
        22 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, String> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_22 = i_prot.read_string()?;
            let map_val_23 = i_prot.read_string()?;
            val.insert(map_key_22, map_val_23);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::StrMapValue(val));
          }
          received_field_count += 1;
        },
        23 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<i32, i32> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_24 = i_prot.read_i32()?;
            let map_val_25 = i_prot.read_i32()?;
            val.insert(map_key_24, map_val_25);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::IntToIntMapValue(val));
          }
          received_field_count += 1;
        },
        24 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<i64, i32> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_26 = i_prot.read_i64()?;
            let map_val_27 = i_prot.read_i32()?;
            val.insert(map_key_26, map_val_27);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::LongToIntMapValue(val));
          }
          received_field_count += 1;
        },
        25 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<i16, OrderedFloat<f64>> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_28 = i_prot.read_i16()?;
            let map_val_29 = OrderedFloat::from(i_prot.read_double()?);
            val.insert(map_key_28, map_val_29);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::ShortToDoubleMapValue(val));
          }
          received_field_count += 1;
        },
        26 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<i64, OrderedFloat<f64>> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_30 = i_prot.read_i64()?;
            let map_val_31 = OrderedFloat::from(i_prot.read_double()?);
            val.insert(map_key_30, map_val_31);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::LongToDoubleMapValue(val));
          }
          received_field_count += 1;
        },
        27 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<i64, String> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_32 = i_prot.read_i64()?;
            let map_val_33 = i_prot.read_string()?;
            val.insert(map_key_32, map_val_33);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::LongToStringMapValue(val));
          }
          received_field_count += 1;
        },
        28 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<i32, OrderedFloat<f64>> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_34 = i_prot.read_i32()?;
            let map_val_35 = OrderedFloat::from(i_prot.read_double()?);
            val.insert(map_key_34, map_val_35);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::IntToDoubleMapValue(val));
          }
          received_field_count += 1;
        },
        29 => {
          let val = i_prot.read_bytes()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::BinaryValue(val));
          }
          received_field_count += 1;
        },
        30 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, Vec<String>> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_36 = i_prot.read_string()?;
            let list_ident = i_prot.read_list_begin()?;
            let mut map_val_37: Vec<String> = Vec::with_capacity(list_ident.size as usize);
            for _ in 0..list_ident.size {
              let list_elem_38 = i_prot.read_string()?;
              map_val_37.push(list_elem_38);
            }
            i_prot.read_list_end()?;
            val.insert(map_key_36, map_val_37);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::StrListMapValue(val));
          }
          received_field_count += 1;
        },
        31 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, Vec<i64>> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_39 = i_prot.read_string()?;
            let list_ident = i_prot.read_list_begin()?;
            let mut map_val_40: Vec<i64> = Vec::with_capacity(list_ident.size as usize);
            for _ in 0..list_ident.size {
              let list_elem_41 = i_prot.read_i64()?;
              map_val_40.push(list_elem_41);
            }
            i_prot.read_list_end()?;
            val.insert(map_key_39, map_val_40);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::LongListMapValue(val));
          }
          received_field_count += 1;
        },
        32 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, Vec<i32>> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_42 = i_prot.read_string()?;
            let list_ident = i_prot.read_list_begin()?;
            let mut map_val_43: Vec<i32> = Vec::with_capacity(list_ident.size as usize);
            for _ in 0..list_ident.size {
              let list_elem_44 = i_prot.read_i32()?;
              map_val_43.push(list_elem_44);
            }
            i_prot.read_list_end()?;
            val.insert(map_key_42, map_val_43);
          }
          i_prot.read_map_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::IntListMapValue(val));
          }
          received_field_count += 1;
        },
        33 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<Vec<u8>> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_45 = i_prot.read_bytes()?;
            val.push(list_elem_45);
          }
          i_prot.read_list_end()?;
          if ret.is_none() {
            ret = Some(AnnotationValue::BinaryListValue(val));
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
            "received empty union from remote AnnotationValue"
          )
        )
      )
    } else if received_field_count > 1 {
      Err(
        thrift::Error::Protocol(
          ProtocolError::new(
            ProtocolErrorKind::InvalidData,
            "received multiple fields for union from remote AnnotationValue"
          )
        )
      )
    } else {
      Ok(ret.expect("return value should have been constructed"))
    }
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AnnotationValue");
    o_prot.write_struct_begin(&struct_ident)?;
    match *self {
      AnnotationValue::ShortValue(f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("shortValue", TType::I16, 1))?;
        o_prot.write_i16(f)?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::IntValue(f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("intValue", TType::I32, 2))?;
        o_prot.write_i32(f)?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::LongValue(f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("longValue", TType::I64, 3))?;
        o_prot.write_i64(f)?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::DoubleValue(f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("doubleValue", TType::Double, 4))?;
        o_prot.write_double(f.into())?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::BooleanValue(f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("booleanValue", TType::Bool, 5))?;
        o_prot.write_bool(f)?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::StrValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("strValue", TType::String, 6))?;
        o_prot.write_string(f)?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::ShortSetValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("shortSetValue", TType::Set, 7))?;
        o_prot.write_set_begin(&TSetIdentifier::new(TType::I16, f.len() as i32))?;
        for e in f {
          o_prot.write_i16(*e)?;
        }
        o_prot.write_set_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::IntSetValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("intSetValue", TType::Set, 8))?;
        o_prot.write_set_begin(&TSetIdentifier::new(TType::I32, f.len() as i32))?;
        for e in f {
          o_prot.write_i32(*e)?;
        }
        o_prot.write_set_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::LongSetValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("longSetValue", TType::Set, 9))?;
        o_prot.write_set_begin(&TSetIdentifier::new(TType::I64, f.len() as i32))?;
        for e in f {
          o_prot.write_i64(*e)?;
        }
        o_prot.write_set_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::StrSetValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("strSetValue", TType::Set, 10))?;
        o_prot.write_set_begin(&TSetIdentifier::new(TType::String, f.len() as i32))?;
        for e in f {
          o_prot.write_string(e)?;
        }
        o_prot.write_set_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::ShortListValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("shortListValue", TType::List, 11))?;
        o_prot.write_list_begin(&TListIdentifier::new(TType::I16, f.len() as i32))?;
        for e in f {
          o_prot.write_i16(*e)?;
        }
        o_prot.write_list_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::IntListValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("intListValue", TType::List, 12))?;
        o_prot.write_list_begin(&TListIdentifier::new(TType::I32, f.len() as i32))?;
        for e in f {
          o_prot.write_i32(*e)?;
        }
        o_prot.write_list_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::LongListValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("longListValue", TType::List, 13))?;
        o_prot.write_list_begin(&TListIdentifier::new(TType::I64, f.len() as i32))?;
        for e in f {
          o_prot.write_i64(*e)?;
        }
        o_prot.write_list_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::DoubleListValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("doubleListValue", TType::List, 14))?;
        o_prot.write_list_begin(&TListIdentifier::new(TType::Double, f.len() as i32))?;
        for e in f {
          o_prot.write_double((*e).into())?;
        }
        o_prot.write_list_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::BooleanListValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("booleanListValue", TType::List, 15))?;
        o_prot.write_list_begin(&TListIdentifier::new(TType::Bool, f.len() as i32))?;
        for e in f {
          o_prot.write_bool(*e)?;
        }
        o_prot.write_list_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::StrListValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("strListValue", TType::List, 16))?;
        o_prot.write_list_begin(&TListIdentifier::new(TType::String, f.len() as i32))?;
        for e in f {
          o_prot.write_string(e)?;
        }
        o_prot.write_list_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::ShortMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("shortMapValue", TType::Map, 17))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::I16, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_string(k)?;
          o_prot.write_i16(*v)?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::IntMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("intMapValue", TType::Map, 18))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::I32, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_string(k)?;
          o_prot.write_i32(*v)?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::LongMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("longMapValue", TType::Map, 19))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::I64, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_string(k)?;
          o_prot.write_i64(*v)?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::DoubleMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("doubleMapValue", TType::Map, 20))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::Double, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_string(k)?;
          o_prot.write_double((*v).into())?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::BooleanMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("booleanMapValue", TType::Map, 21))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::Bool, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_string(k)?;
          o_prot.write_bool(*v)?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::StrMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("strMapValue", TType::Map, 22))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::String, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_string(k)?;
          o_prot.write_string(v)?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::IntToIntMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("intToIntMapValue", TType::Map, 23))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::I32, TType::I32, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_i32(*k)?;
          o_prot.write_i32(*v)?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::LongToIntMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("longToIntMapValue", TType::Map, 24))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::I64, TType::I32, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_i64(*k)?;
          o_prot.write_i32(*v)?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::ShortToDoubleMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("shortToDoubleMapValue", TType::Map, 25))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::I16, TType::Double, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_i16(*k)?;
          o_prot.write_double((*v).into())?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::LongToDoubleMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("longToDoubleMapValue", TType::Map, 26))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::I64, TType::Double, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_i64(*k)?;
          o_prot.write_double((*v).into())?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::LongToStringMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("longToStringMapValue", TType::Map, 27))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::I64, TType::String, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_i64(*k)?;
          o_prot.write_string(v)?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::IntToDoubleMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("intToDoubleMapValue", TType::Map, 28))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::I32, TType::Double, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_i32(*k)?;
          o_prot.write_double((*v).into())?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::BinaryValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("binaryValue", TType::String, 29))?;
        o_prot.write_bytes(f)?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::StrListMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("strListMapValue", TType::Map, 30))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::List, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_string(k)?;
          o_prot.write_list_begin(&TListIdentifier::new(TType::String, v.len() as i32))?;
          for e in v {
            o_prot.write_string(e)?;
          }
          o_prot.write_list_end()?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::LongListMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("longListMapValue", TType::Map, 31))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::List, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_string(k)?;
          o_prot.write_list_begin(&TListIdentifier::new(TType::I64, v.len() as i32))?;
          for e in v {
            o_prot.write_i64(*e)?;
          }
          o_prot.write_list_end()?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::IntListMapValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("intListMapValue", TType::Map, 32))?;
        o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::List, f.len() as i32))?;
        for (k, v) in f {
          o_prot.write_string(k)?;
          o_prot.write_list_begin(&TListIdentifier::new(TType::I32, v.len() as i32))?;
          for e in v {
            o_prot.write_i32(*e)?;
          }
          o_prot.write_list_end()?;
        }
        o_prot.write_map_end()?;
        o_prot.write_field_end()?;
      },
      AnnotationValue::BinaryListValue(ref f) => {
        o_prot.write_field_begin(&TFieldIdentifier::new("binaryListValue", TType::List, 33))?;
        o_prot.write_list_begin(&TListIdentifier::new(TType::String, f.len() as i32))?;
        for e in f {
          o_prot.write_bytes(e)?;
        }
        o_prot.write_list_end()?;
        o_prot.write_field_end()?;
      },
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Annotation {
  pub annotation_value: AnnotationValue,
  pub expires_at_msec: Option<i64>,
  pub created_at_msec: Option<i64>,
}

impl Annotation {
  pub fn new<F2, F3>(annotation_value: AnnotationValue, expires_at_msec: F2, created_at_msec: F3) -> Annotation where F2: Into<Option<i64>>, F3: Into<Option<i64>> {
    Annotation {
      annotation_value,
      expires_at_msec: expires_at_msec.into(),
      created_at_msec: created_at_msec.into(),
    }
  }
}

impl TSerializable for Annotation {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Annotation> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<AnnotationValue> = None;
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
          let val = AnnotationValue::read_from_in_protocol(i_prot)?;
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
    verify_required_field_exists("Annotation.annotation_value", &f_1)?;
    let ret = Annotation {
      annotation_value: f_1.expect("auto-generated code should have checked for presence of required fields"),
      expires_at_msec: f_2,
      created_at_msec: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Annotation");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("annotation_value", TType::Struct, 1))?;
    self.annotation_value.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.expires_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("expires_at_msec", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_msec", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Annotations {
  pub annotations: Option<BTreeMap<AnnotationKey, Annotation>>,
}

impl Annotations {
  pub fn new<F1>(annotations: F1) -> Annotations where F1: Into<Option<BTreeMap<AnnotationKey, Annotation>>> {
    Annotations {
      annotations: annotations.into(),
    }
  }
}

impl TSerializable for Annotations {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Annotations> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<BTreeMap<AnnotationKey, Annotation>> = Some(BTreeMap::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<AnnotationKey, Annotation> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_46 = AnnotationKey::read_from_in_protocol(i_prot)?;
            let map_val_47 = Annotation::read_from_in_protocol(i_prot)?;
            val.insert(map_key_46, map_val_47);
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
    let ret = Annotations {
      annotations: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Annotations");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.annotations {
      o_prot.write_field_begin(&TFieldIdentifier::new("annotations", TType::Map, 1))?;
      o_prot.write_map_begin(&TMapIdentifier::new(TType::Struct, TType::Struct, fld_var.len() as i32))?;
      for (k, v) in fld_var {
        k.write_to_out_protocol(o_prot)?;
        v.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_map_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProfileRedirectTo {
  pub user_id: i64,
  pub screen_name: String,
}

impl ProfileRedirectTo {
  pub fn new(user_id: i64, screen_name: String) -> ProfileRedirectTo {
    ProfileRedirectTo {
      user_id,
      screen_name,
    }
  }
}

impl TSerializable for ProfileRedirectTo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProfileRedirectTo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
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
    verify_required_field_exists("ProfileRedirectTo.user_id", &f_1)?;
    verify_required_field_exists("ProfileRedirectTo.screen_name", &f_2)?;
    let ret = ProfileRedirectTo {
      user_id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      screen_name: f_2.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ProfileRedirectTo");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 1))?;
    o_prot.write_i64(self.user_id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("screen_name", TType::String, 2))?;
    o_prot.write_string(&self.screen_name)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Account {
  pub language: Option<String>,
  pub time_zone: Option<TimeZone>,
  pub geo_enabled: Option<bool>,
  pub has_geotagged_statuses: Option<bool>,
  pub contributors_enabled: Option<bool>,
  pub phx_on: Option<bool>,
  pub phx_opted_out: Option<bool>,
  pub ssl_only: Option<bool>,
  pub ssl_only_opt_out: Option<bool>,
  pub nsfw_view: Option<bool>,
  pub show_all_inline_media: Option<bool>,
  pub trend_location_autoselected: Option<bool>,
  pub trend_location_id: Option<i32>,
  pub created_via: Option<String>,
  pub can_receive_direct_messages_from_all_followers: Option<bool>,
  pub has_facebook_connections: Option<bool>,
  pub sms_password_reset_only: Option<bool>,
  pub use_cookie_personalization: Option<bool>,
  pub creation_ip: Option<String>,
  pub facebook_crosspost_retweets: Option<bool>,
  pub requires_login_verification: Option<bool>,
  pub has_personalized_trends: Option<bool>,
  pub allow_ads_personalization: Option<bool>,
  pub mention_filter: Option<MentionFilter>,
  pub has_highline_profile: Option<bool>,
  pub passed_alcohol_age_challenge: Option<bool>,
  pub allow_media_tagging: Option<AllowMediaTagging>,
  pub has_custom_timelines: Option<bool>,
  pub birthdate: Option<String>,
  pub gender: Option<String>,
  pub abuse_filter: Option<AbuseFilter>,
  pub last_geotag_scrub_msec: Option<i64>,
  pub allow_contributor_request: Option<AllowContributorRequest>,
  pub allow_dms_from: Option<AllowDmsFrom>,
  pub allow_dm_groups_from: Option<AllowDmGroupsFrom>,
  pub notifications_filter_quality: Option<NotificationsFilterQuality>,
  pub notifications_abuse_filter_quality: Option<NotificationsAbuseFilterQuality>,
  pub smart_block: Option<SmartBlock>,
  pub autoplay_disabled: Option<bool>,
  pub smart_mute_enabled: Option<bool>,
  pub analytics_type: Option<AnalyticsType>,
  pub country_code: Option<String>,
  pub address_book_live_sync_enabled: Option<bool>,
  pub dm_receipt_setting: Option<DmReceiptSetting>,
  pub web_push_enabled: Option<bool>,
  pub computed_alt_text_enabled: Option<bool>,
  pub background_location_tracking_enabled: Option<bool>,
  pub allow_authenticated_periscope_requests: Option<bool>,
  pub allow_logged_out_device_personalization: Option<bool>,
  pub allow_location_history_personalization: Option<bool>,
  pub allow_sharing_data_for_third_party_personalization: Option<bool>,
  pub dm_quality_filter: Option<DmQualityFilterSetting>,
  pub allow_device_id_access: Option<bool>,
  pub allow_gambling_ads: Option<bool>,
  pub hide_likes_on_profile: Option<bool>,
  pub allow_video_downloads: Option<bool>,
  pub hide_subscriptions_on_profile: Option<bool>,
  pub hide_verified_checkmark: Option<bool>,
  pub allow_dms_from_v2: Option<AllowDmsFrom>,
  pub enable_parody_account_profile_label: Option<bool>,
  pub always_allow_dms_from_subscribers: Option<bool>,
  pub passkey_auth_enrolled: Option<bool>,
  pub allow_xai_data_sharing: Option<bool>,
  pub allow_for_you_recommendations: Option<bool>,
  pub xpayments_enrolled: Option<bool>,
  pub parody_commentary_fan_label: Option<PCFLabel>,
  pub allow_xai_personalization: Option<bool>,
  pub parody_commentary_fan_label_last_modified_msec: Option<i64>,
}

impl Account {
  pub fn new<F2, F3, F4, F5, F6, F7, F8, F10, F11, F12, F13, F14, F15, F17, F18, F19, F21, F23, F24, F25, F26, F27, F28, F29, F30, F31, F33, F34, F35, F36, F37, F38, F39, F40, F41, F42, F43, F44, F45, F46, F47, F48, F49, F50, F51, F52, F53, F54, F55, F56, F57, F58, F59, F60, F61, F62, F63, F64, F65, F66, F67, F68, F69, F70, F71, F72, F73, F74>(language: F2, time_zone: F3, geo_enabled: F4, has_geotagged_statuses: F5, contributors_enabled: F6, phx_on: F7, phx_opted_out: F8, ssl_only: F10, ssl_only_opt_out: F11, nsfw_view: F12, show_all_inline_media: F13, trend_location_autoselected: F14, trend_location_id: F15, created_via: F17, can_receive_direct_messages_from_all_followers: F18, has_facebook_connections: F19, sms_password_reset_only: F21, use_cookie_personalization: F23, creation_ip: F24, facebook_crosspost_retweets: F25, requires_login_verification: F26, has_personalized_trends: F27, allow_ads_personalization: F28, mention_filter: F29, has_highline_profile: F30, passed_alcohol_age_challenge: F31, allow_media_tagging: F33, has_custom_timelines: F34, birthdate: F35, gender: F36, abuse_filter: F37, last_geotag_scrub_msec: F38, allow_contributor_request: F39, allow_dms_from: F40, allow_dm_groups_from: F41, notifications_filter_quality: F42, notifications_abuse_filter_quality: F43, smart_block: F44, autoplay_disabled: F45, smart_mute_enabled: F46, analytics_type: F47, country_code: F48, address_book_live_sync_enabled: F49, dm_receipt_setting: F50, web_push_enabled: F51, computed_alt_text_enabled: F52, background_location_tracking_enabled: F53, allow_authenticated_periscope_requests: F54, allow_logged_out_device_personalization: F55, allow_location_history_personalization: F56, allow_sharing_data_for_third_party_personalization: F57, dm_quality_filter: F58, allow_device_id_access: F59, allow_gambling_ads: F60, hide_likes_on_profile: F61, allow_video_downloads: F62, hide_subscriptions_on_profile: F63, hide_verified_checkmark: F64, allow_dms_from_v2: F65, enable_parody_account_profile_label: F66, always_allow_dms_from_subscribers: F67, passkey_auth_enrolled: F68, allow_xai_data_sharing: F69, allow_for_you_recommendations: F70, xpayments_enrolled: F71, parody_commentary_fan_label: F72, allow_xai_personalization: F73, parody_commentary_fan_label_last_modified_msec: F74) -> Account where F2: Into<Option<String>>, F3: Into<Option<TimeZone>>, F4: Into<Option<bool>>, F5: Into<Option<bool>>, F6: Into<Option<bool>>, F7: Into<Option<bool>>, F8: Into<Option<bool>>, F10: Into<Option<bool>>, F11: Into<Option<bool>>, F12: Into<Option<bool>>, F13: Into<Option<bool>>, F14: Into<Option<bool>>, F15: Into<Option<i32>>, F17: Into<Option<String>>, F18: Into<Option<bool>>, F19: Into<Option<bool>>, F21: Into<Option<bool>>, F23: Into<Option<bool>>, F24: Into<Option<String>>, F25: Into<Option<bool>>, F26: Into<Option<bool>>, F27: Into<Option<bool>>, F28: Into<Option<bool>>, F29: Into<Option<MentionFilter>>, F30: Into<Option<bool>>, F31: Into<Option<bool>>, F33: Into<Option<AllowMediaTagging>>, F34: Into<Option<bool>>, F35: Into<Option<String>>, F36: Into<Option<String>>, F37: Into<Option<AbuseFilter>>, F38: Into<Option<i64>>, F39: Into<Option<AllowContributorRequest>>, F40: Into<Option<AllowDmsFrom>>, F41: Into<Option<AllowDmGroupsFrom>>, F42: Into<Option<NotificationsFilterQuality>>, F43: Into<Option<NotificationsAbuseFilterQuality>>, F44: Into<Option<SmartBlock>>, F45: Into<Option<bool>>, F46: Into<Option<bool>>, F47: Into<Option<AnalyticsType>>, F48: Into<Option<String>>, F49: Into<Option<bool>>, F50: Into<Option<DmReceiptSetting>>, F51: Into<Option<bool>>, F52: Into<Option<bool>>, F53: Into<Option<bool>>, F54: Into<Option<bool>>, F55: Into<Option<bool>>, F56: Into<Option<bool>>, F57: Into<Option<bool>>, F58: Into<Option<DmQualityFilterSetting>>, F59: Into<Option<bool>>, F60: Into<Option<bool>>, F61: Into<Option<bool>>, F62: Into<Option<bool>>, F63: Into<Option<bool>>, F64: Into<Option<bool>>, F65: Into<Option<AllowDmsFrom>>, F66: Into<Option<bool>>, F67: Into<Option<bool>>, F68: Into<Option<bool>>, F69: Into<Option<bool>>, F70: Into<Option<bool>>, F71: Into<Option<bool>>, F72: Into<Option<PCFLabel>>, F73: Into<Option<bool>>, F74: Into<Option<i64>> {
    Account {
      language: language.into(),
      time_zone: time_zone.into(),
      geo_enabled: geo_enabled.into(),
      has_geotagged_statuses: has_geotagged_statuses.into(),
      contributors_enabled: contributors_enabled.into(),
      phx_on: phx_on.into(),
      phx_opted_out: phx_opted_out.into(),
      ssl_only: ssl_only.into(),
      ssl_only_opt_out: ssl_only_opt_out.into(),
      nsfw_view: nsfw_view.into(),
      show_all_inline_media: show_all_inline_media.into(),
      trend_location_autoselected: trend_location_autoselected.into(),
      trend_location_id: trend_location_id.into(),
      created_via: created_via.into(),
      can_receive_direct_messages_from_all_followers: can_receive_direct_messages_from_all_followers.into(),
      has_facebook_connections: has_facebook_connections.into(),
      sms_password_reset_only: sms_password_reset_only.into(),
      use_cookie_personalization: use_cookie_personalization.into(),
      creation_ip: creation_ip.into(),
      facebook_crosspost_retweets: facebook_crosspost_retweets.into(),
      requires_login_verification: requires_login_verification.into(),
      has_personalized_trends: has_personalized_trends.into(),
      allow_ads_personalization: allow_ads_personalization.into(),
      mention_filter: mention_filter.into(),
      has_highline_profile: has_highline_profile.into(),
      passed_alcohol_age_challenge: passed_alcohol_age_challenge.into(),
      allow_media_tagging: allow_media_tagging.into(),
      has_custom_timelines: has_custom_timelines.into(),
      birthdate: birthdate.into(),
      gender: gender.into(),
      abuse_filter: abuse_filter.into(),
      last_geotag_scrub_msec: last_geotag_scrub_msec.into(),
      allow_contributor_request: allow_contributor_request.into(),
      allow_dms_from: allow_dms_from.into(),
      allow_dm_groups_from: allow_dm_groups_from.into(),
      notifications_filter_quality: notifications_filter_quality.into(),
      notifications_abuse_filter_quality: notifications_abuse_filter_quality.into(),
      smart_block: smart_block.into(),
      autoplay_disabled: autoplay_disabled.into(),
      smart_mute_enabled: smart_mute_enabled.into(),
      analytics_type: analytics_type.into(),
      country_code: country_code.into(),
      address_book_live_sync_enabled: address_book_live_sync_enabled.into(),
      dm_receipt_setting: dm_receipt_setting.into(),
      web_push_enabled: web_push_enabled.into(),
      computed_alt_text_enabled: computed_alt_text_enabled.into(),
      background_location_tracking_enabled: background_location_tracking_enabled.into(),
      allow_authenticated_periscope_requests: allow_authenticated_periscope_requests.into(),
      allow_logged_out_device_personalization: allow_logged_out_device_personalization.into(),
      allow_location_history_personalization: allow_location_history_personalization.into(),
      allow_sharing_data_for_third_party_personalization: allow_sharing_data_for_third_party_personalization.into(),
      dm_quality_filter: dm_quality_filter.into(),
      allow_device_id_access: allow_device_id_access.into(),
      allow_gambling_ads: allow_gambling_ads.into(),
      hide_likes_on_profile: hide_likes_on_profile.into(),
      allow_video_downloads: allow_video_downloads.into(),
      hide_subscriptions_on_profile: hide_subscriptions_on_profile.into(),
      hide_verified_checkmark: hide_verified_checkmark.into(),
      allow_dms_from_v2: allow_dms_from_v2.into(),
      enable_parody_account_profile_label: enable_parody_account_profile_label.into(),
      always_allow_dms_from_subscribers: always_allow_dms_from_subscribers.into(),
      passkey_auth_enrolled: passkey_auth_enrolled.into(),
      allow_xai_data_sharing: allow_xai_data_sharing.into(),
      allow_for_you_recommendations: allow_for_you_recommendations.into(),
      xpayments_enrolled: xpayments_enrolled.into(),
      parody_commentary_fan_label: parody_commentary_fan_label.into(),
      allow_xai_personalization: allow_xai_personalization.into(),
      parody_commentary_fan_label_last_modified_msec: parody_commentary_fan_label_last_modified_msec.into(),
    }
  }
}

impl TSerializable for Account {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Account> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<TimeZone> = None;
    let mut f_4: Option<bool> = Some(false);
    let mut f_5: Option<bool> = Some(false);
    let mut f_6: Option<bool> = Some(false);
    let mut f_7: Option<bool> = Some(false);
    let mut f_8: Option<bool> = Some(false);
    let mut f_10: Option<bool> = Some(false);
    let mut f_11: Option<bool> = Some(false);
    let mut f_12: Option<bool> = Some(false);
    let mut f_13: Option<bool> = Some(false);
    let mut f_14: Option<bool> = Some(false);
    let mut f_15: Option<i32> = Some(0);
    let mut f_17: Option<String> = Some("".to_owned());
    let mut f_18: Option<bool> = Some(false);
    let mut f_19: Option<bool> = Some(false);
    let mut f_21: Option<bool> = None;
    let mut f_23: Option<bool> = None;
    let mut f_24: Option<String> = None;
    let mut f_25: Option<bool> = None;
    let mut f_26: Option<bool> = None;
    let mut f_27: Option<bool> = None;
    let mut f_28: Option<bool> = None;
    let mut f_29: Option<MentionFilter> = None;
    let mut f_30: Option<bool> = None;
    let mut f_31: Option<bool> = None;
    let mut f_33: Option<AllowMediaTagging> = None;
    let mut f_34: Option<bool> = None;
    let mut f_35: Option<String> = None;
    let mut f_36: Option<String> = None;
    let mut f_37: Option<AbuseFilter> = None;
    let mut f_38: Option<i64> = None;
    let mut f_39: Option<AllowContributorRequest> = None;
    let mut f_40: Option<AllowDmsFrom> = None;
    let mut f_41: Option<AllowDmGroupsFrom> = None;
    let mut f_42: Option<NotificationsFilterQuality> = None;
    let mut f_43: Option<NotificationsAbuseFilterQuality> = None;
    let mut f_44: Option<SmartBlock> = None;
    let mut f_45: Option<bool> = None;
    let mut f_46: Option<bool> = None;
    let mut f_47: Option<AnalyticsType> = None;
    let mut f_48: Option<String> = None;
    let mut f_49: Option<bool> = None;
    let mut f_50: Option<DmReceiptSetting> = None;
    let mut f_51: Option<bool> = None;
    let mut f_52: Option<bool> = None;
    let mut f_53: Option<bool> = None;
    let mut f_54: Option<bool> = None;
    let mut f_55: Option<bool> = None;
    let mut f_56: Option<bool> = None;
    let mut f_57: Option<bool> = None;
    let mut f_58: Option<DmQualityFilterSetting> = None;
    let mut f_59: Option<bool> = None;
    let mut f_60: Option<bool> = None;
    let mut f_61: Option<bool> = None;
    let mut f_62: Option<bool> = None;
    let mut f_63: Option<bool> = None;
    let mut f_64: Option<bool> = None;
    let mut f_65: Option<AllowDmsFrom> = None;
    let mut f_66: Option<bool> = None;
    let mut f_67: Option<bool> = None;
    let mut f_68: Option<bool> = None;
    let mut f_69: Option<bool> = None;
    let mut f_70: Option<bool> = None;
    let mut f_71: Option<bool> = None;
    let mut f_72: Option<PCFLabel> = None;
    let mut f_73: Option<bool> = None;
    let mut f_74: Option<i64> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = i_prot.read_string()?;
          f_2 = Some(val);
        },
        3 => {
          let val = TimeZone::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_bool()?;
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
        8 => {
          let val = i_prot.read_bool()?;
          f_8 = Some(val);
        },
        10 => {
          let val = i_prot.read_bool()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_bool()?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_bool()?;
          f_12 = Some(val);
        },
        13 => {
          let val = i_prot.read_bool()?;
          f_13 = Some(val);
        },
        14 => {
          let val = i_prot.read_bool()?;
          f_14 = Some(val);
        },
        15 => {
          let val = i_prot.read_i32()?;
          f_15 = Some(val);
        },
        17 => {
          let val = i_prot.read_string()?;
          f_17 = Some(val);
        },
        18 => {
          let val = i_prot.read_bool()?;
          f_18 = Some(val);
        },
        19 => {
          let val = i_prot.read_bool()?;
          f_19 = Some(val);
        },
        21 => {
          let val = i_prot.read_bool()?;
          f_21 = Some(val);
        },
        23 => {
          let val = i_prot.read_bool()?;
          f_23 = Some(val);
        },
        24 => {
          let val = i_prot.read_string()?;
          f_24 = Some(val);
        },
        25 => {
          let val = i_prot.read_bool()?;
          f_25 = Some(val);
        },
        26 => {
          let val = i_prot.read_bool()?;
          f_26 = Some(val);
        },
        27 => {
          let val = i_prot.read_bool()?;
          f_27 = Some(val);
        },
        28 => {
          let val = i_prot.read_bool()?;
          f_28 = Some(val);
        },
        29 => {
          let val = MentionFilter::read_from_in_protocol(i_prot)?;
          f_29 = Some(val);
        },
        30 => {
          let val = i_prot.read_bool()?;
          f_30 = Some(val);
        },
        31 => {
          let val = i_prot.read_bool()?;
          f_31 = Some(val);
        },
        33 => {
          let val = AllowMediaTagging::read_from_in_protocol(i_prot)?;
          f_33 = Some(val);
        },
        34 => {
          let val = i_prot.read_bool()?;
          f_34 = Some(val);
        },
        35 => {
          let val = i_prot.read_string()?;
          f_35 = Some(val);
        },
        36 => {
          let val = i_prot.read_string()?;
          f_36 = Some(val);
        },
        37 => {
          let val = AbuseFilter::read_from_in_protocol(i_prot)?;
          f_37 = Some(val);
        },
        38 => {
          let val = i_prot.read_i64()?;
          f_38 = Some(val);
        },
        39 => {
          let val = AllowContributorRequest::read_from_in_protocol(i_prot)?;
          f_39 = Some(val);
        },
        40 => {
          let val = AllowDmsFrom::read_from_in_protocol(i_prot)?;
          f_40 = Some(val);
        },
        41 => {
          let val = AllowDmGroupsFrom::read_from_in_protocol(i_prot)?;
          f_41 = Some(val);
        },
        42 => {
          let val = NotificationsFilterQuality::read_from_in_protocol(i_prot)?;
          f_42 = Some(val);
        },
        43 => {
          let val = NotificationsAbuseFilterQuality::read_from_in_protocol(i_prot)?;
          f_43 = Some(val);
        },
        44 => {
          let val = SmartBlock::read_from_in_protocol(i_prot)?;
          f_44 = Some(val);
        },
        45 => {
          let val = i_prot.read_bool()?;
          f_45 = Some(val);
        },
        46 => {
          let val = i_prot.read_bool()?;
          f_46 = Some(val);
        },
        47 => {
          let val = AnalyticsType::read_from_in_protocol(i_prot)?;
          f_47 = Some(val);
        },
        48 => {
          let val = i_prot.read_string()?;
          f_48 = Some(val);
        },
        49 => {
          let val = i_prot.read_bool()?;
          f_49 = Some(val);
        },
        50 => {
          let val = DmReceiptSetting::read_from_in_protocol(i_prot)?;
          f_50 = Some(val);
        },
        51 => {
          let val = i_prot.read_bool()?;
          f_51 = Some(val);
        },
        52 => {
          let val = i_prot.read_bool()?;
          f_52 = Some(val);
        },
        53 => {
          let val = i_prot.read_bool()?;
          f_53 = Some(val);
        },
        54 => {
          let val = i_prot.read_bool()?;
          f_54 = Some(val);
        },
        55 => {
          let val = i_prot.read_bool()?;
          f_55 = Some(val);
        },
        56 => {
          let val = i_prot.read_bool()?;
          f_56 = Some(val);
        },
        57 => {
          let val = i_prot.read_bool()?;
          f_57 = Some(val);
        },
        58 => {
          let val = DmQualityFilterSetting::read_from_in_protocol(i_prot)?;
          f_58 = Some(val);
        },
        59 => {
          let val = i_prot.read_bool()?;
          f_59 = Some(val);
        },
        60 => {
          let val = i_prot.read_bool()?;
          f_60 = Some(val);
        },
        61 => {
          let val = i_prot.read_bool()?;
          f_61 = Some(val);
        },
        62 => {
          let val = i_prot.read_bool()?;
          f_62 = Some(val);
        },
        63 => {
          let val = i_prot.read_bool()?;
          f_63 = Some(val);
        },
        64 => {
          let val = i_prot.read_bool()?;
          f_64 = Some(val);
        },
        65 => {
          let val = AllowDmsFrom::read_from_in_protocol(i_prot)?;
          f_65 = Some(val);
        },
        66 => {
          let val = i_prot.read_bool()?;
          f_66 = Some(val);
        },
        67 => {
          let val = i_prot.read_bool()?;
          f_67 = Some(val);
        },
        68 => {
          let val = i_prot.read_bool()?;
          f_68 = Some(val);
        },
        69 => {
          let val = i_prot.read_bool()?;
          f_69 = Some(val);
        },
        70 => {
          let val = i_prot.read_bool()?;
          f_70 = Some(val);
        },
        71 => {
          let val = i_prot.read_bool()?;
          f_71 = Some(val);
        },
        72 => {
          let val = PCFLabel::read_from_in_protocol(i_prot)?;
          f_72 = Some(val);
        },
        73 => {
          let val = i_prot.read_bool()?;
          f_73 = Some(val);
        },
        74 => {
          let val = i_prot.read_i64()?;
          f_74 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Account {
      language: f_2,
      time_zone: f_3,
      geo_enabled: f_4,
      has_geotagged_statuses: f_5,
      contributors_enabled: f_6,
      phx_on: f_7,
      phx_opted_out: f_8,
      ssl_only: f_10,
      ssl_only_opt_out: f_11,
      nsfw_view: f_12,
      show_all_inline_media: f_13,
      trend_location_autoselected: f_14,
      trend_location_id: f_15,
      created_via: f_17,
      can_receive_direct_messages_from_all_followers: f_18,
      has_facebook_connections: f_19,
      sms_password_reset_only: f_21,
      use_cookie_personalization: f_23,
      creation_ip: f_24,
      facebook_crosspost_retweets: f_25,
      requires_login_verification: f_26,
      has_personalized_trends: f_27,
      allow_ads_personalization: f_28,
      mention_filter: f_29,
      has_highline_profile: f_30,
      passed_alcohol_age_challenge: f_31,
      allow_media_tagging: f_33,
      has_custom_timelines: f_34,
      birthdate: f_35,
      gender: f_36,
      abuse_filter: f_37,
      last_geotag_scrub_msec: f_38,
      allow_contributor_request: f_39,
      allow_dms_from: f_40,
      allow_dm_groups_from: f_41,
      notifications_filter_quality: f_42,
      notifications_abuse_filter_quality: f_43,
      smart_block: f_44,
      autoplay_disabled: f_45,
      smart_mute_enabled: f_46,
      analytics_type: f_47,
      country_code: f_48,
      address_book_live_sync_enabled: f_49,
      dm_receipt_setting: f_50,
      web_push_enabled: f_51,
      computed_alt_text_enabled: f_52,
      background_location_tracking_enabled: f_53,
      allow_authenticated_periscope_requests: f_54,
      allow_logged_out_device_personalization: f_55,
      allow_location_history_personalization: f_56,
      allow_sharing_data_for_third_party_personalization: f_57,
      dm_quality_filter: f_58,
      allow_device_id_access: f_59,
      allow_gambling_ads: f_60,
      hide_likes_on_profile: f_61,
      allow_video_downloads: f_62,
      hide_subscriptions_on_profile: f_63,
      hide_verified_checkmark: f_64,
      allow_dms_from_v2: f_65,
      enable_parody_account_profile_label: f_66,
      always_allow_dms_from_subscribers: f_67,
      passkey_auth_enrolled: f_68,
      allow_xai_data_sharing: f_69,
      allow_for_you_recommendations: f_70,
      xpayments_enrolled: f_71,
      parody_commentary_fan_label: f_72,
      allow_xai_personalization: f_73,
      parody_commentary_fan_label_last_modified_msec: f_74,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Account");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.language {
      o_prot.write_field_begin(&TFieldIdentifier::new("language", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.time_zone {
      o_prot.write_field_begin(&TFieldIdentifier::new("time_zone", TType::Struct, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.geo_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("geo_enabled", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_geotagged_statuses {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_geotagged_statuses", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.contributors_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("contributors_enabled", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.phx_on {
      o_prot.write_field_begin(&TFieldIdentifier::new("phx_on", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.phx_opted_out {
      o_prot.write_field_begin(&TFieldIdentifier::new("phx_opted_out", TType::Bool, 8))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.ssl_only {
      o_prot.write_field_begin(&TFieldIdentifier::new("ssl_only", TType::Bool, 10))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.ssl_only_opt_out {
      o_prot.write_field_begin(&TFieldIdentifier::new("ssl_only_opt_out", TType::Bool, 11))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.nsfw_view {
      o_prot.write_field_begin(&TFieldIdentifier::new("nsfw_view", TType::Bool, 12))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.show_all_inline_media {
      o_prot.write_field_begin(&TFieldIdentifier::new("show_all_inline_media", TType::Bool, 13))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.trend_location_autoselected {
      o_prot.write_field_begin(&TFieldIdentifier::new("trend_location_autoselected", TType::Bool, 14))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.trend_location_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("trend_location_id", TType::I32, 15))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.created_via {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_via", TType::String, 17))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.can_receive_direct_messages_from_all_followers {
      o_prot.write_field_begin(&TFieldIdentifier::new("can_receive_direct_messages_from_all_followers", TType::Bool, 18))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_facebook_connections {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_facebook_connections", TType::Bool, 19))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.sms_password_reset_only {
      o_prot.write_field_begin(&TFieldIdentifier::new("sms_password_reset_only", TType::Bool, 21))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.use_cookie_personalization {
      o_prot.write_field_begin(&TFieldIdentifier::new("use_cookie_personalization", TType::Bool, 23))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.creation_ip {
      o_prot.write_field_begin(&TFieldIdentifier::new("creation_ip", TType::String, 24))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.facebook_crosspost_retweets {
      o_prot.write_field_begin(&TFieldIdentifier::new("facebook_crosspost_retweets", TType::Bool, 25))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.requires_login_verification {
      o_prot.write_field_begin(&TFieldIdentifier::new("requires_login_verification", TType::Bool, 26))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_personalized_trends {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_personalized_trends", TType::Bool, 27))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_ads_personalization {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_ads_personalization", TType::Bool, 28))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.mention_filter {
      o_prot.write_field_begin(&TFieldIdentifier::new("mention_filter", TType::I32, 29))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_highline_profile {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_highline_profile", TType::Bool, 30))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.passed_alcohol_age_challenge {
      o_prot.write_field_begin(&TFieldIdentifier::new("passed_alcohol_age_challenge", TType::Bool, 31))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.allow_media_tagging {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_media_tagging", TType::I32, 33))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_custom_timelines {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_custom_timelines", TType::Bool, 34))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.birthdate {
      o_prot.write_field_begin(&TFieldIdentifier::new("birthdate", TType::String, 35))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.gender {
      o_prot.write_field_begin(&TFieldIdentifier::new("gender", TType::String, 36))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.abuse_filter {
      o_prot.write_field_begin(&TFieldIdentifier::new("abuse_filter", TType::I32, 37))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.last_geotag_scrub_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("last_geotag_scrub_msec", TType::I64, 38))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.allow_contributor_request {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_contributor_request", TType::I32, 39))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.allow_dms_from {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_dms_from", TType::I32, 40))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.allow_dm_groups_from {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_dm_groups_from", TType::I32, 41))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.notifications_filter_quality {
      o_prot.write_field_begin(&TFieldIdentifier::new("notifications_filter_quality", TType::I32, 42))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.notifications_abuse_filter_quality {
      o_prot.write_field_begin(&TFieldIdentifier::new("notifications_abuse_filter_quality", TType::I32, 43))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.smart_block {
      o_prot.write_field_begin(&TFieldIdentifier::new("smart_block", TType::Struct, 44))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.autoplay_disabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("autoplay_disabled", TType::Bool, 45))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.smart_mute_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("smart_mute_enabled", TType::Bool, 46))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.analytics_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("analytics_type", TType::I32, 47))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.country_code {
      o_prot.write_field_begin(&TFieldIdentifier::new("country_code", TType::String, 48))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.address_book_live_sync_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("address_book_live_sync_enabled", TType::Bool, 49))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.dm_receipt_setting {
      o_prot.write_field_begin(&TFieldIdentifier::new("dm_receipt_setting", TType::I32, 50))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.web_push_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("web_push_enabled", TType::Bool, 51))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.computed_alt_text_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("computed_alt_text_enabled", TType::Bool, 52))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.background_location_tracking_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("background_location_tracking_enabled", TType::Bool, 53))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_authenticated_periscope_requests {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_authenticated_periscope_requests", TType::Bool, 54))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_logged_out_device_personalization {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_logged_out_device_personalization", TType::Bool, 55))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_location_history_personalization {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_location_history_personalization", TType::Bool, 56))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_sharing_data_for_third_party_personalization {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_sharing_data_for_third_party_personalization", TType::Bool, 57))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.dm_quality_filter {
      o_prot.write_field_begin(&TFieldIdentifier::new("dm_quality_filter", TType::I32, 58))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_device_id_access {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_device_id_access", TType::Bool, 59))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_gambling_ads {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_gambling_ads", TType::Bool, 60))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.hide_likes_on_profile {
      o_prot.write_field_begin(&TFieldIdentifier::new("hide_likes_on_profile", TType::Bool, 61))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_video_downloads {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_video_downloads", TType::Bool, 62))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.hide_subscriptions_on_profile {
      o_prot.write_field_begin(&TFieldIdentifier::new("hide_subscriptions_on_profile", TType::Bool, 63))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.hide_verified_checkmark {
      o_prot.write_field_begin(&TFieldIdentifier::new("hide_verified_checkmark", TType::Bool, 64))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.allow_dms_from_v2 {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_dms_from_v2", TType::I32, 65))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.enable_parody_account_profile_label {
      o_prot.write_field_begin(&TFieldIdentifier::new("enable_parody_account_profile_label", TType::Bool, 66))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.always_allow_dms_from_subscribers {
      o_prot.write_field_begin(&TFieldIdentifier::new("always_allow_dms_from_subscribers", TType::Bool, 67))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.passkey_auth_enrolled {
      o_prot.write_field_begin(&TFieldIdentifier::new("passkey_auth_enrolled", TType::Bool, 68))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_xai_data_sharing {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_xai_data_sharing", TType::Bool, 69))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_for_you_recommendations {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_for_you_recommendations", TType::Bool, 70))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.xpayments_enrolled {
      o_prot.write_field_begin(&TFieldIdentifier::new("xpayments_enrolled", TType::Bool, 71))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.parody_commentary_fan_label {
      o_prot.write_field_begin(&TFieldIdentifier::new("parody_commentary_fan_label", TType::I32, 72))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.allow_xai_personalization {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_xai_personalization", TType::Bool, 73))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.parody_commentary_fan_label_last_modified_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("parody_commentary_fan_label_last_modified_msec", TType::I64, 74))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AboutThisAccountPreferences {
  pub inferred_location_resolution_preference: Option<InferredLocationResolution>,
}

impl AboutThisAccountPreferences {
  pub fn new<F1>(inferred_location_resolution_preference: F1) -> AboutThisAccountPreferences where F1: Into<Option<InferredLocationResolution>> {
    AboutThisAccountPreferences {
      inferred_location_resolution_preference: inferred_location_resolution_preference.into(),
    }
  }
}

impl TSerializable for AboutThisAccountPreferences {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AboutThisAccountPreferences> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<InferredLocationResolution> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = InferredLocationResolution::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = AboutThisAccountPreferences {
      inferred_location_resolution_preference: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AboutThisAccountPreferences");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.inferred_location_resolution_preference {
      o_prot.write_field_begin(&TFieldIdentifier::new("inferred_location_resolution_preference", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExtendedAccount {
  pub allow_grok_memory: Option<bool>,
  pub about_this_account_preferences: Option<AboutThisAccountPreferences>,
  pub grok_character_id: Option<String>,
  pub persona_media_gen_access: Option<PersonaMediaGenAccess>,
}

impl ExtendedAccount {
  pub fn new<F1, F2, F3, F4>(allow_grok_memory: F1, about_this_account_preferences: F2, grok_character_id: F3, persona_media_gen_access: F4) -> ExtendedAccount where F1: Into<Option<bool>>, F2: Into<Option<AboutThisAccountPreferences>>, F3: Into<Option<String>>, F4: Into<Option<PersonaMediaGenAccess>> {
    ExtendedAccount {
      allow_grok_memory: allow_grok_memory.into(),
      about_this_account_preferences: about_this_account_preferences.into(),
      grok_character_id: grok_character_id.into(),
      persona_media_gen_access: persona_media_gen_access.into(),
    }
  }
}

impl TSerializable for ExtendedAccount {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ExtendedAccount> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = None;
    let mut f_2: Option<AboutThisAccountPreferences> = None;
    let mut f_3: Option<String> = None;
    let mut f_4: Option<PersonaMediaGenAccess> = None;
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
        2 => {
          let val = AboutThisAccountPreferences::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let val = PersonaMediaGenAccess::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ExtendedAccount {
      allow_grok_memory: f_1,
      about_this_account_preferences: f_2,
      grok_character_id: f_3,
      persona_media_gen_access: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ExtendedAccount");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.allow_grok_memory {
      o_prot.write_field_begin(&TFieldIdentifier::new("allow_grok_memory", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.about_this_account_preferences {
      o_prot.write_field_begin(&TFieldIdentifier::new("about_this_account_preferences", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.grok_character_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("grok_character_id", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.persona_media_gen_access {
      o_prot.write_field_begin(&TFieldIdentifier::new("persona_media_gen_access", TType::I32, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sleep {
  pub sleeping: Option<bool>,
  pub scheduled_sleeping: Option<bool>,
  pub scheduled_sleep_start_hour: Option<i32>,
  pub scheduled_sleep_end_hour: Option<i32>,
}

impl Sleep {
  pub fn new<F2, F3, F4, F5>(sleeping: F2, scheduled_sleeping: F3, scheduled_sleep_start_hour: F4, scheduled_sleep_end_hour: F5) -> Sleep where F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<i32>>, F5: Into<Option<i32>> {
    Sleep {
      sleeping: sleeping.into(),
      scheduled_sleeping: scheduled_sleeping.into(),
      scheduled_sleep_start_hour: scheduled_sleep_start_hour.into(),
      scheduled_sleep_end_hour: scheduled_sleep_end_hour.into(),
    }
  }
}

impl TSerializable for Sleep {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Sleep> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<bool> = None;
    let mut f_3: Option<bool> = Some(false);
    let mut f_4: Option<i32> = Some(0);
    let mut f_5: Option<i32> = Some(0);
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = i_prot.read_bool()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
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
    let ret = Sleep {
      sleeping: f_2,
      scheduled_sleeping: f_3,
      scheduled_sleep_start_hour: f_4,
      scheduled_sleep_end_hour: f_5,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Sleep");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.sleeping {
      o_prot.write_field_begin(&TFieldIdentifier::new("sleeping", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.scheduled_sleeping {
      o_prot.write_field_begin(&TFieldIdentifier::new("scheduled_sleeping", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.scheduled_sleep_start_hour {
      o_prot.write_field_begin(&TFieldIdentifier::new("scheduled_sleep_start_hour", TType::I32, 4))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.scheduled_sleep_end_hour {
      o_prot.write_field_begin(&TFieldIdentifier::new("scheduled_sleep_end_hour", TType::I32, 5))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Prompts {
  pub has_dismissed_geo_promo: Option<bool>,
  pub has_dismissed_mobile_tips: Option<bool>,
  pub has_dismissed_follow_tips: Option<bool>,
  pub cookie_personalization_prompt_displayed: Option<bool>,
  pub gazebo_form_status: Option<BTreeMap<String, GazeboFormStatus>>,
}

impl Prompts {
  pub fn new<F2, F3, F4, F5, F6>(has_dismissed_geo_promo: F2, has_dismissed_mobile_tips: F3, has_dismissed_follow_tips: F4, cookie_personalization_prompt_displayed: F5, gazebo_form_status: F6) -> Prompts where F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<bool>>, F5: Into<Option<bool>>, F6: Into<Option<BTreeMap<String, GazeboFormStatus>>> {
    Prompts {
      has_dismissed_geo_promo: has_dismissed_geo_promo.into(),
      has_dismissed_mobile_tips: has_dismissed_mobile_tips.into(),
      has_dismissed_follow_tips: has_dismissed_follow_tips.into(),
      cookie_personalization_prompt_displayed: cookie_personalization_prompt_displayed.into(),
      gazebo_form_status: gazebo_form_status.into(),
    }
  }
}

impl TSerializable for Prompts {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Prompts> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<bool> = None;
    let mut f_3: Option<bool> = Some(false);
    let mut f_4: Option<bool> = Some(false);
    let mut f_5: Option<bool> = None;
    let mut f_6: Option<BTreeMap<String, GazeboFormStatus>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = i_prot.read_bool()?;
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
        5 => {
          let val = i_prot.read_bool()?;
          f_5 = Some(val);
        },
        6 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, GazeboFormStatus> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_48 = i_prot.read_string()?;
            let map_val_49 = GazeboFormStatus::read_from_in_protocol(i_prot)?;
            val.insert(map_key_48, map_val_49);
          }
          i_prot.read_map_end()?;
          f_6 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Prompts {
      has_dismissed_geo_promo: f_2,
      has_dismissed_mobile_tips: f_3,
      has_dismissed_follow_tips: f_4,
      cookie_personalization_prompt_displayed: f_5,
      gazebo_form_status: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Prompts");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.has_dismissed_geo_promo {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_dismissed_geo_promo", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_dismissed_mobile_tips {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_dismissed_mobile_tips", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_dismissed_follow_tips {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_dismissed_follow_tips", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.cookie_personalization_prompt_displayed {
      o_prot.write_field_begin(&TFieldIdentifier::new("cookie_personalization_prompt_displayed", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.gazebo_form_status {
      o_prot.write_field_begin(&TFieldIdentifier::new("gazebo_form_status", TType::Map, 6))?;
      o_prot.write_map_begin(&TMapIdentifier::new(TType::String, TType::I32, fld_var.len() as i32))?;
      for (k, v) in fld_var {
        o_prot.write_string(k)?;
        v.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_map_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NotificationExtension1 {
  pub send_address_book_notification_email: Option<bool>,
  pub send_similar_people_email: Option<bool>,
  pub send_shared_tweet_email: Option<bool>,
  pub send_conversation_track_email: Option<bool>,
  pub send_magic_recs_email: Option<bool>,
  pub send_twitter_emails: Option<bool>,
  pub send_new_contributor_email: Option<bool>,
  pub send_added_as_contributor_email: Option<bool>,
  pub send_smb_sales_marketing_email: Option<bool>,
  pub send_brand_insights_email: Option<bool>,
  pub reserved_11: Option<bool>,
  pub reserved_12: Option<bool>,
  pub reserved_13: Option<bool>,
  pub send_retweeted_retweet_email: Option<EmailSettingState>,
  pub send_favorited_retweet_email: Option<EmailSettingState>,
  pub send_retweeted_mention_email: Option<EmailSettingState>,
  pub send_favorited_mention_email: Option<EmailSettingState>,
  pub reserved_18: Option<EmailSettingState>,
  pub send_performance_digest: Option<DigestSchedule>,
  pub send_login_notification_email: Option<bool>,
  pub send_money_email: Option<bool>,
}

impl NotificationExtension1 {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20, F21>(send_address_book_notification_email: F1, send_similar_people_email: F2, send_shared_tweet_email: F3, send_conversation_track_email: F4, send_magic_recs_email: F5, send_twitter_emails: F6, send_new_contributor_email: F7, send_added_as_contributor_email: F8, send_smb_sales_marketing_email: F9, send_brand_insights_email: F10, reserved_11: F11, reserved_12: F12, reserved_13: F13, send_retweeted_retweet_email: F14, send_favorited_retweet_email: F15, send_retweeted_mention_email: F16, send_favorited_mention_email: F17, reserved_18: F18, send_performance_digest: F19, send_login_notification_email: F20, send_money_email: F21) -> NotificationExtension1 where F1: Into<Option<bool>>, F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<bool>>, F5: Into<Option<bool>>, F6: Into<Option<bool>>, F7: Into<Option<bool>>, F8: Into<Option<bool>>, F9: Into<Option<bool>>, F10: Into<Option<bool>>, F11: Into<Option<bool>>, F12: Into<Option<bool>>, F13: Into<Option<bool>>, F14: Into<Option<EmailSettingState>>, F15: Into<Option<EmailSettingState>>, F16: Into<Option<EmailSettingState>>, F17: Into<Option<EmailSettingState>>, F18: Into<Option<EmailSettingState>>, F19: Into<Option<DigestSchedule>>, F20: Into<Option<bool>>, F21: Into<Option<bool>> {
    NotificationExtension1 {
      send_address_book_notification_email: send_address_book_notification_email.into(),
      send_similar_people_email: send_similar_people_email.into(),
      send_shared_tweet_email: send_shared_tweet_email.into(),
      send_conversation_track_email: send_conversation_track_email.into(),
      send_magic_recs_email: send_magic_recs_email.into(),
      send_twitter_emails: send_twitter_emails.into(),
      send_new_contributor_email: send_new_contributor_email.into(),
      send_added_as_contributor_email: send_added_as_contributor_email.into(),
      send_smb_sales_marketing_email: send_smb_sales_marketing_email.into(),
      send_brand_insights_email: send_brand_insights_email.into(),
      reserved_11: reserved_11.into(),
      reserved_12: reserved_12.into(),
      reserved_13: reserved_13.into(),
      send_retweeted_retweet_email: send_retweeted_retweet_email.into(),
      send_favorited_retweet_email: send_favorited_retweet_email.into(),
      send_retweeted_mention_email: send_retweeted_mention_email.into(),
      send_favorited_mention_email: send_favorited_mention_email.into(),
      reserved_18: reserved_18.into(),
      send_performance_digest: send_performance_digest.into(),
      send_login_notification_email: send_login_notification_email.into(),
      send_money_email: send_money_email.into(),
    }
  }
}

impl TSerializable for NotificationExtension1 {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<NotificationExtension1> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = None;
    let mut f_2: Option<bool> = None;
    let mut f_3: Option<bool> = None;
    let mut f_4: Option<bool> = None;
    let mut f_5: Option<bool> = None;
    let mut f_6: Option<bool> = None;
    let mut f_7: Option<bool> = None;
    let mut f_8: Option<bool> = None;
    let mut f_9: Option<bool> = None;
    let mut f_10: Option<bool> = None;
    let mut f_11: Option<bool> = None;
    let mut f_12: Option<bool> = None;
    let mut f_13: Option<bool> = None;
    let mut f_14: Option<EmailSettingState> = None;
    let mut f_15: Option<EmailSettingState> = None;
    let mut f_16: Option<EmailSettingState> = None;
    let mut f_17: Option<EmailSettingState> = None;
    let mut f_18: Option<EmailSettingState> = None;
    let mut f_19: Option<DigestSchedule> = None;
    let mut f_20: Option<bool> = None;
    let mut f_21: Option<bool> = None;
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
        2 => {
          let val = i_prot.read_bool()?;
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
        8 => {
          let val = i_prot.read_bool()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_bool()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_bool()?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_bool()?;
          f_12 = Some(val);
        },
        13 => {
          let val = i_prot.read_bool()?;
          f_13 = Some(val);
        },
        14 => {
          let val = EmailSettingState::read_from_in_protocol(i_prot)?;
          f_14 = Some(val);
        },
        15 => {
          let val = EmailSettingState::read_from_in_protocol(i_prot)?;
          f_15 = Some(val);
        },
        16 => {
          let val = EmailSettingState::read_from_in_protocol(i_prot)?;
          f_16 = Some(val);
        },
        17 => {
          let val = EmailSettingState::read_from_in_protocol(i_prot)?;
          f_17 = Some(val);
        },
        18 => {
          let val = EmailSettingState::read_from_in_protocol(i_prot)?;
          f_18 = Some(val);
        },
        19 => {
          let val = DigestSchedule::read_from_in_protocol(i_prot)?;
          f_19 = Some(val);
        },
        20 => {
          let val = i_prot.read_bool()?;
          f_20 = Some(val);
        },
        21 => {
          let val = i_prot.read_bool()?;
          f_21 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = NotificationExtension1 {
      send_address_book_notification_email: f_1,
      send_similar_people_email: f_2,
      send_shared_tweet_email: f_3,
      send_conversation_track_email: f_4,
      send_magic_recs_email: f_5,
      send_twitter_emails: f_6,
      send_new_contributor_email: f_7,
      send_added_as_contributor_email: f_8,
      send_smb_sales_marketing_email: f_9,
      send_brand_insights_email: f_10,
      reserved_11: f_11,
      reserved_12: f_12,
      reserved_13: f_13,
      send_retweeted_retweet_email: f_14,
      send_favorited_retweet_email: f_15,
      send_retweeted_mention_email: f_16,
      send_favorited_mention_email: f_17,
      reserved_18: f_18,
      send_performance_digest: f_19,
      send_login_notification_email: f_20,
      send_money_email: f_21,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("NotificationExtension1");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.send_address_book_notification_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_address_book_notification_email", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_similar_people_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_similar_people_email", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_shared_tweet_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_shared_tweet_email", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_conversation_track_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_conversation_track_email", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_magic_recs_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_magic_recs_email", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_twitter_emails {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_twitter_emails", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_new_contributor_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_new_contributor_email", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_added_as_contributor_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_added_as_contributor_email", TType::Bool, 8))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_smb_sales_marketing_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_smb_sales_marketing_email", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_brand_insights_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_brand_insights_email", TType::Bool, 10))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.reserved_11 {
      o_prot.write_field_begin(&TFieldIdentifier::new("reserved_11", TType::Bool, 11))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.reserved_12 {
      o_prot.write_field_begin(&TFieldIdentifier::new("reserved_12", TType::Bool, 12))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.reserved_13 {
      o_prot.write_field_begin(&TFieldIdentifier::new("reserved_13", TType::Bool, 13))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.send_retweeted_retweet_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_retweeted_retweet_email", TType::I32, 14))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.send_favorited_retweet_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_favorited_retweet_email", TType::I32, 15))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.send_retweeted_mention_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_retweeted_mention_email", TType::I32, 16))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.send_favorited_mention_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_favorited_mention_email", TType::I32, 17))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.reserved_18 {
      o_prot.write_field_begin(&TFieldIdentifier::new("reserved_18", TType::I32, 18))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.send_performance_digest {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_performance_digest", TType::I32, 19))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_login_notification_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_login_notification_email", TType::Bool, 20))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_money_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_money_email", TType::Bool, 21))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Notification {
  pub send_mention_email: Option<EmailSettingState>,
  pub send_listed_email: Option<EmailSettingState>,
  pub send_favorited_email: Option<EmailSettingState>,
  pub send_retweeted_email: Option<EmailSettingState>,
  pub send_new_follower_email_only_if_following: Option<bool>,
  pub send_account_updates_email: Option<bool>,
  pub send_new_friend_email: Option<bool>,
  pub send_new_direct_text_email: Option<bool>,
  pub send_email_newsletter: Option<bool>,
  pub send_individual_follower_emails: Option<bool>,
  pub email_confirmed: Option<EmailConfirmedState>,
  pub send_resurrection_email: Option<bool>,
  pub send_network_digest: Option<DigestSchedule>,
  pub send_follow_recs_email: Option<bool>,
  pub send_network_activity_email: Option<bool>,
  pub send_activation_email: Option<bool>,
  pub send_partner_email: Option<bool>,
  pub send_survey_email: Option<bool>,
  pub send_email_vit_weekly: Option<bool>,
  pub notification_extension_1: Option<NotificationExtension1>,
}

impl Notification {
  pub fn new<F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20, F34>(send_mention_email: F2, send_listed_email: F3, send_favorited_email: F4, send_retweeted_email: F5, send_new_follower_email_only_if_following: F6, send_account_updates_email: F7, send_new_friend_email: F8, send_new_direct_text_email: F9, send_email_newsletter: F10, send_individual_follower_emails: F11, email_confirmed: F12, send_resurrection_email: F13, send_network_digest: F14, send_follow_recs_email: F15, send_network_activity_email: F16, send_activation_email: F17, send_partner_email: F18, send_survey_email: F19, send_email_vit_weekly: F20, notification_extension_1: F34) -> Notification where F2: Into<Option<EmailSettingState>>, F3: Into<Option<EmailSettingState>>, F4: Into<Option<EmailSettingState>>, F5: Into<Option<EmailSettingState>>, F6: Into<Option<bool>>, F7: Into<Option<bool>>, F8: Into<Option<bool>>, F9: Into<Option<bool>>, F10: Into<Option<bool>>, F11: Into<Option<bool>>, F12: Into<Option<EmailConfirmedState>>, F13: Into<Option<bool>>, F14: Into<Option<DigestSchedule>>, F15: Into<Option<bool>>, F16: Into<Option<bool>>, F17: Into<Option<bool>>, F18: Into<Option<bool>>, F19: Into<Option<bool>>, F20: Into<Option<bool>>, F34: Into<Option<NotificationExtension1>> {
    Notification {
      send_mention_email: send_mention_email.into(),
      send_listed_email: send_listed_email.into(),
      send_favorited_email: send_favorited_email.into(),
      send_retweeted_email: send_retweeted_email.into(),
      send_new_follower_email_only_if_following: send_new_follower_email_only_if_following.into(),
      send_account_updates_email: send_account_updates_email.into(),
      send_new_friend_email: send_new_friend_email.into(),
      send_new_direct_text_email: send_new_direct_text_email.into(),
      send_email_newsletter: send_email_newsletter.into(),
      send_individual_follower_emails: send_individual_follower_emails.into(),
      email_confirmed: email_confirmed.into(),
      send_resurrection_email: send_resurrection_email.into(),
      send_network_digest: send_network_digest.into(),
      send_follow_recs_email: send_follow_recs_email.into(),
      send_network_activity_email: send_network_activity_email.into(),
      send_activation_email: send_activation_email.into(),
      send_partner_email: send_partner_email.into(),
      send_survey_email: send_survey_email.into(),
      send_email_vit_weekly: send_email_vit_weekly.into(),
      notification_extension_1: notification_extension_1.into(),
    }
  }
}

impl TSerializable for Notification {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Notification> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<EmailSettingState> = None;
    let mut f_3: Option<EmailSettingState> = None;
    let mut f_4: Option<EmailSettingState> = None;
    let mut f_5: Option<EmailSettingState> = None;
    let mut f_6: Option<bool> = Some(false);
    let mut f_7: Option<bool> = Some(false);
    let mut f_8: Option<bool> = Some(false);
    let mut f_9: Option<bool> = Some(false);
    let mut f_10: Option<bool> = Some(false);
    let mut f_11: Option<bool> = None;
    let mut f_12: Option<EmailConfirmedState> = None;
    let mut f_13: Option<bool> = None;
    let mut f_14: Option<DigestSchedule> = None;
    let mut f_15: Option<bool> = None;
    let mut f_16: Option<bool> = None;
    let mut f_17: Option<bool> = None;
    let mut f_18: Option<bool> = None;
    let mut f_19: Option<bool> = None;
    let mut f_20: Option<bool> = None;
    let mut f_34: Option<NotificationExtension1> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = EmailSettingState::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = EmailSettingState::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = EmailSettingState::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        5 => {
          let val = EmailSettingState::read_from_in_protocol(i_prot)?;
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
        8 => {
          let val = i_prot.read_bool()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_bool()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_bool()?;
          f_11 = Some(val);
        },
        12 => {
          let val = EmailConfirmedState::read_from_in_protocol(i_prot)?;
          f_12 = Some(val);
        },
        13 => {
          let val = i_prot.read_bool()?;
          f_13 = Some(val);
        },
        14 => {
          let val = DigestSchedule::read_from_in_protocol(i_prot)?;
          f_14 = Some(val);
        },
        15 => {
          let val = i_prot.read_bool()?;
          f_15 = Some(val);
        },
        16 => {
          let val = i_prot.read_bool()?;
          f_16 = Some(val);
        },
        17 => {
          let val = i_prot.read_bool()?;
          f_17 = Some(val);
        },
        18 => {
          let val = i_prot.read_bool()?;
          f_18 = Some(val);
        },
        19 => {
          let val = i_prot.read_bool()?;
          f_19 = Some(val);
        },
        20 => {
          let val = i_prot.read_bool()?;
          f_20 = Some(val);
        },
        34 => {
          let val = NotificationExtension1::read_from_in_protocol(i_prot)?;
          f_34 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Notification {
      send_mention_email: f_2,
      send_listed_email: f_3,
      send_favorited_email: f_4,
      send_retweeted_email: f_5,
      send_new_follower_email_only_if_following: f_6,
      send_account_updates_email: f_7,
      send_new_friend_email: f_8,
      send_new_direct_text_email: f_9,
      send_email_newsletter: f_10,
      send_individual_follower_emails: f_11,
      email_confirmed: f_12,
      send_resurrection_email: f_13,
      send_network_digest: f_14,
      send_follow_recs_email: f_15,
      send_network_activity_email: f_16,
      send_activation_email: f_17,
      send_partner_email: f_18,
      send_survey_email: f_19,
      send_email_vit_weekly: f_20,
      notification_extension_1: f_34,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Notification");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.send_mention_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_mention_email", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.send_listed_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_listed_email", TType::I32, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.send_favorited_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_favorited_email", TType::I32, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.send_retweeted_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_retweeted_email", TType::I32, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_new_follower_email_only_if_following {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_new_follower_email_only_if_following", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_account_updates_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_account_updates_email", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_new_friend_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_new_friend_email", TType::Bool, 8))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_new_direct_text_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_new_direct_text_email", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_email_newsletter {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_email_newsletter", TType::Bool, 10))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_individual_follower_emails {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_individual_follower_emails", TType::Bool, 11))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.email_confirmed {
      o_prot.write_field_begin(&TFieldIdentifier::new("email_confirmed", TType::I32, 12))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_resurrection_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_resurrection_email", TType::Bool, 13))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.send_network_digest {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_network_digest", TType::I32, 14))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_follow_recs_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_follow_recs_email", TType::Bool, 15))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_network_activity_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_network_activity_email", TType::Bool, 16))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_activation_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_activation_email", TType::Bool, 17))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_partner_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_partner_email", TType::Bool, 18))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_survey_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_survey_email", TType::Bool, 19))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.send_email_vit_weekly {
      o_prot.write_field_begin(&TFieldIdentifier::new("send_email_vit_weekly", TType::Bool, 20))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.notification_extension_1 {
      o_prot.write_field_begin(&TFieldIdentifier::new("notification_extension_1", TType::Struct, 34))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForceLoginChallenge {
  pub created_at_msec: Option<i64>,
  pub challenge_type: Option<i16>,
}

impl ForceLoginChallenge {
  pub fn new<F1, F2>(created_at_msec: F1, challenge_type: F2) -> ForceLoginChallenge where F1: Into<Option<i64>>, F2: Into<Option<i16>> {
    ForceLoginChallenge {
      created_at_msec: created_at_msec.into(),
      challenge_type: challenge_type.into(),
    }
  }
}

impl TSerializable for ForceLoginChallenge {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ForceLoginChallenge> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i16> = None;
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
    let ret = ForceLoginChallenge {
      created_at_msec: f_1,
      challenge_type: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ForceLoginChallenge");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.created_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_msec", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.challenge_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("challenge_type", TType::I16, 2))?;
      o_prot.write_i16(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SuspensionDetails {
  pub permanently_suspended: Option<bool>,
  pub actor_name: Option<String>,
  pub suspended_at_msec: Option<i64>,
}

impl SuspensionDetails {
  pub fn new<F1, F2, F3>(permanently_suspended: F1, actor_name: F2, suspended_at_msec: F3) -> SuspensionDetails where F1: Into<Option<bool>>, F2: Into<Option<String>>, F3: Into<Option<i64>> {
    SuspensionDetails {
      permanently_suspended: permanently_suspended.into(),
      actor_name: actor_name.into(),
      suspended_at_msec: suspended_at_msec.into(),
    }
  }
}

impl TSerializable for SuspensionDetails {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SuspensionDetails> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = Some(false);
    let mut f_2: Option<String> = None;
    let mut f_3: Option<i64> = None;
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
        2 => {
          let val = i_prot.read_string()?;
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
    let ret = SuspensionDetails {
      permanently_suspended: f_1,
      actor_name: f_2,
      suspended_at_msec: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SuspensionDetails");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.permanently_suspended {
      o_prot.write_field_begin(&TFieldIdentifier::new("permanently_suspended", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.actor_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("actor_name", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.suspended_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("suspended_at_msec", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConsentViolation {
  pub in_violation: Option<bool>,
  pub updated_at_msec: Option<i64>,
  pub violation_from_country_code: Option<String>,
  pub determined_from: Option<ConsentFlow>,
}

impl ConsentViolation {
  pub fn new<F1, F2, F3, F4>(in_violation: F1, updated_at_msec: F2, violation_from_country_code: F3, determined_from: F4) -> ConsentViolation where F1: Into<Option<bool>>, F2: Into<Option<i64>>, F3: Into<Option<String>>, F4: Into<Option<ConsentFlow>> {
    ConsentViolation {
      in_violation: in_violation.into(),
      updated_at_msec: updated_at_msec.into(),
      violation_from_country_code: violation_from_country_code.into(),
      determined_from: determined_from.into(),
    }
  }
}

impl TSerializable for ConsentViolation {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConsentViolation> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = Some(false);
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<ConsentFlow> = None;
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
        2 => {
          let val = i_prot.read_i64()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let val = ConsentFlow::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ConsentViolation {
      in_violation: f_1,
      updated_at_msec: f_2,
      violation_from_country_code: f_3,
      determined_from: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConsentViolation");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.in_violation {
      o_prot.write_field_begin(&TFieldIdentifier::new("in_violation", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.updated_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("updated_at_msec", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.violation_from_country_code {
      o_prot.write_field_begin(&TFieldIdentifier::new("violation_from_country_code", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.determined_from {
      o_prot.write_field_begin(&TFieldIdentifier::new("determined_from", TType::I32, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConsentResponse {
  pub response_value: Option<ConsentResponseValue>,
  pub response_at_msec: Option<i64>,
  pub consent_version: Option<i64>,
  pub responded_from_country_code: Option<String>,
  pub indicators: Option<BTreeSet<DerivedConsentIndicator>>,
}

impl ConsentResponse {
  pub fn new<F1, F2, F3, F4, F6>(response_value: F1, response_at_msec: F2, consent_version: F3, responded_from_country_code: F4, indicators: F6) -> ConsentResponse where F1: Into<Option<ConsentResponseValue>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<String>>, F6: Into<Option<BTreeSet<DerivedConsentIndicator>>> {
    ConsentResponse {
      response_value: response_value.into(),
      response_at_msec: response_at_msec.into(),
      consent_version: consent_version.into(),
      responded_from_country_code: responded_from_country_code.into(),
      indicators: indicators.into(),
    }
  }
}

impl TSerializable for ConsentResponse {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ConsentResponse> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<ConsentResponseValue> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<String> = Some("".to_owned());
    let mut f_6: Option<BTreeSet<DerivedConsentIndicator>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = ConsentResponseValue::read_from_in_protocol(i_prot)?;
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
        6 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<DerivedConsentIndicator> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_50 = DerivedConsentIndicator::read_from_in_protocol(i_prot)?;
            val.insert(set_elem_50);
          }
          i_prot.read_set_end()?;
          f_6 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ConsentResponse {
      response_value: f_1,
      response_at_msec: f_2,
      consent_version: f_3,
      responded_from_country_code: f_4,
      indicators: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ConsentResponse");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.response_value {
      o_prot.write_field_begin(&TFieldIdentifier::new("response_value", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.response_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("response_at_msec", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.consent_version {
      o_prot.write_field_begin(&TFieldIdentifier::new("consent_version", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.responded_from_country_code {
      o_prot.write_field_begin(&TFieldIdentifier::new("responded_from_country_code", TType::String, 4))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.indicators {
      o_prot.write_field_begin(&TFieldIdentifier::new("indicators", TType::Set, 6))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::I32, fld_var.len() as i32))?;
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
pub struct Consent {
  pub violation: Option<ConsentViolation>,
  pub response: Option<ConsentResponse>,
}

impl Consent {
  pub fn new<F1, F2>(violation: F1, response: F2) -> Consent where F1: Into<Option<ConsentViolation>>, F2: Into<Option<ConsentResponse>> {
    Consent {
      violation: violation.into(),
      response: response.into(),
    }
  }
}

impl TSerializable for Consent {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Consent> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<ConsentViolation> = None;
    let mut f_2: Option<ConsentResponse> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = ConsentViolation::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = ConsentResponse::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Consent {
      violation: f_1,
      response: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Consent");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.violation {
      o_prot.write_field_begin(&TFieldIdentifier::new("violation", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.response {
      o_prot.write_field_begin(&TFieldIdentifier::new("response", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U13Remediation {
  pub reason: Option<U13RemediationReason>,
  pub state: Option<U13RemediationState>,
}

impl U13Remediation {
  pub fn new<F1, F2>(reason: F1, state: F2) -> U13Remediation where F1: Into<Option<U13RemediationReason>>, F2: Into<Option<U13RemediationState>> {
    U13Remediation {
      reason: reason.into(),
      state: state.into(),
    }
  }
}

impl TSerializable for U13Remediation {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<U13Remediation> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<U13RemediationReason> = None;
    let mut f_2: Option<U13RemediationState> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = U13RemediationReason::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = U13RemediationState::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = U13Remediation {
      reason: f_1,
      state: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("U13Remediation");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.reason {
      o_prot.write_field_begin(&TFieldIdentifier::new("reason", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.state {
      o_prot.write_field_begin(&TFieldIdentifier::new("state", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U13RestorationDataStatus {
  pub tweets_deleted: Option<bool>,
  pub gd_settings_reset: Option<bool>,
  pub sgs_graphs_erased: Option<bool>,
  pub timelines_erased: Option<bool>,
  pub acl_erased: Option<bool>,
  pub moments_erased: Option<bool>,
  pub media_library_erased: Option<bool>,
  pub video_analytics_erased: Option<bool>,
  pub convosvc_inbox_initial_state_cache_flushed: Option<bool>,
  pub partial_erasure_processed: Option<bool>,
  pub geo_locations_erased: Option<bool>,
}

impl U13RestorationDataStatus {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11>(tweets_deleted: F1, gd_settings_reset: F2, sgs_graphs_erased: F3, timelines_erased: F4, acl_erased: F5, moments_erased: F6, media_library_erased: F7, video_analytics_erased: F8, convosvc_inbox_initial_state_cache_flushed: F9, partial_erasure_processed: F10, geo_locations_erased: F11) -> U13RestorationDataStatus where F1: Into<Option<bool>>, F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<bool>>, F5: Into<Option<bool>>, F6: Into<Option<bool>>, F7: Into<Option<bool>>, F8: Into<Option<bool>>, F9: Into<Option<bool>>, F10: Into<Option<bool>>, F11: Into<Option<bool>> {
    U13RestorationDataStatus {
      tweets_deleted: tweets_deleted.into(),
      gd_settings_reset: gd_settings_reset.into(),
      sgs_graphs_erased: sgs_graphs_erased.into(),
      timelines_erased: timelines_erased.into(),
      acl_erased: acl_erased.into(),
      moments_erased: moments_erased.into(),
      media_library_erased: media_library_erased.into(),
      video_analytics_erased: video_analytics_erased.into(),
      convosvc_inbox_initial_state_cache_flushed: convosvc_inbox_initial_state_cache_flushed.into(),
      partial_erasure_processed: partial_erasure_processed.into(),
      geo_locations_erased: geo_locations_erased.into(),
    }
  }
}

impl TSerializable for U13RestorationDataStatus {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<U13RestorationDataStatus> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = None;
    let mut f_2: Option<bool> = None;
    let mut f_3: Option<bool> = None;
    let mut f_4: Option<bool> = None;
    let mut f_5: Option<bool> = None;
    let mut f_6: Option<bool> = None;
    let mut f_7: Option<bool> = None;
    let mut f_8: Option<bool> = None;
    let mut f_9: Option<bool> = None;
    let mut f_10: Option<bool> = None;
    let mut f_11: Option<bool> = None;
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
        2 => {
          let val = i_prot.read_bool()?;
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
        8 => {
          let val = i_prot.read_bool()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_bool()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_bool()?;
          f_11 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = U13RestorationDataStatus {
      tweets_deleted: f_1,
      gd_settings_reset: f_2,
      sgs_graphs_erased: f_3,
      timelines_erased: f_4,
      acl_erased: f_5,
      moments_erased: f_6,
      media_library_erased: f_7,
      video_analytics_erased: f_8,
      convosvc_inbox_initial_state_cache_flushed: f_9,
      partial_erasure_processed: f_10,
      geo_locations_erased: f_11,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("U13RestorationDataStatus");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.tweets_deleted {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweets_deleted", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.gd_settings_reset {
      o_prot.write_field_begin(&TFieldIdentifier::new("gd_settings_reset", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.sgs_graphs_erased {
      o_prot.write_field_begin(&TFieldIdentifier::new("sgs_graphs_erased", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.timelines_erased {
      o_prot.write_field_begin(&TFieldIdentifier::new("timelines_erased", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.acl_erased {
      o_prot.write_field_begin(&TFieldIdentifier::new("acl_erased", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.moments_erased {
      o_prot.write_field_begin(&TFieldIdentifier::new("moments_erased", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.media_library_erased {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_library_erased", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.video_analytics_erased {
      o_prot.write_field_begin(&TFieldIdentifier::new("video_analytics_erased", TType::Bool, 8))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.convosvc_inbox_initial_state_cache_flushed {
      o_prot.write_field_begin(&TFieldIdentifier::new("convosvc_inbox_initial_state_cache_flushed", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.partial_erasure_processed {
      o_prot.write_field_begin(&TFieldIdentifier::new("partial_erasure_processed", TType::Bool, 10))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.geo_locations_erased {
      o_prot.write_field_begin(&TFieldIdentifier::new("geo_locations_erased", TType::Bool, 11))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U13Restoration {
  pub status: Option<U13RestorationStatus>,
  pub created_at_msec: Option<i64>,
  pub updated_at_msec: Option<i64>,
  pub restoration_started_at_msec: Option<i64>,
  pub data_status: Option<U13RestorationDataStatus>,
}

impl U13Restoration {
  pub fn new<F1, F2, F3, F4, F5>(status: F1, created_at_msec: F2, updated_at_msec: F3, restoration_started_at_msec: F4, data_status: F5) -> U13Restoration where F1: Into<Option<U13RestorationStatus>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<U13RestorationDataStatus>> {
    U13Restoration {
      status: status.into(),
      created_at_msec: created_at_msec.into(),
      updated_at_msec: updated_at_msec.into(),
      restoration_started_at_msec: restoration_started_at_msec.into(),
      data_status: data_status.into(),
    }
  }
}

impl TSerializable for U13Restoration {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<U13Restoration> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<U13RestorationStatus> = None;
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<i64> = None;
    let mut f_4: Option<i64> = None;
    let mut f_5: Option<U13RestorationDataStatus> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = U13RestorationStatus::read_from_in_protocol(i_prot)?;
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
          let val = U13RestorationDataStatus::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = U13Restoration {
      status: f_1,
      created_at_msec: f_2,
      updated_at_msec: f_3,
      restoration_started_at_msec: f_4,
      data_status: f_5,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("U13Restoration");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.status {
      o_prot.write_field_begin(&TFieldIdentifier::new("status", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_msec", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.updated_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("updated_at_msec", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.restoration_started_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("restoration_started_at_msec", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.data_status {
      o_prot.write_field_begin(&TFieldIdentifier::new("data_status", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Compliance {
  pub u13_restoration: Option<U13Restoration>,
  pub eligible_for_u13_restoration: Option<bool>,
  pub eligible_for_parental_consent: Option<bool>,
}

impl Compliance {
  pub fn new<F1, F2, F3>(u13_restoration: F1, eligible_for_u13_restoration: F2, eligible_for_parental_consent: F3) -> Compliance where F1: Into<Option<U13Restoration>>, F2: Into<Option<bool>>, F3: Into<Option<bool>> {
    Compliance {
      u13_restoration: u13_restoration.into(),
      eligible_for_u13_restoration: eligible_for_u13_restoration.into(),
      eligible_for_parental_consent: eligible_for_parental_consent.into(),
    }
  }
}

impl TSerializable for Compliance {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Compliance> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<U13Restoration> = None;
    let mut f_2: Option<bool> = None;
    let mut f_3: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = U13Restoration::read_from_in_protocol(i_prot)?;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Compliance {
      u13_restoration: f_1,
      eligible_for_u13_restoration: f_2,
      eligible_for_parental_consent: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Compliance");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.u13_restoration {
      o_prot.write_field_begin(&TFieldIdentifier::new("u13Restoration", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.eligible_for_u13_restoration {
      o_prot.write_field_begin(&TFieldIdentifier::new("eligible_for_u13_restoration", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.eligible_for_parental_consent {
      o_prot.write_field_begin(&TFieldIdentifier::new("eligible_for_parental_consent", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Consents {
  pub age_consent: Option<Consent>,
  pub settings_consent: Option<Consent>,
  pub terms_consent: Option<Consent>,
  pub u13_remediation: Option<U13Remediation>,
}

impl Consents {
  pub fn new<F1, F2, F3, F4>(age_consent: F1, settings_consent: F2, terms_consent: F3, u13_remediation: F4) -> Consents where F1: Into<Option<Consent>>, F2: Into<Option<Consent>>, F3: Into<Option<Consent>>, F4: Into<Option<U13Remediation>> {
    Consents {
      age_consent: age_consent.into(),
      settings_consent: settings_consent.into(),
      terms_consent: terms_consent.into(),
      u13_remediation: u13_remediation.into(),
    }
  }
}

impl TSerializable for Consents {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Consents> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Consent> = None;
    let mut f_2: Option<Consent> = None;
    let mut f_3: Option<Consent> = None;
    let mut f_4: Option<U13Remediation> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = Consent::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = Consent::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = Consent::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = U13Remediation::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Consents {
      age_consent: f_1,
      settings_consent: f_2,
      terms_consent: f_3,
      u13_remediation: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Consents");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.age_consent {
      o_prot.write_field_begin(&TFieldIdentifier::new("age_consent", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.settings_consent {
      o_prot.write_field_begin(&TFieldIdentifier::new("settings_consent", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.terms_consent {
      o_prot.write_field_begin(&TFieldIdentifier::new("terms_consent", TType::Struct, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.u13_remediation {
      o_prot.write_field_begin(&TFieldIdentifier::new("u13Remediation", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IdentityVerification {
  pub is_identity_verified: Option<bool>,
  pub is_identity_verified_label_hidden: Option<bool>,
}

impl IdentityVerification {
  pub fn new<F1, F2>(is_identity_verified: F1, is_identity_verified_label_hidden: F2) -> IdentityVerification where F1: Into<Option<bool>>, F2: Into<Option<bool>> {
    IdentityVerification {
      is_identity_verified: is_identity_verified.into(),
      is_identity_verified_label_hidden: is_identity_verified_label_hidden.into(),
    }
  }
}

impl TSerializable for IdentityVerification {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<IdentityVerification> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = None;
    let mut f_2: Option<bool> = None;
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
    let ret = IdentityVerification {
      is_identity_verified: f_1,
      is_identity_verified_label_hidden: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("IdentityVerification");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.is_identity_verified {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_identity_verified", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_identity_verified_label_hidden {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_identity_verified_label_hidden", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VerifiedOrganizationDetails {
  pub is_verified_organization: Option<bool>,
  pub is_verified_organization_affiliate: Option<bool>,
}

impl VerifiedOrganizationDetails {
  pub fn new<F1, F2>(is_verified_organization: F1, is_verified_organization_affiliate: F2) -> VerifiedOrganizationDetails where F1: Into<Option<bool>>, F2: Into<Option<bool>> {
    VerifiedOrganizationDetails {
      is_verified_organization: is_verified_organization.into(),
      is_verified_organization_affiliate: is_verified_organization_affiliate.into(),
    }
  }
}

impl TSerializable for VerifiedOrganizationDetails {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<VerifiedOrganizationDetails> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = None;
    let mut f_2: Option<bool> = None;
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
    let ret = VerifiedOrganizationDetails {
      is_verified_organization: f_1,
      is_verified_organization_affiliate: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("VerifiedOrganizationDetails");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.is_verified_organization {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_verified_organization", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_verified_organization_affiliate {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_verified_organization_affiliate", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Safety {
  pub is_protected: Option<bool>,
  pub verified: Option<bool>,
  pub deactivated: Option<bool>,
  pub suspended: Option<bool>,
  pub restricted: Option<bool>,
  pub nsfw_user: Option<bool>,
  pub nsfw_admin: Option<bool>,
  pub has_takedown: Option<bool>,
  pub has_role: Option<bool>,
  pub frictionless_follower_state: Option<FrictionlessFollowerState>,
  pub erased: Option<bool>,
  pub force_password_reset: Option<bool>,
  pub protect_password_reset: Option<bool>,
  pub has_saved_searches: Option<bool>,
  pub is_lifeline_institution: Option<bool>,
  pub translation_enabled: Option<bool>,
  pub frictionless_follower_type: Option<FrictionlessFollowerType>,
  pub admin_password_last_reset_msec: Option<i64>,
  pub needs_phone_verification: Option<bool>,
  pub deactivated_at_msec: Option<i64>,
  pub manhattan_updated_at_msec: Option<i64>,
  pub force_login_challenge: Option<ForceLoginChallenge>,
  pub suspension_details: Option<SuspensionDetails>,
  pub access_policy: Option<AccessPolicy>,
  pub has_labels: Option<bool>,
  pub signup_category: Option<SignupCategory>,
  pub signup_trust_level: Option<SignupTrustLevel>,
  pub signup_country_code: Option<String>,
  pub require_password_login: Option<bool>,
  pub has_extended_profile: Option<bool>,
  pub last_annotated_at_msec: Option<i64>,
  pub universal_quality_filtering: Option<UniversalQualityFiltering>,
  pub access_policy_expiry_msec: Option<i64>,
  pub should_offboard: Option<bool>,
  pub should_offboard_updated_at_msec: Option<i64>,
  pub offboarded: Option<bool>,
  pub offboarded_updated_at_msec: Option<i64>,
  pub require_some_consent: Option<bool>,
  pub consents: Option<Consents>,
  pub deactivation_timespan: Option<DeactivationTimespan>,
  pub compromised_state: Option<CompromisedState>,
  pub compromised_at_msec: Option<i64>,
  pub preerased: Option<bool>,
  pub verified_type: Option<VerifiedType>,
  pub external_user_update_msec: Option<i64>,
  pub is_blue_verified: Option<bool>,
  pub blue_verified_expiration_msec: Option<i64>,
  pub blue_checkmark_hidden_reason: Option<BlueCheckmarkHiddenReason>,
  pub identity_verification: Option<IdentityVerification>,
  pub verified_organization_details: Option<VerifiedOrganizationDetails>,
  pub skip_rate_limit: Option<bool>,
  pub community_verified: Option<bool>,
  pub subscription_level: Option<SubscriptionLevel>,
  pub signup_creation_source: Option<SignupCreationSource>,
}

impl Safety {
  pub fn new<F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F25, F26, F27, F28, F29, F30, F31, F32, F33, F34, F35, F36, F37, F38, F39, F40, F41, F42, F43, F44, F45, F46, F47, F49, F50, F51, F52, F53, F54, F55, F56, F57>(is_protected: F2, verified: F3, deactivated: F4, suspended: F5, restricted: F6, nsfw_user: F7, nsfw_admin: F8, has_takedown: F9, has_role: F10, frictionless_follower_state: F11, erased: F12, force_password_reset: F13, protect_password_reset: F14, has_saved_searches: F15, is_lifeline_institution: F16, translation_enabled: F17, frictionless_follower_type: F18, admin_password_last_reset_msec: F19, needs_phone_verification: F20, deactivated_at_msec: F21, manhattan_updated_at_msec: F22, force_login_challenge: F23, suspension_details: F25, access_policy: F26, has_labels: F27, signup_category: F28, signup_trust_level: F29, signup_country_code: F30, require_password_login: F31, has_extended_profile: F32, last_annotated_at_msec: F33, universal_quality_filtering: F34, access_policy_expiry_msec: F35, should_offboard: F36, should_offboard_updated_at_msec: F37, offboarded: F38, offboarded_updated_at_msec: F39, require_some_consent: F40, consents: F41, deactivation_timespan: F42, compromised_state: F43, compromised_at_msec: F44, preerased: F45, verified_type: F46, external_user_update_msec: F47, is_blue_verified: F49, blue_verified_expiration_msec: F50, blue_checkmark_hidden_reason: F51, identity_verification: F52, verified_organization_details: F53, skip_rate_limit: F54, community_verified: F55, subscription_level: F56, signup_creation_source: F57) -> Safety where F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<bool>>, F5: Into<Option<bool>>, F6: Into<Option<bool>>, F7: Into<Option<bool>>, F8: Into<Option<bool>>, F9: Into<Option<bool>>, F10: Into<Option<bool>>, F11: Into<Option<FrictionlessFollowerState>>, F12: Into<Option<bool>>, F13: Into<Option<bool>>, F14: Into<Option<bool>>, F15: Into<Option<bool>>, F16: Into<Option<bool>>, F17: Into<Option<bool>>, F18: Into<Option<FrictionlessFollowerType>>, F19: Into<Option<i64>>, F20: Into<Option<bool>>, F21: Into<Option<i64>>, F22: Into<Option<i64>>, F23: Into<Option<ForceLoginChallenge>>, F25: Into<Option<SuspensionDetails>>, F26: Into<Option<AccessPolicy>>, F27: Into<Option<bool>>, F28: Into<Option<SignupCategory>>, F29: Into<Option<SignupTrustLevel>>, F30: Into<Option<String>>, F31: Into<Option<bool>>, F32: Into<Option<bool>>, F33: Into<Option<i64>>, F34: Into<Option<UniversalQualityFiltering>>, F35: Into<Option<i64>>, F36: Into<Option<bool>>, F37: Into<Option<i64>>, F38: Into<Option<bool>>, F39: Into<Option<i64>>, F40: Into<Option<bool>>, F41: Into<Option<Consents>>, F42: Into<Option<DeactivationTimespan>>, F43: Into<Option<CompromisedState>>, F44: Into<Option<i64>>, F45: Into<Option<bool>>, F46: Into<Option<VerifiedType>>, F47: Into<Option<i64>>, F49: Into<Option<bool>>, F50: Into<Option<i64>>, F51: Into<Option<BlueCheckmarkHiddenReason>>, F52: Into<Option<IdentityVerification>>, F53: Into<Option<VerifiedOrganizationDetails>>, F54: Into<Option<bool>>, F55: Into<Option<bool>>, F56: Into<Option<SubscriptionLevel>>, F57: Into<Option<SignupCreationSource>> {
    Safety {
      is_protected: is_protected.into(),
      verified: verified.into(),
      deactivated: deactivated.into(),
      suspended: suspended.into(),
      restricted: restricted.into(),
      nsfw_user: nsfw_user.into(),
      nsfw_admin: nsfw_admin.into(),
      has_takedown: has_takedown.into(),
      has_role: has_role.into(),
      frictionless_follower_state: frictionless_follower_state.into(),
      erased: erased.into(),
      force_password_reset: force_password_reset.into(),
      protect_password_reset: protect_password_reset.into(),
      has_saved_searches: has_saved_searches.into(),
      is_lifeline_institution: is_lifeline_institution.into(),
      translation_enabled: translation_enabled.into(),
      frictionless_follower_type: frictionless_follower_type.into(),
      admin_password_last_reset_msec: admin_password_last_reset_msec.into(),
      needs_phone_verification: needs_phone_verification.into(),
      deactivated_at_msec: deactivated_at_msec.into(),
      manhattan_updated_at_msec: manhattan_updated_at_msec.into(),
      force_login_challenge: force_login_challenge.into(),
      suspension_details: suspension_details.into(),
      access_policy: access_policy.into(),
      has_labels: has_labels.into(),
      signup_category: signup_category.into(),
      signup_trust_level: signup_trust_level.into(),
      signup_country_code: signup_country_code.into(),
      require_password_login: require_password_login.into(),
      has_extended_profile: has_extended_profile.into(),
      last_annotated_at_msec: last_annotated_at_msec.into(),
      universal_quality_filtering: universal_quality_filtering.into(),
      access_policy_expiry_msec: access_policy_expiry_msec.into(),
      should_offboard: should_offboard.into(),
      should_offboard_updated_at_msec: should_offboard_updated_at_msec.into(),
      offboarded: offboarded.into(),
      offboarded_updated_at_msec: offboarded_updated_at_msec.into(),
      require_some_consent: require_some_consent.into(),
      consents: consents.into(),
      deactivation_timespan: deactivation_timespan.into(),
      compromised_state: compromised_state.into(),
      compromised_at_msec: compromised_at_msec.into(),
      preerased: preerased.into(),
      verified_type: verified_type.into(),
      external_user_update_msec: external_user_update_msec.into(),
      is_blue_verified: is_blue_verified.into(),
      blue_verified_expiration_msec: blue_verified_expiration_msec.into(),
      blue_checkmark_hidden_reason: blue_checkmark_hidden_reason.into(),
      identity_verification: identity_verification.into(),
      verified_organization_details: verified_organization_details.into(),
      skip_rate_limit: skip_rate_limit.into(),
      community_verified: community_verified.into(),
      subscription_level: subscription_level.into(),
      signup_creation_source: signup_creation_source.into(),
    }
  }
}

impl TSerializable for Safety {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Safety> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<bool> = Some(false);
    let mut f_3: Option<bool> = Some(false);
    let mut f_4: Option<bool> = Some(false);
    let mut f_5: Option<bool> = Some(false);
    let mut f_6: Option<bool> = Some(false);
    let mut f_7: Option<bool> = Some(false);
    let mut f_8: Option<bool> = Some(false);
    let mut f_9: Option<bool> = Some(false);
    let mut f_10: Option<bool> = Some(false);
    let mut f_11: Option<FrictionlessFollowerState> = None;
    let mut f_12: Option<bool> = None;
    let mut f_13: Option<bool> = None;
    let mut f_14: Option<bool> = None;
    let mut f_15: Option<bool> = None;
    let mut f_16: Option<bool> = None;
    let mut f_17: Option<bool> = None;
    let mut f_18: Option<FrictionlessFollowerType> = None;
    let mut f_19: Option<i64> = None;
    let mut f_20: Option<bool> = None;
    let mut f_21: Option<i64> = None;
    let mut f_22: Option<i64> = None;
    let mut f_23: Option<ForceLoginChallenge> = None;
    let mut f_25: Option<SuspensionDetails> = None;
    let mut f_26: Option<AccessPolicy> = None;
    let mut f_27: Option<bool> = None;
    let mut f_28: Option<SignupCategory> = None;
    let mut f_29: Option<SignupTrustLevel> = None;
    let mut f_30: Option<String> = None;
    let mut f_31: Option<bool> = None;
    let mut f_32: Option<bool> = None;
    let mut f_33: Option<i64> = None;
    let mut f_34: Option<UniversalQualityFiltering> = None;
    let mut f_35: Option<i64> = None;
    let mut f_36: Option<bool> = None;
    let mut f_37: Option<i64> = None;
    let mut f_38: Option<bool> = None;
    let mut f_39: Option<i64> = None;
    let mut f_40: Option<bool> = None;
    let mut f_41: Option<Consents> = None;
    let mut f_42: Option<DeactivationTimespan> = None;
    let mut f_43: Option<CompromisedState> = None;
    let mut f_44: Option<i64> = None;
    let mut f_45: Option<bool> = None;
    let mut f_46: Option<VerifiedType> = None;
    let mut f_47: Option<i64> = None;
    let mut f_49: Option<bool> = None;
    let mut f_50: Option<i64> = None;
    let mut f_51: Option<BlueCheckmarkHiddenReason> = None;
    let mut f_52: Option<IdentityVerification> = None;
    let mut f_53: Option<VerifiedOrganizationDetails> = None;
    let mut f_54: Option<bool> = None;
    let mut f_55: Option<bool> = None;
    let mut f_56: Option<SubscriptionLevel> = None;
    let mut f_57: Option<SignupCreationSource> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let val = i_prot.read_bool()?;
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
        8 => {
          let val = i_prot.read_bool()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_bool()?;
          f_10 = Some(val);
        },
        11 => {
          let val = FrictionlessFollowerState::read_from_in_protocol(i_prot)?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_bool()?;
          f_12 = Some(val);
        },
        13 => {
          let val = i_prot.read_bool()?;
          f_13 = Some(val);
        },
        14 => {
          let val = i_prot.read_bool()?;
          f_14 = Some(val);
        },
        15 => {
          let val = i_prot.read_bool()?;
          f_15 = Some(val);
        },
        16 => {
          let val = i_prot.read_bool()?;
          f_16 = Some(val);
        },
        17 => {
          let val = i_prot.read_bool()?;
          f_17 = Some(val);
        },
        18 => {
          let val = FrictionlessFollowerType::read_from_in_protocol(i_prot)?;
          f_18 = Some(val);
        },
        19 => {
          let val = i_prot.read_i64()?;
          f_19 = Some(val);
        },
        20 => {
          let val = i_prot.read_bool()?;
          f_20 = Some(val);
        },
        21 => {
          let val = i_prot.read_i64()?;
          f_21 = Some(val);
        },
        22 => {
          let val = i_prot.read_i64()?;
          f_22 = Some(val);
        },
        23 => {
          let val = ForceLoginChallenge::read_from_in_protocol(i_prot)?;
          f_23 = Some(val);
        },
        25 => {
          let val = SuspensionDetails::read_from_in_protocol(i_prot)?;
          f_25 = Some(val);
        },
        26 => {
          let val = AccessPolicy::read_from_in_protocol(i_prot)?;
          f_26 = Some(val);
        },
        27 => {
          let val = i_prot.read_bool()?;
          f_27 = Some(val);
        },
        28 => {
          let val = SignupCategory::read_from_in_protocol(i_prot)?;
          f_28 = Some(val);
        },
        29 => {
          let val = SignupTrustLevel::read_from_in_protocol(i_prot)?;
          f_29 = Some(val);
        },
        30 => {
          let val = i_prot.read_string()?;
          f_30 = Some(val);
        },
        31 => {
          let val = i_prot.read_bool()?;
          f_31 = Some(val);
        },
        32 => {
          let val = i_prot.read_bool()?;
          f_32 = Some(val);
        },
        33 => {
          let val = i_prot.read_i64()?;
          f_33 = Some(val);
        },
        34 => {
          let val = UniversalQualityFiltering::read_from_in_protocol(i_prot)?;
          f_34 = Some(val);
        },
        35 => {
          let val = i_prot.read_i64()?;
          f_35 = Some(val);
        },
        36 => {
          let val = i_prot.read_bool()?;
          f_36 = Some(val);
        },
        37 => {
          let val = i_prot.read_i64()?;
          f_37 = Some(val);
        },
        38 => {
          let val = i_prot.read_bool()?;
          f_38 = Some(val);
        },
        39 => {
          let val = i_prot.read_i64()?;
          f_39 = Some(val);
        },
        40 => {
          let val = i_prot.read_bool()?;
          f_40 = Some(val);
        },
        41 => {
          let val = Consents::read_from_in_protocol(i_prot)?;
          f_41 = Some(val);
        },
        42 => {
          let val = DeactivationTimespan::read_from_in_protocol(i_prot)?;
          f_42 = Some(val);
        },
        43 => {
          let val = CompromisedState::read_from_in_protocol(i_prot)?;
          f_43 = Some(val);
        },
        44 => {
          let val = i_prot.read_i64()?;
          f_44 = Some(val);
        },
        45 => {
          let val = i_prot.read_bool()?;
          f_45 = Some(val);
        },
        46 => {
          let val = VerifiedType::read_from_in_protocol(i_prot)?;
          f_46 = Some(val);
        },
        47 => {
          let val = i_prot.read_i64()?;
          f_47 = Some(val);
        },
        49 => {
          let val = i_prot.read_bool()?;
          f_49 = Some(val);
        },
        50 => {
          let val = i_prot.read_i64()?;
          f_50 = Some(val);
        },
        51 => {
          let val = BlueCheckmarkHiddenReason::read_from_in_protocol(i_prot)?;
          f_51 = Some(val);
        },
        52 => {
          let val = IdentityVerification::read_from_in_protocol(i_prot)?;
          f_52 = Some(val);
        },
        53 => {
          let val = VerifiedOrganizationDetails::read_from_in_protocol(i_prot)?;
          f_53 = Some(val);
        },
        54 => {
          let val = i_prot.read_bool()?;
          f_54 = Some(val);
        },
        55 => {
          let val = i_prot.read_bool()?;
          f_55 = Some(val);
        },
        56 => {
          let val = SubscriptionLevel::read_from_in_protocol(i_prot)?;
          f_56 = Some(val);
        },
        57 => {
          let val = SignupCreationSource::read_from_in_protocol(i_prot)?;
          f_57 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Safety {
      is_protected: f_2,
      verified: f_3,
      deactivated: f_4,
      suspended: f_5,
      restricted: f_6,
      nsfw_user: f_7,
      nsfw_admin: f_8,
      has_takedown: f_9,
      has_role: f_10,
      frictionless_follower_state: f_11,
      erased: f_12,
      force_password_reset: f_13,
      protect_password_reset: f_14,
      has_saved_searches: f_15,
      is_lifeline_institution: f_16,
      translation_enabled: f_17,
      frictionless_follower_type: f_18,
      admin_password_last_reset_msec: f_19,
      needs_phone_verification: f_20,
      deactivated_at_msec: f_21,
      manhattan_updated_at_msec: f_22,
      force_login_challenge: f_23,
      suspension_details: f_25,
      access_policy: f_26,
      has_labels: f_27,
      signup_category: f_28,
      signup_trust_level: f_29,
      signup_country_code: f_30,
      require_password_login: f_31,
      has_extended_profile: f_32,
      last_annotated_at_msec: f_33,
      universal_quality_filtering: f_34,
      access_policy_expiry_msec: f_35,
      should_offboard: f_36,
      should_offboard_updated_at_msec: f_37,
      offboarded: f_38,
      offboarded_updated_at_msec: f_39,
      require_some_consent: f_40,
      consents: f_41,
      deactivation_timespan: f_42,
      compromised_state: f_43,
      compromised_at_msec: f_44,
      preerased: f_45,
      verified_type: f_46,
      external_user_update_msec: f_47,
      is_blue_verified: f_49,
      blue_verified_expiration_msec: f_50,
      blue_checkmark_hidden_reason: f_51,
      identity_verification: f_52,
      verified_organization_details: f_53,
      skip_rate_limit: f_54,
      community_verified: f_55,
      subscription_level: f_56,
      signup_creation_source: f_57,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Safety");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.is_protected {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_protected", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.verified {
      o_prot.write_field_begin(&TFieldIdentifier::new("verified", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.deactivated {
      o_prot.write_field_begin(&TFieldIdentifier::new("deactivated", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.suspended {
      o_prot.write_field_begin(&TFieldIdentifier::new("suspended", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.restricted {
      o_prot.write_field_begin(&TFieldIdentifier::new("restricted", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.nsfw_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("nsfw_user", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.nsfw_admin {
      o_prot.write_field_begin(&TFieldIdentifier::new("nsfw_admin", TType::Bool, 8))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_takedown {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_takedown", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_role {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_role", TType::Bool, 10))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.frictionless_follower_state {
      o_prot.write_field_begin(&TFieldIdentifier::new("frictionless_follower_state", TType::I32, 11))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.erased {
      o_prot.write_field_begin(&TFieldIdentifier::new("erased", TType::Bool, 12))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.force_password_reset {
      o_prot.write_field_begin(&TFieldIdentifier::new("force_password_reset", TType::Bool, 13))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.protect_password_reset {
      o_prot.write_field_begin(&TFieldIdentifier::new("protect_password_reset", TType::Bool, 14))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_saved_searches {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_saved_searches", TType::Bool, 15))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_lifeline_institution {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_lifeline_institution", TType::Bool, 16))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.translation_enabled {
      o_prot.write_field_begin(&TFieldIdentifier::new("translation_enabled", TType::Bool, 17))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.frictionless_follower_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("frictionless_follower_type", TType::I32, 18))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.admin_password_last_reset_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("admin_password_last_reset_msec", TType::I64, 19))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.needs_phone_verification {
      o_prot.write_field_begin(&TFieldIdentifier::new("needs_phone_verification", TType::Bool, 20))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.deactivated_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("deactivated_at_msec", TType::I64, 21))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.manhattan_updated_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("manhattan_updated_at_msec", TType::I64, 22))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.force_login_challenge {
      o_prot.write_field_begin(&TFieldIdentifier::new("force_login_challenge", TType::Struct, 23))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.suspension_details {
      o_prot.write_field_begin(&TFieldIdentifier::new("suspension_details", TType::Struct, 25))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.access_policy {
      o_prot.write_field_begin(&TFieldIdentifier::new("access_policy", TType::I32, 26))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_labels {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_labels", TType::Bool, 27))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.signup_category {
      o_prot.write_field_begin(&TFieldIdentifier::new("signup_category", TType::I32, 28))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.signup_trust_level {
      o_prot.write_field_begin(&TFieldIdentifier::new("signup_trust_level", TType::I32, 29))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.signup_country_code {
      o_prot.write_field_begin(&TFieldIdentifier::new("signup_country_code", TType::String, 30))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.require_password_login {
      o_prot.write_field_begin(&TFieldIdentifier::new("require_password_login", TType::Bool, 31))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_extended_profile {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_extended_profile", TType::Bool, 32))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.last_annotated_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("last_annotated_at_msec", TType::I64, 33))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.universal_quality_filtering {
      o_prot.write_field_begin(&TFieldIdentifier::new("universal_quality_filtering", TType::I32, 34))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.access_policy_expiry_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("access_policy_expiry_msec", TType::I64, 35))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.should_offboard {
      o_prot.write_field_begin(&TFieldIdentifier::new("should_offboard", TType::Bool, 36))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.should_offboard_updated_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("should_offboard_updated_at_msec", TType::I64, 37))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.offboarded {
      o_prot.write_field_begin(&TFieldIdentifier::new("offboarded", TType::Bool, 38))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.offboarded_updated_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("offboarded_updated_at_msec", TType::I64, 39))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.require_some_consent {
      o_prot.write_field_begin(&TFieldIdentifier::new("require_some_consent", TType::Bool, 40))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.consents {
      o_prot.write_field_begin(&TFieldIdentifier::new("consents", TType::Struct, 41))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.deactivation_timespan {
      o_prot.write_field_begin(&TFieldIdentifier::new("deactivation_timespan", TType::I32, 42))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.compromised_state {
      o_prot.write_field_begin(&TFieldIdentifier::new("compromised_state", TType::I32, 43))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.compromised_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("compromised_at_msec", TType::I64, 44))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.preerased {
      o_prot.write_field_begin(&TFieldIdentifier::new("preerased", TType::Bool, 45))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.verified_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("verified_type", TType::I32, 46))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.external_user_update_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("external_user_update_msec", TType::I64, 47))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.is_blue_verified {
      o_prot.write_field_begin(&TFieldIdentifier::new("is_blue_verified", TType::Bool, 49))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.blue_verified_expiration_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("blue_verified_expiration_msec", TType::I64, 50))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.blue_checkmark_hidden_reason {
      o_prot.write_field_begin(&TFieldIdentifier::new("blue_checkmark_hidden_reason", TType::I32, 51))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.identity_verification {
      o_prot.write_field_begin(&TFieldIdentifier::new("identity_verification", TType::Struct, 52))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.verified_organization_details {
      o_prot.write_field_begin(&TFieldIdentifier::new("verified_organization_details", TType::Struct, 53))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.skip_rate_limit {
      o_prot.write_field_begin(&TFieldIdentifier::new("skip_rate_limit", TType::Bool, 54))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.community_verified {
      o_prot.write_field_begin(&TFieldIdentifier::new("community_verified", TType::Bool, 55))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.subscription_level {
      o_prot.write_field_begin(&TFieldIdentifier::new("subscription_level", TType::I32, 56))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.signup_creation_source {
      o_prot.write_field_begin(&TFieldIdentifier::new("signup_creation_source", TType::I32, 57))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Counts {
  pub followers: Option<i64>,
  pub following: Option<i64>,
  pub tweets: Option<i64>,
  pub favorites: Option<i64>,
  pub listed: Option<i64>,
  pub blockers: Option<i64>,
  pub owned_lists: Option<i64>,
  pub subscribed_lists: Option<i64>,
  pub media_tweets: Option<i64>,
  pub owned_public_lists: Option<i64>,
  pub frictionless_followers: Option<i64>,
  pub normal_followers: Option<i64>,
  pub blocking: Option<i64>,
  pub muters: Option<i64>,
  pub muting: Option<i64>,
}

impl Counts {
  pub fn new<F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16>(followers: F2, following: F3, tweets: F4, favorites: F5, listed: F6, blockers: F7, owned_lists: F8, subscribed_lists: F9, media_tweets: F10, owned_public_lists: F11, frictionless_followers: F12, normal_followers: F13, blocking: F14, muters: F15, muting: F16) -> Counts where F2: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<i64>>, F6: Into<Option<i64>>, F7: Into<Option<i64>>, F8: Into<Option<i64>>, F9: Into<Option<i64>>, F10: Into<Option<i64>>, F11: Into<Option<i64>>, F12: Into<Option<i64>>, F13: Into<Option<i64>>, F14: Into<Option<i64>>, F15: Into<Option<i64>>, F16: Into<Option<i64>> {
    Counts {
      followers: followers.into(),
      following: following.into(),
      tweets: tweets.into(),
      favorites: favorites.into(),
      listed: listed.into(),
      blockers: blockers.into(),
      owned_lists: owned_lists.into(),
      subscribed_lists: subscribed_lists.into(),
      media_tweets: media_tweets.into(),
      owned_public_lists: owned_public_lists.into(),
      frictionless_followers: frictionless_followers.into(),
      normal_followers: normal_followers.into(),
      blocking: blocking.into(),
      muters: muters.into(),
      muting: muting.into(),
    }
  }
}

impl TSerializable for Counts {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Counts> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<i64> = Some(0);
    let mut f_6: Option<i64> = Some(0);
    let mut f_7: Option<i64> = None;
    let mut f_8: Option<i64> = None;
    let mut f_9: Option<i64> = None;
    let mut f_10: Option<i64> = None;
    let mut f_11: Option<i64> = None;
    let mut f_12: Option<i64> = None;
    let mut f_13: Option<i64> = None;
    let mut f_14: Option<i64> = None;
    let mut f_15: Option<i64> = None;
    let mut f_16: Option<i64> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
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
        8 => {
          let val = i_prot.read_i64()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_i64()?;
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
          let val = i_prot.read_i64()?;
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
    let ret = Counts {
      followers: f_2,
      following: f_3,
      tweets: f_4,
      favorites: f_5,
      listed: f_6,
      blockers: f_7,
      owned_lists: f_8,
      subscribed_lists: f_9,
      media_tweets: f_10,
      owned_public_lists: f_11,
      frictionless_followers: f_12,
      normal_followers: f_13,
      blocking: f_14,
      muters: f_15,
      muting: f_16,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Counts");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.followers {
      o_prot.write_field_begin(&TFieldIdentifier::new("followers", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.following {
      o_prot.write_field_begin(&TFieldIdentifier::new("following", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.tweets {
      o_prot.write_field_begin(&TFieldIdentifier::new("tweets", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.favorites {
      o_prot.write_field_begin(&TFieldIdentifier::new("favorites", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.listed {
      o_prot.write_field_begin(&TFieldIdentifier::new("listed", TType::I64, 6))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.blockers {
      o_prot.write_field_begin(&TFieldIdentifier::new("blockers", TType::I64, 7))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.owned_lists {
      o_prot.write_field_begin(&TFieldIdentifier::new("owned_lists", TType::I64, 8))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.subscribed_lists {
      o_prot.write_field_begin(&TFieldIdentifier::new("subscribed_lists", TType::I64, 9))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.media_tweets {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_tweets", TType::I64, 10))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.owned_public_lists {
      o_prot.write_field_begin(&TFieldIdentifier::new("owned_public_lists", TType::I64, 11))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.frictionless_followers {
      o_prot.write_field_begin(&TFieldIdentifier::new("frictionless_followers", TType::I64, 12))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.normal_followers {
      o_prot.write_field_begin(&TFieldIdentifier::new("normal_followers", TType::I64, 13))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.blocking {
      o_prot.write_field_begin(&TFieldIdentifier::new("blocking", TType::I64, 14))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.muters {
      o_prot.write_field_begin(&TFieldIdentifier::new("muters", TType::I64, 15))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.muting {
      o_prot.write_field_begin(&TFieldIdentifier::new("muting", TType::I64, 16))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Roles {
  pub roles: Option<BTreeSet<String>>,
  pub rights: Option<BTreeSet<String>>,
  pub features: Option<BTreeSet<String>>,
}

impl Roles {
  pub fn new<F1, F2, F3>(roles: F1, rights: F2, features: F3) -> Roles where F1: Into<Option<BTreeSet<String>>>, F2: Into<Option<BTreeSet<String>>>, F3: Into<Option<BTreeSet<String>>> {
    Roles {
      roles: roles.into(),
      rights: rights.into(),
      features: features.into(),
    }
  }
}

impl TSerializable for Roles {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Roles> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<BTreeSet<String>> = Some(BTreeSet::new());
    let mut f_2: Option<BTreeSet<String>> = Some(BTreeSet::new());
    let mut f_3: Option<BTreeSet<String>> = Some(BTreeSet::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<String> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_51 = i_prot.read_string()?;
            val.insert(set_elem_51);
          }
          i_prot.read_set_end()?;
          f_1 = Some(val);
        },
        2 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<String> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_52 = i_prot.read_string()?;
            val.insert(set_elem_52);
          }
          i_prot.read_set_end()?;
          f_2 = Some(val);
        },
        3 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<String> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_53 = i_prot.read_string()?;
            val.insert(set_elem_53);
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
    let ret = Roles {
      roles: f_1,
      rights: f_2,
      features: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Roles");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.roles {
      o_prot.write_field_begin(&TFieldIdentifier::new("roles", TType::Set, 1))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::String, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_string(e)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.rights {
      o_prot.write_field_begin(&TFieldIdentifier::new("rights", TType::Set, 2))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::String, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_string(e)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.features {
      o_prot.write_field_begin(&TFieldIdentifier::new("features", TType::Set, 3))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::String, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_string(e)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct View {
  pub following: Option<bool>,
  pub followed_by: Option<bool>,
  pub follow_request_sent: Option<bool>,
  pub device_following: Option<bool>,
  pub blocking: Option<bool>,
  pub blocked_by: Option<bool>,
  pub no_retweets_from: Option<bool>,
  pub lifeline_following: Option<bool>,
  pub lifeline_followed_by: Option<bool>,
}

impl View {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9>(following: F1, followed_by: F2, follow_request_sent: F3, device_following: F4, blocking: F5, blocked_by: F6, no_retweets_from: F7, lifeline_following: F8, lifeline_followed_by: F9) -> View where F1: Into<Option<bool>>, F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<bool>>, F5: Into<Option<bool>>, F6: Into<Option<bool>>, F7: Into<Option<bool>>, F8: Into<Option<bool>>, F9: Into<Option<bool>> {
    View {
      following: following.into(),
      followed_by: followed_by.into(),
      follow_request_sent: follow_request_sent.into(),
      device_following: device_following.into(),
      blocking: blocking.into(),
      blocked_by: blocked_by.into(),
      no_retweets_from: no_retweets_from.into(),
      lifeline_following: lifeline_following.into(),
      lifeline_followed_by: lifeline_followed_by.into(),
    }
  }
}

impl TSerializable for View {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<View> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = Some(false);
    let mut f_2: Option<bool> = Some(false);
    let mut f_3: Option<bool> = Some(false);
    let mut f_4: Option<bool> = Some(false);
    let mut f_5: Option<bool> = Some(false);
    let mut f_6: Option<bool> = Some(false);
    let mut f_7: Option<bool> = Some(false);
    let mut f_8: Option<bool> = None;
    let mut f_9: Option<bool> = None;
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
        2 => {
          let val = i_prot.read_bool()?;
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
        8 => {
          let val = i_prot.read_bool()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = View {
      following: f_1,
      followed_by: f_2,
      follow_request_sent: f_3,
      device_following: f_4,
      blocking: f_5,
      blocked_by: f_6,
      no_retweets_from: f_7,
      lifeline_following: f_8,
      lifeline_followed_by: f_9,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("View");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.following {
      o_prot.write_field_begin(&TFieldIdentifier::new("following", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.followed_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("followed_by", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.follow_request_sent {
      o_prot.write_field_begin(&TFieldIdentifier::new("follow_request_sent", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.device_following {
      o_prot.write_field_begin(&TFieldIdentifier::new("device_following", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.blocking {
      o_prot.write_field_begin(&TFieldIdentifier::new("blocking", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.blocked_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("blocked_by", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.no_retweets_from {
      o_prot.write_field_begin(&TFieldIdentifier::new("no_retweets_from", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.lifeline_following {
      o_prot.write_field_begin(&TFieldIdentifier::new("lifeline_following", TType::Bool, 8))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.lifeline_followed_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("lifeline_followed_by", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediaView {
  pub can_media_tag: Option<bool>,
}

impl MediaView {
  pub fn new<F1>(can_media_tag: F1) -> MediaView where F1: Into<Option<bool>> {
    MediaView {
      can_media_tag: can_media_tag.into(),
    }
  }
}

impl TSerializable for MediaView {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MediaView> {
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
    let ret = MediaView {
      can_media_tag: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MediaView");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.can_media_tag {
      o_prot.write_field_begin(&TFieldIdentifier::new("can_media_tag", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DirectMessageView {
  pub can_send_dm: Option<bool>,
  pub can_send_secret_dm: Option<bool>,
}

impl DirectMessageView {
  pub fn new<F1, F2>(can_send_dm: F1, can_send_secret_dm: F2) -> DirectMessageView where F1: Into<Option<bool>>, F2: Into<Option<bool>> {
    DirectMessageView {
      can_send_dm: can_send_dm.into(),
      can_send_secret_dm: can_send_secret_dm.into(),
    }
  }
}

impl TSerializable for DirectMessageView {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<DirectMessageView> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = Some(false);
    let mut f_2: Option<bool> = None;
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
    let ret = DirectMessageView {
      can_send_dm: f_1,
      can_send_secret_dm: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("DirectMessageView");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.can_send_dm {
      o_prot.write_field_begin(&TFieldIdentifier::new("can_send_dm", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.can_send_secret_dm {
      o_prot.write_field_begin(&TFieldIdentifier::new("can_send_secret_dm", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Perspective {
  pub following: Option<bool>,
  pub followed_by: Option<bool>,
  pub follow_request_sent: Option<bool>,
  pub device_following: Option<bool>,
  pub blocking: Option<bool>,
  pub blocked_by: Option<bool>,
  pub no_retweets_from: Option<bool>,
  pub lifeline_following: Option<bool>,
  pub lifeline_followed_by: Option<bool>,
  pub muting: Option<bool>,
  pub muted_by: Option<bool>,
  pub follow_request_received: Option<bool>,
  pub dm_trusted_by: Option<bool>,
  pub dm_blocking: Option<bool>,
  pub dm_blocked_by: Option<bool>,
  pub deprecated_1: Option<bool>,
  pub deprecated_2: Option<bool>,
  pub live_following: Option<bool>,
  pub live_followed_by: Option<bool>,
  pub keyword_muting: Option<bool>,
  pub advanced_filtering: Option<bool>,
  pub frictionless_following: Option<bool>,
  pub frictionless_device_following: Option<bool>,
  pub subscribed_by: Option<bool>,
  pub subscribing: Option<bool>,
}

impl Perspective {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24, F25>(following: F1, followed_by: F2, follow_request_sent: F3, device_following: F4, blocking: F5, blocked_by: F6, no_retweets_from: F7, lifeline_following: F8, lifeline_followed_by: F9, muting: F10, muted_by: F11, follow_request_received: F12, dm_trusted_by: F13, dm_blocking: F14, dm_blocked_by: F15, deprecated_1: F16, deprecated_2: F17, live_following: F18, live_followed_by: F19, keyword_muting: F20, advanced_filtering: F21, frictionless_following: F22, frictionless_device_following: F23, subscribed_by: F24, subscribing: F25) -> Perspective where F1: Into<Option<bool>>, F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<bool>>, F5: Into<Option<bool>>, F6: Into<Option<bool>>, F7: Into<Option<bool>>, F8: Into<Option<bool>>, F9: Into<Option<bool>>, F10: Into<Option<bool>>, F11: Into<Option<bool>>, F12: Into<Option<bool>>, F13: Into<Option<bool>>, F14: Into<Option<bool>>, F15: Into<Option<bool>>, F16: Into<Option<bool>>, F17: Into<Option<bool>>, F18: Into<Option<bool>>, F19: Into<Option<bool>>, F20: Into<Option<bool>>, F21: Into<Option<bool>>, F22: Into<Option<bool>>, F23: Into<Option<bool>>, F24: Into<Option<bool>>, F25: Into<Option<bool>> {
    Perspective {
      following: following.into(),
      followed_by: followed_by.into(),
      follow_request_sent: follow_request_sent.into(),
      device_following: device_following.into(),
      blocking: blocking.into(),
      blocked_by: blocked_by.into(),
      no_retweets_from: no_retweets_from.into(),
      lifeline_following: lifeline_following.into(),
      lifeline_followed_by: lifeline_followed_by.into(),
      muting: muting.into(),
      muted_by: muted_by.into(),
      follow_request_received: follow_request_received.into(),
      dm_trusted_by: dm_trusted_by.into(),
      dm_blocking: dm_blocking.into(),
      dm_blocked_by: dm_blocked_by.into(),
      deprecated_1: deprecated_1.into(),
      deprecated_2: deprecated_2.into(),
      live_following: live_following.into(),
      live_followed_by: live_followed_by.into(),
      keyword_muting: keyword_muting.into(),
      advanced_filtering: advanced_filtering.into(),
      frictionless_following: frictionless_following.into(),
      frictionless_device_following: frictionless_device_following.into(),
      subscribed_by: subscribed_by.into(),
      subscribing: subscribing.into(),
    }
  }
}

impl TSerializable for Perspective {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Perspective> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = None;
    let mut f_2: Option<bool> = None;
    let mut f_3: Option<bool> = None;
    let mut f_4: Option<bool> = None;
    let mut f_5: Option<bool> = None;
    let mut f_6: Option<bool> = None;
    let mut f_7: Option<bool> = None;
    let mut f_8: Option<bool> = None;
    let mut f_9: Option<bool> = None;
    let mut f_10: Option<bool> = None;
    let mut f_11: Option<bool> = None;
    let mut f_12: Option<bool> = None;
    let mut f_13: Option<bool> = None;
    let mut f_14: Option<bool> = None;
    let mut f_15: Option<bool> = None;
    let mut f_16: Option<bool> = None;
    let mut f_17: Option<bool> = None;
    let mut f_18: Option<bool> = None;
    let mut f_19: Option<bool> = None;
    let mut f_20: Option<bool> = None;
    let mut f_21: Option<bool> = None;
    let mut f_22: Option<bool> = None;
    let mut f_23: Option<bool> = None;
    let mut f_24: Option<bool> = None;
    let mut f_25: Option<bool> = None;
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
        2 => {
          let val = i_prot.read_bool()?;
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
        8 => {
          let val = i_prot.read_bool()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_bool()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_bool()?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_bool()?;
          f_12 = Some(val);
        },
        13 => {
          let val = i_prot.read_bool()?;
          f_13 = Some(val);
        },
        14 => {
          let val = i_prot.read_bool()?;
          f_14 = Some(val);
        },
        15 => {
          let val = i_prot.read_bool()?;
          f_15 = Some(val);
        },
        16 => {
          let val = i_prot.read_bool()?;
          f_16 = Some(val);
        },
        17 => {
          let val = i_prot.read_bool()?;
          f_17 = Some(val);
        },
        18 => {
          let val = i_prot.read_bool()?;
          f_18 = Some(val);
        },
        19 => {
          let val = i_prot.read_bool()?;
          f_19 = Some(val);
        },
        20 => {
          let val = i_prot.read_bool()?;
          f_20 = Some(val);
        },
        21 => {
          let val = i_prot.read_bool()?;
          f_21 = Some(val);
        },
        22 => {
          let val = i_prot.read_bool()?;
          f_22 = Some(val);
        },
        23 => {
          let val = i_prot.read_bool()?;
          f_23 = Some(val);
        },
        24 => {
          let val = i_prot.read_bool()?;
          f_24 = Some(val);
        },
        25 => {
          let val = i_prot.read_bool()?;
          f_25 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Perspective {
      following: f_1,
      followed_by: f_2,
      follow_request_sent: f_3,
      device_following: f_4,
      blocking: f_5,
      blocked_by: f_6,
      no_retweets_from: f_7,
      lifeline_following: f_8,
      lifeline_followed_by: f_9,
      muting: f_10,
      muted_by: f_11,
      follow_request_received: f_12,
      dm_trusted_by: f_13,
      dm_blocking: f_14,
      dm_blocked_by: f_15,
      deprecated_1: f_16,
      deprecated_2: f_17,
      live_following: f_18,
      live_followed_by: f_19,
      keyword_muting: f_20,
      advanced_filtering: f_21,
      frictionless_following: f_22,
      frictionless_device_following: f_23,
      subscribed_by: f_24,
      subscribing: f_25,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Perspective");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.following {
      o_prot.write_field_begin(&TFieldIdentifier::new("following", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.followed_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("followed_by", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.follow_request_sent {
      o_prot.write_field_begin(&TFieldIdentifier::new("follow_request_sent", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.device_following {
      o_prot.write_field_begin(&TFieldIdentifier::new("device_following", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.blocking {
      o_prot.write_field_begin(&TFieldIdentifier::new("blocking", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.blocked_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("blocked_by", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.no_retweets_from {
      o_prot.write_field_begin(&TFieldIdentifier::new("no_retweets_from", TType::Bool, 7))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.lifeline_following {
      o_prot.write_field_begin(&TFieldIdentifier::new("lifeline_following", TType::Bool, 8))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.lifeline_followed_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("lifeline_followed_by", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.muting {
      o_prot.write_field_begin(&TFieldIdentifier::new("muting", TType::Bool, 10))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.muted_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("muted_by", TType::Bool, 11))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.follow_request_received {
      o_prot.write_field_begin(&TFieldIdentifier::new("follow_request_received", TType::Bool, 12))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.dm_trusted_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("dm_trusted_by", TType::Bool, 13))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.dm_blocking {
      o_prot.write_field_begin(&TFieldIdentifier::new("dm_blocking", TType::Bool, 14))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.dm_blocked_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("dm_blocked_by", TType::Bool, 15))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.deprecated_1 {
      o_prot.write_field_begin(&TFieldIdentifier::new("deprecated_1", TType::Bool, 16))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.deprecated_2 {
      o_prot.write_field_begin(&TFieldIdentifier::new("deprecated_2", TType::Bool, 17))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.live_following {
      o_prot.write_field_begin(&TFieldIdentifier::new("live_following", TType::Bool, 18))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.live_followed_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("live_followed_by", TType::Bool, 19))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.keyword_muting {
      o_prot.write_field_begin(&TFieldIdentifier::new("keyword_muting", TType::Bool, 20))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.advanced_filtering {
      o_prot.write_field_begin(&TFieldIdentifier::new("advanced_filtering", TType::Bool, 21))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.frictionless_following {
      o_prot.write_field_begin(&TFieldIdentifier::new("frictionless_following", TType::Bool, 22))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.frictionless_device_following {
      o_prot.write_field_begin(&TFieldIdentifier::new("frictionless_device_following", TType::Bool, 23))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.subscribed_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("subscribed_by", TType::Bool, 24))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.subscribing {
      o_prot.write_field_begin(&TFieldIdentifier::new("subscribing", TType::Bool, 25))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Takedown {
  pub id: Option<i64>,
}

impl Takedown {
  pub fn new<F1>(id: F1) -> Takedown where F1: Into<Option<i64>> {
    Takedown {
      id: id.into(),
    }
  }
}

impl TSerializable for Takedown {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Takedown> {
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
    let ret = Takedown {
      id: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Takedown");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Takedowns {
  pub country_codes: Option<BTreeSet<String>>,
  pub takedown_country_reasons: Option<Vec<Takedown>>,
}

impl Takedowns {
  pub fn new<F1, F2>(country_codes: F1, takedown_country_reasons: F2) -> Takedowns where F1: Into<Option<BTreeSet<String>>>, F2: Into<Option<Vec<Takedown>>> {
    Takedowns {
      country_codes: country_codes.into(),
      takedown_country_reasons: takedown_country_reasons.into(),
    }
  }
}

impl TSerializable for Takedowns {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Takedowns> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<BTreeSet<String>> = Some(BTreeSet::new());
    let mut f_2: Option<Vec<Takedown>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<String> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_54 = i_prot.read_string()?;
            val.insert(set_elem_54);
          }
          i_prot.read_set_end()?;
          f_1 = Some(val);
        },
        2 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<Takedown> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_55 = Takedown::read_from_in_protocol(i_prot)?;
            val.push(list_elem_55);
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
    let ret = Takedowns {
      country_codes: f_1,
      takedown_country_reasons: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Takedowns");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.country_codes {
      o_prot.write_field_begin(&TFieldIdentifier::new("country_codes", TType::Set, 1))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::String, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_string(e)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.takedown_country_reasons {
      o_prot.write_field_begin(&TFieldIdentifier::new("takedown_country_reasons", TType::List, 2))?;
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
pub struct SavedSearch {
  pub id: Option<i64>,
  pub query: Option<String>,
  pub name: Option<String>,
  pub created_at_msec: Option<i64>,
}

impl SavedSearch {
  pub fn new<F1, F2, F3, F4>(id: F1, query: F2, name: F3, created_at_msec: F4) -> SavedSearch where F1: Into<Option<i64>>, F2: Into<Option<String>>, F3: Into<Option<String>>, F4: Into<Option<i64>> {
    SavedSearch {
      id: id.into(),
      query: query.into(),
      name: name.into(),
      created_at_msec: created_at_msec.into(),
    }
  }
}

impl TSerializable for SavedSearch {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SavedSearch> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<String> = None;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = SavedSearch {
      id: f_1,
      query: f_2,
      name: f_3,
      created_at_msec: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SavedSearch");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.query {
      o_prot.write_field_begin(&TFieldIdentifier::new("query", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_msec", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SavedSearches {
  pub searches: Option<Vec<SavedSearch>>,
}

impl SavedSearches {
  pub fn new<F1>(searches: F1) -> SavedSearches where F1: Into<Option<Vec<SavedSearch>>> {
    SavedSearches {
      searches: searches.into(),
    }
  }
}

impl TSerializable for SavedSearches {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<SavedSearches> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<SavedSearch>> = Some(Vec::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<SavedSearch> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_56 = SavedSearch::read_from_in_protocol(i_prot)?;
            val.push(list_elem_56);
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
    let ret = SavedSearches {
      searches: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("SavedSearches");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.searches {
      o_prot.write_field_begin(&TFieldIdentifier::new("searches", TType::List, 1))?;
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
pub struct Contribution {
  pub contributors: Option<BTreeMap<i64, ContributorAccessLevel>>,
  pub contributees: Option<BTreeMap<i64, ContributorAccessLevel>>,
}

impl Contribution {
  pub fn new<F2, F3>(contributors: F2, contributees: F3) -> Contribution where F2: Into<Option<BTreeMap<i64, ContributorAccessLevel>>>, F3: Into<Option<BTreeMap<i64, ContributorAccessLevel>>> {
    Contribution {
      contributors: contributors.into(),
      contributees: contributees.into(),
    }
  }
}

impl TSerializable for Contribution {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Contribution> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<BTreeMap<i64, ContributorAccessLevel>> = Some(BTreeMap::new());
    let mut f_3: Option<BTreeMap<i64, ContributorAccessLevel>> = Some(BTreeMap::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<i64, ContributorAccessLevel> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_57 = i_prot.read_i64()?;
            let map_val_58 = ContributorAccessLevel::read_from_in_protocol(i_prot)?;
            val.insert(map_key_57, map_val_58);
          }
          i_prot.read_map_end()?;
          f_2 = Some(val);
        },
        3 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<i64, ContributorAccessLevel> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_59 = i_prot.read_i64()?;
            let map_val_60 = ContributorAccessLevel::read_from_in_protocol(i_prot)?;
            val.insert(map_key_59, map_val_60);
          }
          i_prot.read_map_end()?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Contribution {
      contributors: f_2,
      contributees: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Contribution");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.contributors {
      o_prot.write_field_begin(&TFieldIdentifier::new("contributors", TType::Map, 2))?;
      o_prot.write_map_begin(&TMapIdentifier::new(TType::I64, TType::I32, fld_var.len() as i32))?;
      for (k, v) in fld_var {
        o_prot.write_i64(*k)?;
        v.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_map_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.contributees {
      o_prot.write_field_begin(&TFieldIdentifier::new("contributees", TType::Map, 3))?;
      o_prot.write_map_begin(&TMapIdentifier::new(TType::I64, TType::I32, fld_var.len() as i32))?;
      for (k, v) in fld_var {
        o_prot.write_i64(*k)?;
        v.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_map_end()?;
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
}

impl UrlEntity {
  pub fn new<F1, F2, F3, F4, F5>(from_index: F1, to_index: F2, url: F3, expanded: F4, display: F5) -> UrlEntity where F1: Into<Option<i16>>, F2: Into<Option<i16>>, F3: Into<Option<String>>, F4: Into<Option<String>>, F5: Into<Option<String>> {
    UrlEntity {
      from_index: from_index.into(),
      to_index: to_index.into(),
      url: url.into(),
      expanded: expanded.into(),
      display: display.into(),
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
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UrlEntity");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.from_index {
      o_prot.write_field_begin(&TFieldIdentifier::new("fromIndex", TType::I16, 1))?;
      o_prot.write_i16(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.to_index {
      o_prot.write_field_begin(&TFieldIdentifier::new("toIndex", TType::I16, 2))?;
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
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UrlEntities {
  pub description_entities: Option<Vec<UrlEntity>>,
  pub url_entities: Option<Vec<UrlEntity>>,
}

impl UrlEntities {
  pub fn new<F2, F3>(description_entities: F2, url_entities: F3) -> UrlEntities where F2: Into<Option<Vec<UrlEntity>>>, F3: Into<Option<Vec<UrlEntity>>> {
    UrlEntities {
      description_entities: description_entities.into(),
      url_entities: url_entities.into(),
    }
  }
}

impl TSerializable for UrlEntities {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<UrlEntities> {
    i_prot.read_struct_begin()?;
    let mut f_2: Option<Vec<UrlEntity>> = Some(Vec::new());
    let mut f_3: Option<Vec<UrlEntity>> = Some(Vec::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        2 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<UrlEntity> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_61 = UrlEntity::read_from_in_protocol(i_prot)?;
            val.push(list_elem_61);
          }
          i_prot.read_list_end()?;
          f_2 = Some(val);
        },
        3 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<UrlEntity> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_62 = UrlEntity::read_from_in_protocol(i_prot)?;
            val.push(list_elem_62);
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
    let ret = UrlEntities {
      description_entities: f_2,
      url_entities: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("UrlEntities");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.description_entities {
      o_prot.write_field_begin(&TFieldIdentifier::new("description_entities", TType::List, 2))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.url_entities {
      o_prot.write_field_begin(&TFieldIdentifier::new("url_entities", TType::List, 3))?;
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
pub struct MentionEntity {
      pub from_index: Option<i16>,
      pub to_index: Option<i16>,
    pub screen_name: Option<String>,
                pub user_id: Option<i64>,
        pub name: Option<String>,
}

impl MentionEntity {
  pub fn new<F1, F2, F3, F4, F5>(from_index: F1, to_index: F2, screen_name: F3, user_id: F4, name: F5) -> MentionEntity where F1: Into<Option<i16>>, F2: Into<Option<i16>>, F3: Into<Option<String>>, F4: Into<Option<i64>>, F5: Into<Option<String>> {
    MentionEntity {
      from_index: from_index.into(),
      to_index: to_index.into(),
      screen_name: screen_name.into(),
      user_id: user_id.into(),
      name: name.into(),
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
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MentionEntities {
  pub description_entities: Option<Vec<MentionEntity>>,
}

impl MentionEntities {
  pub fn new<F1>(description_entities: F1) -> MentionEntities where F1: Into<Option<Vec<MentionEntity>>> {
    MentionEntities {
      description_entities: description_entities.into(),
    }
  }
}

impl TSerializable for MentionEntities {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MentionEntities> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<MentionEntity>> = Some(Vec::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<MentionEntity> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_63 = MentionEntity::read_from_in_protocol(i_prot)?;
            val.push(list_elem_63);
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
    let ret = MentionEntities {
      description_entities: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MentionEntities");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.description_entities {
      o_prot.write_field_begin(&TFieldIdentifier::new("description_entities", TType::List, 1))?;
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
pub struct HashtagEntities {
  pub description_entities: Option<Vec<HashtagEntity>>,
}

impl HashtagEntities {
  pub fn new<F1>(description_entities: F1) -> HashtagEntities where F1: Into<Option<Vec<HashtagEntity>>> {
    HashtagEntities {
      description_entities: description_entities.into(),
    }
  }
}

impl TSerializable for HashtagEntities {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<HashtagEntities> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<HashtagEntity>> = Some(Vec::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<HashtagEntity> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_64 = HashtagEntity::read_from_in_protocol(i_prot)?;
            val.push(list_elem_64);
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
    let ret = HashtagEntities {
      description_entities: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("HashtagEntities");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.description_entities {
      o_prot.write_field_begin(&TFieldIdentifier::new("description_entities", TType::List, 1))?;
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
pub struct CashtagEntities {
  pub description_entities: Option<Vec<CashtagEntity>>,
}

impl CashtagEntities {
  pub fn new<F1>(description_entities: F1) -> CashtagEntities where F1: Into<Option<Vec<CashtagEntity>>> {
    CashtagEntities {
      description_entities: description_entities.into(),
    }
  }
}

impl TSerializable for CashtagEntities {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<CashtagEntities> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<CashtagEntity>> = Some(Vec::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<CashtagEntity> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_65 = CashtagEntity::read_from_in_protocol(i_prot)?;
            val.push(list_elem_65);
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
    let ret = CashtagEntities {
      description_entities: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("CashtagEntities");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.description_entities {
      o_prot.write_field_begin(&TFieldIdentifier::new("description_entities", TType::List, 1))?;
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
pub struct FacebookConnection {
  pub id: Option<i64>,
  pub type_: Option<FacebookConnectionType>,
  pub crossposting: Option<bool>,
  pub token: Option<String>,
  pub fb_id: Option<String>,
}

impl FacebookConnection {
  pub fn new<F1, F2, F3, F4, F5>(id: F1, type_: F2, crossposting: F3, token: F4, fb_id: F5) -> FacebookConnection where F1: Into<Option<i64>>, F2: Into<Option<FacebookConnectionType>>, F3: Into<Option<bool>>, F4: Into<Option<String>>, F5: Into<Option<String>> {
    FacebookConnection {
      id: id.into(),
      type_: type_.into(),
      crossposting: crossposting.into(),
      token: token.into(),
      fb_id: fb_id.into(),
    }
  }
}

impl TSerializable for FacebookConnection {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FacebookConnection> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<FacebookConnectionType> = None;
    let mut f_3: Option<bool> = Some(false);
    let mut f_4: Option<String> = None;
    let mut f_5: Option<String> = None;
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
          let val = FacebookConnectionType::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_bool()?;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = FacebookConnection {
      id: f_1,
      type_: f_2,
      crossposting: f_3,
      token: f_4,
      fb_id: f_5,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FacebookConnection");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.type_ {
      o_prot.write_field_begin(&TFieldIdentifier::new("type", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.crossposting {
      o_prot.write_field_begin(&TFieldIdentifier::new("crossposting", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.token {
      o_prot.write_field_begin(&TFieldIdentifier::new("token", TType::String, 4))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.fb_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("fb_id", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FacebookConnections {
  pub connections: Option<Vec<FacebookConnection>>,
}

impl FacebookConnections {
  pub fn new<F1>(connections: F1) -> FacebookConnections where F1: Into<Option<Vec<FacebookConnection>>> {
    FacebookConnections {
      connections: connections.into(),
    }
  }
}

impl TSerializable for FacebookConnections {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<FacebookConnections> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<FacebookConnection>> = Some(Vec::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<FacebookConnection> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_66 = FacebookConnection::read_from_in_protocol(i_prot)?;
            val.push(list_elem_66);
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
    let ret = FacebookConnections {
      connections: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("FacebookConnections");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.connections {
      o_prot.write_field_begin(&TFieldIdentifier::new("connections", TType::List, 1))?;
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
pub struct PeriscopeConnection {
  pub periscope_user_id: Option<String>,
  pub periscope_user_type: Option<PeriscopeUserType>,
}

impl PeriscopeConnection {
  pub fn new<F1, F2>(periscope_user_id: F1, periscope_user_type: F2) -> PeriscopeConnection where F1: Into<Option<String>>, F2: Into<Option<PeriscopeUserType>> {
    PeriscopeConnection {
      periscope_user_id: periscope_user_id.into(),
      periscope_user_type: periscope_user_type.into(),
    }
  }
}

impl TSerializable for PeriscopeConnection {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PeriscopeConnection> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<PeriscopeUserType> = None;
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
          let val = PeriscopeUserType::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = PeriscopeConnection {
      periscope_user_id: f_1,
      periscope_user_type: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("PeriscopeConnection");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.periscope_user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("periscope_user_id", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.periscope_user_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("periscope_user_type", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProfileConnection {
  pub id: Option<String>,
  pub crossposting: Option<bool>,
  pub token: Option<String>,
}

impl ProfileConnection {
  pub fn new<F1, F2, F3>(id: F1, crossposting: F2, token: F3) -> ProfileConnection where F1: Into<Option<String>>, F2: Into<Option<bool>>, F3: Into<Option<String>> {
    ProfileConnection {
      id: id.into(),
      crossposting: crossposting.into(),
      token: token.into(),
    }
  }
}

impl TSerializable for ProfileConnection {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProfileConnection> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<bool> = Some(false);
    let mut f_3: Option<String> = None;
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
    let ret = ProfileConnection {
      id: f_1,
      crossposting: f_2,
      token: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ProfileConnection");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.crossposting {
      o_prot.write_field_begin(&TFieldIdentifier::new("crossposting", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.token {
      o_prot.write_field_begin(&TFieldIdentifier::new("token", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PageConnection {
  pub id: Option<String>,
  pub crossposting: Option<bool>,
  pub token: Option<String>,
}

impl PageConnection {
  pub fn new<F1, F2, F3>(id: F1, crossposting: F2, token: F3) -> PageConnection where F1: Into<Option<String>>, F2: Into<Option<bool>>, F3: Into<Option<String>> {
    PageConnection {
      id: id.into(),
      crossposting: crossposting.into(),
      token: token.into(),
    }
  }
}

impl TSerializable for PageConnection {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PageConnection> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<bool> = Some(false);
    let mut f_3: Option<String> = None;
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
    let ret = PageConnection {
      id: f_1,
      crossposting: f_2,
      token: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("PageConnection");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.crossposting {
      o_prot.write_field_begin(&TFieldIdentifier::new("crossposting", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.token {
      o_prot.write_field_begin(&TFieldIdentifier::new("token", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Facebook {
  pub profile_connection: Option<ProfileConnection>,
  pub page_connection: Option<PageConnection>,
}

impl Facebook {
  pub fn new<F1, F2>(profile_connection: F1, page_connection: F2) -> Facebook where F1: Into<Option<ProfileConnection>>, F2: Into<Option<PageConnection>> {
    Facebook {
      profile_connection: profile_connection.into(),
      page_connection: page_connection.into(),
    }
  }
}

impl TSerializable for Facebook {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Facebook> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<ProfileConnection> = None;
    let mut f_2: Option<PageConnection> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = ProfileConnection::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = PageConnection::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Facebook {
      profile_connection: f_1,
      page_connection: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Facebook");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.profile_connection {
      o_prot.write_field_begin(&TFieldIdentifier::new("profile_connection", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.page_connection {
      o_prot.write_field_begin(&TFieldIdentifier::new("page_connection", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ThirdPartyConnections {
  pub periscope_connection: Option<PeriscopeConnection>,
  pub facebook: Option<Facebook>,
}

impl ThirdPartyConnections {
  pub fn new<F1, F2>(periscope_connection: F1, facebook: F2) -> ThirdPartyConnections where F1: Into<Option<PeriscopeConnection>>, F2: Into<Option<Facebook>> {
    ThirdPartyConnections {
      periscope_connection: periscope_connection.into(),
      facebook: facebook.into(),
    }
  }
}

impl TSerializable for ThirdPartyConnections {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ThirdPartyConnections> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<PeriscopeConnection> = None;
    let mut f_2: Option<Facebook> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = PeriscopeConnection::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = Facebook::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ThirdPartyConnections {
      periscope_connection: f_1,
      facebook: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ThirdPartyConnections");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.periscope_connection {
      o_prot.write_field_begin(&TFieldIdentifier::new("periscopeConnection", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.facebook {
      o_prot.write_field_begin(&TFieldIdentifier::new("facebook", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MutedKeyword {
  pub id: Option<i64>,
  pub keyword: Option<String>,
  pub valid_from: Option<i64>,
  pub valid_until: Option<i64>,
  pub created_at_msec: Option<i64>,
  pub mute_surfaces: Option<BTreeSet<MuteSurface>>,
  pub mute_options: Option<BTreeSet<MuteOption>>,
}

impl MutedKeyword {
  pub fn new<F1, F2, F3, F4, F5, F6, F7>(id: F1, keyword: F2, valid_from: F3, valid_until: F4, created_at_msec: F5, mute_surfaces: F6, mute_options: F7) -> MutedKeyword where F1: Into<Option<i64>>, F2: Into<Option<String>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<i64>>, F6: Into<Option<BTreeSet<MuteSurface>>>, F7: Into<Option<BTreeSet<MuteOption>>> {
    MutedKeyword {
      id: id.into(),
      keyword: keyword.into(),
      valid_from: valid_from.into(),
      valid_until: valid_until.into(),
      created_at_msec: created_at_msec.into(),
      mute_surfaces: mute_surfaces.into(),
      mute_options: mute_options.into(),
    }
  }
}

impl TSerializable for MutedKeyword {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MutedKeyword> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<i64> = None;
    let mut f_4: Option<i64> = None;
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<BTreeSet<MuteSurface>> = None;
    let mut f_7: Option<BTreeSet<MuteOption>> = None;
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
          let val = i_prot.read_i64()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_i64()?;
          f_5 = Some(val);
        },
        6 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<MuteSurface> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_67 = MuteSurface::read_from_in_protocol(i_prot)?;
            val.insert(set_elem_67);
          }
          i_prot.read_set_end()?;
          f_6 = Some(val);
        },
        7 => {
          let set_ident = i_prot.read_set_begin()?;
          let mut val: BTreeSet<MuteOption> = BTreeSet::new();
          for _ in 0..set_ident.size {
            let set_elem_68 = MuteOption::read_from_in_protocol(i_prot)?;
            val.insert(set_elem_68);
          }
          i_prot.read_set_end()?;
          f_7 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = MutedKeyword {
      id: f_1,
      keyword: f_2,
      valid_from: f_3,
      valid_until: f_4,
      created_at_msec: f_5,
      mute_surfaces: f_6,
      mute_options: f_7,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MutedKeyword");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.keyword {
      o_prot.write_field_begin(&TFieldIdentifier::new("keyword", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.valid_from {
      o_prot.write_field_begin(&TFieldIdentifier::new("valid_from", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.valid_until {
      o_prot.write_field_begin(&TFieldIdentifier::new("valid_until", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_msec", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.mute_surfaces {
      o_prot.write_field_begin(&TFieldIdentifier::new("mute_surfaces", TType::Set, 6))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::I32, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_set_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.mute_options {
      o_prot.write_field_begin(&TFieldIdentifier::new("mute_options", TType::Set, 7))?;
      o_prot.write_set_begin(&TSetIdentifier::new(TType::I32, fld_var.len() as i32))?;
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
pub struct AdvancedFilters {
    pub filter_no_confirmed_email: Option<bool>,
    pub filter_no_confirmed_phone: Option<bool>,
    pub filter_default_profile_image: Option<bool>,
      pub filter_new_users: Option<bool>,
    pub filter_not_followed_by: Option<bool>,
}

impl AdvancedFilters {
  pub fn new<F1, F2, F3, F4, F5>(filter_no_confirmed_email: F1, filter_no_confirmed_phone: F2, filter_default_profile_image: F3, filter_new_users: F4, filter_not_followed_by: F5) -> AdvancedFilters where F1: Into<Option<bool>>, F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<bool>>, F5: Into<Option<bool>> {
    AdvancedFilters {
      filter_no_confirmed_email: filter_no_confirmed_email.into(),
      filter_no_confirmed_phone: filter_no_confirmed_phone.into(),
      filter_default_profile_image: filter_default_profile_image.into(),
      filter_new_users: filter_new_users.into(),
      filter_not_followed_by: filter_not_followed_by.into(),
    }
  }
}

impl TSerializable for AdvancedFilters {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AdvancedFilters> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = None;
    let mut f_2: Option<bool> = None;
    let mut f_3: Option<bool> = None;
    let mut f_4: Option<bool> = None;
    let mut f_5: Option<bool> = None;
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
        2 => {
          let val = i_prot.read_bool()?;
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
    let ret = AdvancedFilters {
      filter_no_confirmed_email: f_1,
      filter_no_confirmed_phone: f_2,
      filter_default_profile_image: f_3,
      filter_new_users: f_4,
      filter_not_followed_by: f_5,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AdvancedFilters");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.filter_no_confirmed_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("filter_no_confirmed_email", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.filter_no_confirmed_phone {
      o_prot.write_field_begin(&TFieldIdentifier::new("filter_no_confirmed_phone", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.filter_default_profile_image {
      o_prot.write_field_begin(&TFieldIdentifier::new("filter_default_profile_image", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.filter_new_users {
      o_prot.write_field_begin(&TFieldIdentifier::new("filter_new_users", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.filter_not_followed_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("filter_not_followed_by", TType::Bool, 5))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MuteSettings {
  pub muted_keywords: Option<Vec<MutedKeyword>>,
  pub advanced_notification_filters: Option<AdvancedFilters>,
}

impl MuteSettings {
  pub fn new<F1, F2>(muted_keywords: F1, advanced_notification_filters: F2) -> MuteSettings where F1: Into<Option<Vec<MutedKeyword>>>, F2: Into<Option<AdvancedFilters>> {
    MuteSettings {
      muted_keywords: muted_keywords.into(),
      advanced_notification_filters: advanced_notification_filters.into(),
    }
  }
}

impl TSerializable for MuteSettings {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MuteSettings> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<MutedKeyword>> = Some(Vec::new());
    let mut f_2: Option<AdvancedFilters> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<MutedKeyword> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_69 = MutedKeyword::read_from_in_protocol(i_prot)?;
            val.push(list_elem_69);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let val = AdvancedFilters::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = MuteSettings {
      muted_keywords: f_1,
      advanced_notification_filters: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MuteSettings");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.muted_keywords {
      o_prot.write_field_begin(&TFieldIdentifier::new("muted_keywords", TType::List, 1))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.advanced_notification_filters {
      o_prot.write_field_begin(&TFieldIdentifier::new("advanced_notification_filters", TType::Struct, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MessagingDeviceFeatures {
  pub mo_mms: Option<bool>,
  pub mo_sms: Option<bool>,
  pub mt_mms: Option<bool>,
  pub mt_sms: Option<bool>,
  pub follow_limit: Option<i32>,
  pub mt_sms_limit: Option<i32>,
}

impl MessagingDeviceFeatures {
  pub fn new<F1, F2, F3, F4, F5, F6>(mo_mms: F1, mo_sms: F2, mt_mms: F3, mt_sms: F4, follow_limit: F5, mt_sms_limit: F6) -> MessagingDeviceFeatures where F1: Into<Option<bool>>, F2: Into<Option<bool>>, F3: Into<Option<bool>>, F4: Into<Option<bool>>, F5: Into<Option<i32>>, F6: Into<Option<i32>> {
    MessagingDeviceFeatures {
      mo_mms: mo_mms.into(),
      mo_sms: mo_sms.into(),
      mt_mms: mt_mms.into(),
      mt_sms: mt_sms.into(),
      follow_limit: follow_limit.into(),
      mt_sms_limit: mt_sms_limit.into(),
    }
  }
}

impl TSerializable for MessagingDeviceFeatures {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MessagingDeviceFeatures> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = Some(false);
    let mut f_2: Option<bool> = Some(false);
    let mut f_3: Option<bool> = Some(false);
    let mut f_4: Option<bool> = Some(false);
    let mut f_5: Option<i32> = None;
    let mut f_6: Option<i32> = None;
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
        2 => {
          let val = i_prot.read_bool()?;
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
        5 => {
          let val = i_prot.read_i32()?;
          f_5 = Some(val);
        },
        6 => {
          let val = i_prot.read_i32()?;
          f_6 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = MessagingDeviceFeatures {
      mo_mms: f_1,
      mo_sms: f_2,
      mt_mms: f_3,
      mt_sms: f_4,
      follow_limit: f_5,
      mt_sms_limit: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MessagingDeviceFeatures");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.mo_mms {
      o_prot.write_field_begin(&TFieldIdentifier::new("mo_mms", TType::Bool, 1))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.mo_sms {
      o_prot.write_field_begin(&TFieldIdentifier::new("mo_sms", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.mt_mms {
      o_prot.write_field_begin(&TFieldIdentifier::new("mt_mms", TType::Bool, 3))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.mt_sms {
      o_prot.write_field_begin(&TFieldIdentifier::new("mt_sms", TType::Bool, 4))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.follow_limit {
      o_prot.write_field_begin(&TFieldIdentifier::new("follow_limit", TType::I32, 5))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.mt_sms_limit {
      o_prot.write_field_begin(&TFieldIdentifier::new("mt_sms_limit", TType::I32, 6))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MessagingDevice {
  pub device_id: Option<i64>,
  pub user_id: Option<i64>,
  pub device_type: Option<MessagingDeviceType>,
  pub created_at: Option<i64>,
  pub updated_at: Option<i64>,
  pub verified: Option<bool>,
  pub address: Option<String>,
  pub phone_number: Option<String>,
  pub locale: Option<String>,
  pub short_code: Option<String>,
  pub enabled_for: Option<String>,
  pub carrier_id: Option<String>,
  pub carrier_name: Option<String>,
  pub country_code: Option<String>,
  pub country_name: Option<String>,
  pub features: Option<MessagingDeviceFeatures>,
}

impl MessagingDevice {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16>(device_id: F1, user_id: F2, device_type: F3, created_at: F4, updated_at: F5, verified: F6, address: F7, phone_number: F8, locale: F9, short_code: F10, enabled_for: F11, carrier_id: F12, carrier_name: F13, country_code: F14, country_name: F15, features: F16) -> MessagingDevice where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<MessagingDeviceType>>, F4: Into<Option<i64>>, F5: Into<Option<i64>>, F6: Into<Option<bool>>, F7: Into<Option<String>>, F8: Into<Option<String>>, F9: Into<Option<String>>, F10: Into<Option<String>>, F11: Into<Option<String>>, F12: Into<Option<String>>, F13: Into<Option<String>>, F14: Into<Option<String>>, F15: Into<Option<String>>, F16: Into<Option<MessagingDeviceFeatures>> {
    MessagingDevice {
      device_id: device_id.into(),
      user_id: user_id.into(),
      device_type: device_type.into(),
      created_at: created_at.into(),
      updated_at: updated_at.into(),
      verified: verified.into(),
      address: address.into(),
      phone_number: phone_number.into(),
      locale: locale.into(),
      short_code: short_code.into(),
      enabled_for: enabled_for.into(),
      carrier_id: carrier_id.into(),
      carrier_name: carrier_name.into(),
      country_code: country_code.into(),
      country_name: country_name.into(),
      features: features.into(),
    }
  }
}

impl TSerializable for MessagingDevice {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<MessagingDevice> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<MessagingDeviceType> = None;
    let mut f_4: Option<i64> = None;
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<bool> = None;
    let mut f_7: Option<String> = None;
    let mut f_8: Option<String> = None;
    let mut f_9: Option<String> = None;
    let mut f_10: Option<String> = None;
    let mut f_11: Option<String> = None;
    let mut f_12: Option<String> = None;
    let mut f_13: Option<String> = None;
    let mut f_14: Option<String> = None;
    let mut f_15: Option<String> = None;
    let mut f_16: Option<MessagingDeviceFeatures> = None;
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
          let val = MessagingDeviceType::read_from_in_protocol(i_prot)?;
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
          let val = i_prot.read_bool()?;
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
          let val = i_prot.read_string()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_string()?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_string()?;
          f_12 = Some(val);
        },
        13 => {
          let val = i_prot.read_string()?;
          f_13 = Some(val);
        },
        14 => {
          let val = i_prot.read_string()?;
          f_14 = Some(val);
        },
        15 => {
          let val = i_prot.read_string()?;
          f_15 = Some(val);
        },
        16 => {
          let val = MessagingDeviceFeatures::read_from_in_protocol(i_prot)?;
          f_16 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = MessagingDevice {
      device_id: f_1,
      user_id: f_2,
      device_type: f_3,
      created_at: f_4,
      updated_at: f_5,
      verified: f_6,
      address: f_7,
      phone_number: f_8,
      locale: f_9,
      short_code: f_10,
      enabled_for: f_11,
      carrier_id: f_12,
      carrier_name: f_13,
      country_code: f_14,
      country_name: f_15,
      features: f_16,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("MessagingDevice");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.device_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("device_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.device_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("device_type", TType::I32, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.updated_at {
      o_prot.write_field_begin(&TFieldIdentifier::new("updated_at", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.verified {
      o_prot.write_field_begin(&TFieldIdentifier::new("verified", TType::Bool, 6))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.address {
      o_prot.write_field_begin(&TFieldIdentifier::new("address", TType::String, 7))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.phone_number {
      o_prot.write_field_begin(&TFieldIdentifier::new("phone_number", TType::String, 8))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.locale {
      o_prot.write_field_begin(&TFieldIdentifier::new("locale", TType::String, 9))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.short_code {
      o_prot.write_field_begin(&TFieldIdentifier::new("short_code", TType::String, 10))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.enabled_for {
      o_prot.write_field_begin(&TFieldIdentifier::new("enabled_for", TType::String, 11))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.carrier_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("carrier_id", TType::String, 12))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.carrier_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("carrier_name", TType::String, 13))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.country_code {
      o_prot.write_field_begin(&TFieldIdentifier::new("country_code", TType::String, 14))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.country_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("country_name", TType::String, 15))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.features {
      o_prot.write_field_begin(&TFieldIdentifier::new("features", TType::Struct, 16))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AppPushDevice {
  pub device_id: Option<i64>,
  pub user_id: Option<i64>,
  pub device_type: Option<AppPushDeviceType>,
  pub token: Option<String>,
  pub udid: Option<String>,
  pub created_at: Option<i64>,
  pub updated_at: Option<i64>,
  pub enabled_for: Option<i32>,
  pub app_version: Option<i32>,
  pub client_application_id: Option<i32>,
  pub environment: Option<i32>,
  pub available_levels: Option<i32>,
  pub display: Option<i32>,
  pub locale: Option<String>,
  pub description: Option<String>,
  pub push_last_received_ts: Option<i64>,
  pub last_active_ts: Option<i64>,
  pub app_release_version: Option<String>,
  pub os_version: Option<String>,
  pub encryption_key_1: Option<String>,
  pub encryption_key_2: Option<String>,
  pub settings: Option<BTreeMap<String, String>>,
}

impl AppPushDevice {
  pub fn new<F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20, F21, F22>(device_id: F1, user_id: F2, device_type: F3, token: F4, udid: F5, created_at: F6, updated_at: F7, enabled_for: F8, app_version: F9, client_application_id: F10, environment: F11, available_levels: F12, display: F13, locale: F14, description: F15, push_last_received_ts: F16, last_active_ts: F17, app_release_version: F18, os_version: F19, encryption_key_1: F20, encryption_key_2: F21, settings: F22) -> AppPushDevice where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<AppPushDeviceType>>, F4: Into<Option<String>>, F5: Into<Option<String>>, F6: Into<Option<i64>>, F7: Into<Option<i64>>, F8: Into<Option<i32>>, F9: Into<Option<i32>>, F10: Into<Option<i32>>, F11: Into<Option<i32>>, F12: Into<Option<i32>>, F13: Into<Option<i32>>, F14: Into<Option<String>>, F15: Into<Option<String>>, F16: Into<Option<i64>>, F17: Into<Option<i64>>, F18: Into<Option<String>>, F19: Into<Option<String>>, F20: Into<Option<String>>, F21: Into<Option<String>>, F22: Into<Option<BTreeMap<String, String>>> {
    AppPushDevice {
      device_id: device_id.into(),
      user_id: user_id.into(),
      device_type: device_type.into(),
      token: token.into(),
      udid: udid.into(),
      created_at: created_at.into(),
      updated_at: updated_at.into(),
      enabled_for: enabled_for.into(),
      app_version: app_version.into(),
      client_application_id: client_application_id.into(),
      environment: environment.into(),
      available_levels: available_levels.into(),
      display: display.into(),
      locale: locale.into(),
      description: description.into(),
      push_last_received_ts: push_last_received_ts.into(),
      last_active_ts: last_active_ts.into(),
      app_release_version: app_release_version.into(),
      os_version: os_version.into(),
      encryption_key_1: encryption_key_1.into(),
      encryption_key_2: encryption_key_2.into(),
      settings: settings.into(),
    }
  }
}

impl TSerializable for AppPushDevice {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AppPushDevice> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = None;
    let mut f_3: Option<AppPushDeviceType> = None;
    let mut f_4: Option<String> = Some("".to_owned());
    let mut f_5: Option<String> = Some("".to_owned());
    let mut f_6: Option<i64> = None;
    let mut f_7: Option<i64> = None;
    let mut f_8: Option<i32> = None;
    let mut f_9: Option<i32> = None;
    let mut f_10: Option<i32> = None;
    let mut f_11: Option<i32> = None;
    let mut f_12: Option<i32> = None;
    let mut f_13: Option<i32> = None;
    let mut f_14: Option<String> = None;
    let mut f_15: Option<String> = None;
    let mut f_16: Option<i64> = None;
    let mut f_17: Option<i64> = None;
    let mut f_18: Option<String> = None;
    let mut f_19: Option<String> = None;
    let mut f_20: Option<String> = None;
    let mut f_21: Option<String> = None;
    let mut f_22: Option<BTreeMap<String, String>> = None;
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
          let val = AppPushDeviceType::read_from_in_protocol(i_prot)?;
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
        7 => {
          let val = i_prot.read_i64()?;
          f_7 = Some(val);
        },
        8 => {
          let val = i_prot.read_i32()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_i32()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_i32()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_i32()?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_i32()?;
          f_12 = Some(val);
        },
        13 => {
          let val = i_prot.read_i32()?;
          f_13 = Some(val);
        },
        14 => {
          let val = i_prot.read_string()?;
          f_14 = Some(val);
        },
        15 => {
          let val = i_prot.read_string()?;
          f_15 = Some(val);
        },
        16 => {
          let val = i_prot.read_i64()?;
          f_16 = Some(val);
        },
        17 => {
          let val = i_prot.read_i64()?;
          f_17 = Some(val);
        },
        18 => {
          let val = i_prot.read_string()?;
          f_18 = Some(val);
        },
        19 => {
          let val = i_prot.read_string()?;
          f_19 = Some(val);
        },
        20 => {
          let val = i_prot.read_string()?;
          f_20 = Some(val);
        },
        21 => {
          let val = i_prot.read_string()?;
          f_21 = Some(val);
        },
        22 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, String> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_70 = i_prot.read_string()?;
            let map_val_71 = i_prot.read_string()?;
            val.insert(map_key_70, map_val_71);
          }
          i_prot.read_map_end()?;
          f_22 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = AppPushDevice {
      device_id: f_1,
      user_id: f_2,
      device_type: f_3,
      token: f_4,
      udid: f_5,
      created_at: f_6,
      updated_at: f_7,
      enabled_for: f_8,
      app_version: f_9,
      client_application_id: f_10,
      environment: f_11,
      available_levels: f_12,
      display: f_13,
      locale: f_14,
      description: f_15,
      push_last_received_ts: f_16,
      last_active_ts: f_17,
      app_release_version: f_18,
      os_version: f_19,
      encryption_key_1: f_20,
      encryption_key_2: f_21,
      settings: f_22,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AppPushDevice");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.device_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("device_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.user_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_id", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.device_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("device_type", TType::I32, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.token {
      o_prot.write_field_begin(&TFieldIdentifier::new("token", TType::String, 4))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.udid {
      o_prot.write_field_begin(&TFieldIdentifier::new("udid", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at", TType::I64, 6))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.updated_at {
      o_prot.write_field_begin(&TFieldIdentifier::new("updated_at", TType::I64, 7))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.enabled_for {
      o_prot.write_field_begin(&TFieldIdentifier::new("enabled_for", TType::I32, 8))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.app_version {
      o_prot.write_field_begin(&TFieldIdentifier::new("app_version", TType::I32, 9))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.client_application_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("client_application_id", TType::I32, 10))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.environment {
      o_prot.write_field_begin(&TFieldIdentifier::new("environment", TType::I32, 11))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.available_levels {
      o_prot.write_field_begin(&TFieldIdentifier::new("available_levels", TType::I32, 12))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.display {
      o_prot.write_field_begin(&TFieldIdentifier::new("display", TType::I32, 13))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.locale {
      o_prot.write_field_begin(&TFieldIdentifier::new("locale", TType::String, 14))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.description {
      o_prot.write_field_begin(&TFieldIdentifier::new("description", TType::String, 15))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.push_last_received_ts {
      o_prot.write_field_begin(&TFieldIdentifier::new("push_last_received_ts", TType::I64, 16))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.last_active_ts {
      o_prot.write_field_begin(&TFieldIdentifier::new("last_active_ts", TType::I64, 17))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.app_release_version {
      o_prot.write_field_begin(&TFieldIdentifier::new("app_release_version", TType::String, 18))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.os_version {
      o_prot.write_field_begin(&TFieldIdentifier::new("os_version", TType::String, 19))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.encryption_key_1 {
      o_prot.write_field_begin(&TFieldIdentifier::new("encryption_key_1", TType::String, 20))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.encryption_key_2 {
      o_prot.write_field_begin(&TFieldIdentifier::new("encryption_key_2", TType::String, 21))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.settings {
      o_prot.write_field_begin(&TFieldIdentifier::new("settings", TType::Map, 22))?;
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
pub struct Devices {
  pub messaging_devices: Option<Vec<MessagingDevice>>,
  pub app_push_devices: Option<Vec<AppPushDevice>>,
}

impl Devices {
  pub fn new<F1, F2>(messaging_devices: F1, app_push_devices: F2) -> Devices where F1: Into<Option<Vec<MessagingDevice>>>, F2: Into<Option<Vec<AppPushDevice>>> {
    Devices {
      messaging_devices: messaging_devices.into(),
      app_push_devices: app_push_devices.into(),
    }
  }
}

impl TSerializable for Devices {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Devices> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<MessagingDevice>> = Some(Vec::new());
    let mut f_2: Option<Vec<AppPushDevice>> = Some(Vec::new());
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<MessagingDevice> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_72 = MessagingDevice::read_from_in_protocol(i_prot)?;
            val.push(list_elem_72);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<AppPushDevice> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_73 = AppPushDevice::read_from_in_protocol(i_prot)?;
            val.push(list_elem_73);
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
    let ret = Devices {
      messaging_devices: f_1,
      app_push_devices: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Devices");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.messaging_devices {
      o_prot.write_field_begin(&TFieldIdentifier::new("messaging_devices", TType::List, 1))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.app_push_devices {
      o_prot.write_field_begin(&TFieldIdentifier::new("app_push_devices", TType::List, 2))?;
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
pub struct Place {
  pub place_id: Option<String>,
  pub confidence: Option<OrderedFloat<f64>>,
  pub display_name: Option<String>,
  pub names: Option<BTreeMap<String, String>>,
}

impl Place {
  pub fn new<F1, F2, F3, F4>(place_id: F1, confidence: F2, display_name: F3, names: F4) -> Place where F1: Into<Option<String>>, F2: Into<Option<OrderedFloat<f64>>>, F3: Into<Option<String>>, F4: Into<Option<BTreeMap<String, String>>> {
    Place {
      place_id: place_id.into(),
      confidence: confidence.into(),
      display_name: display_name.into(),
      names: names.into(),
    }
  }
}

impl TSerializable for Place {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Place> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<OrderedFloat<f64>> = None;
    let mut f_3: Option<String> = None;
    let mut f_4: Option<BTreeMap<String, String>> = None;
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
          let val = OrderedFloat::from(i_prot.read_double()?);
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let map_ident = i_prot.read_map_begin()?;
          let mut val: BTreeMap<String, String> = BTreeMap::new();
          for _ in 0..map_ident.size {
            let map_key_74 = i_prot.read_string()?;
            let map_val_75 = i_prot.read_string()?;
            val.insert(map_key_74, map_val_75);
          }
          i_prot.read_map_end()?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Place {
      place_id: f_1,
      confidence: f_2,
      display_name: f_3,
      names: f_4,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Place");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.place_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("place_id", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.confidence {
      o_prot.write_field_begin(&TFieldIdentifier::new("confidence", TType::Double, 2))?;
      o_prot.write_double(fld_var.into())?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.display_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("display_name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.names {
      o_prot.write_field_begin(&TFieldIdentifier::new("names", TType::Map, 4))?;
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
pub struct PublicLocation {
  pub country: Option<Place>,
}

impl PublicLocation {
  pub fn new<F1>(country: F1) -> PublicLocation where F1: Into<Option<Place>> {
    PublicLocation {
      country: country.into(),
    }
  }
}

impl TSerializable for PublicLocation {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<PublicLocation> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Place> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = Place::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = PublicLocation {
      country: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("PublicLocation");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.country {
      o_prot.write_field_begin(&TFieldIdentifier::new("country", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Derived {
  pub user_state: Option<DerivedUserState>,
  pub cohort: Option<UserCohort>,
}

impl Derived {
  pub fn new<F1, F2>(user_state: F1, cohort: F2) -> Derived where F1: Into<Option<DerivedUserState>>, F2: Into<Option<UserCohort>> {
    Derived {
      user_state: user_state.into(),
      cohort: cohort.into(),
    }
  }
}

impl TSerializable for Derived {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Derived> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<DerivedUserState> = None;
    let mut f_2: Option<UserCohort> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = DerivedUserState::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = UserCohort::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Derived {
      user_state: f_1,
      cohort: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Derived");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.user_state {
      o_prot.write_field_begin(&TFieldIdentifier::new("user_state", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.cohort {
      o_prot.write_field_begin(&TFieldIdentifier::new("cohort", TType::I32, 2))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProfileLocation {
  pub place: Option<Place>,
}

impl ProfileLocation {
  pub fn new<F1>(place: F1) -> ProfileLocation where F1: Into<Option<Place>> {
    ProfileLocation {
      place: place.into(),
    }
  }
}

impl TSerializable for ProfileLocation {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProfileLocation> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Place> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = Place::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ProfileLocation {
      place: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ProfileLocation");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.place {
      o_prot.write_field_begin(&TFieldIdentifier::new("place", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Label {
  pub id: Option<i64>,
  pub created_at_msec: Option<i64>,
  pub expires_at_msec: Option<i64>,
  pub by_user: Option<String>,
  pub reason: Option<String>,
  pub source: Option<String>,
}

impl Label {
  pub fn new<F1, F3, F4, F5, F6, F7>(id: F1, created_at_msec: F3, expires_at_msec: F4, by_user: F5, reason: F6, source: F7) -> Label where F1: Into<Option<i64>>, F3: Into<Option<i64>>, F4: Into<Option<i64>>, F5: Into<Option<String>>, F6: Into<Option<String>>, F7: Into<Option<String>> {
    Label {
      id: id.into(),
      created_at_msec: created_at_msec.into(),
      expires_at_msec: expires_at_msec.into(),
      by_user: by_user.into(),
      reason: reason.into(),
      source: source.into(),
    }
  }
}

impl TSerializable for Label {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Label> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_4: Option<i64> = None;
    let mut f_5: Option<String> = Some("".to_owned());
    let mut f_6: Option<String> = None;
    let mut f_7: Option<String> = None;
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
        3 => {
          let val = i_prot.read_i64()?;
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
          let val = i_prot.read_string()?;
          f_6 = Some(val);
        },
        7 => {
          let val = i_prot.read_string()?;
          f_7 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Label {
      id: f_1,
      created_at_msec: f_3,
      expires_at_msec: f_4,
      by_user: f_5,
      reason: f_6,
      source: f_7,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Label");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_msec", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.expires_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("expires_at_msec", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.by_user {
      o_prot.write_field_begin(&TFieldIdentifier::new("by_user", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.reason {
      o_prot.write_field_begin(&TFieldIdentifier::new("reason", TType::String, 6))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.source {
      o_prot.write_field_begin(&TFieldIdentifier::new("source", TType::String, 7))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Labels {
  pub labels: Option<Vec<Label>>,
  pub exempted_labels: Option<Vec<Label>>,
}

impl Labels {
  pub fn new<F1, F2>(labels: F1, exempted_labels: F2) -> Labels where F1: Into<Option<Vec<Label>>>, F2: Into<Option<Vec<Label>>> {
    Labels {
      labels: labels.into(),
      exempted_labels: exempted_labels.into(),
    }
  }
}

impl TSerializable for Labels {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Labels> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<Label>> = Some(Vec::new());
    let mut f_2: Option<Vec<Label>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<Label> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_76 = Label::read_from_in_protocol(i_prot)?;
            val.push(list_elem_76);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        2 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<Label> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_77 = Label::read_from_in_protocol(i_prot)?;
            val.push(list_elem_77);
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
    let ret = Labels {
      labels: f_1,
      exempted_labels: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Labels");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.labels {
      o_prot.write_field_begin(&TFieldIdentifier::new("labels", TType::List, 1))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.exempted_labels {
      o_prot.write_field_begin(&TFieldIdentifier::new("exempted_labels", TType::List, 2))?;
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
pub struct AdvertiserAccount {
  pub advertiser_type: Option<AdvertiserType>,
}

impl AdvertiserAccount {
  pub fn new<F1>(advertiser_type: F1) -> AdvertiserAccount where F1: Into<Option<AdvertiserType>> {
    AdvertiserAccount {
      advertiser_type: advertiser_type.into(),
    }
  }
}

impl TSerializable for AdvertiserAccount {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AdvertiserAccount> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<AdvertiserType> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = AdvertiserType::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = AdvertiserAccount {
      advertiser_type: f_1,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("AdvertiserAccount");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.advertiser_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("advertiser_type", TType::I32, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProfileBirthdate {
  pub value: String,
  pub visibility: ProfileVisibility,
  pub birth_year_visibility: ProfileVisibility,
  pub day: Option<i32>,
  pub month: Option<i32>,
  pub year: Option<i32>,
}

impl ProfileBirthdate {
  pub fn new<F4, F5, F6>(value: String, visibility: ProfileVisibility, birth_year_visibility: ProfileVisibility, day: F4, month: F5, year: F6) -> ProfileBirthdate where F4: Into<Option<i32>>, F5: Into<Option<i32>>, F6: Into<Option<i32>> {
    ProfileBirthdate {
      value,
      visibility,
      birth_year_visibility,
      day: day.into(),
      month: month.into(),
      year: year.into(),
    }
  }
}

impl TSerializable for ProfileBirthdate {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ProfileBirthdate> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = None;
    let mut f_2: Option<ProfileVisibility> = None;
    let mut f_3: Option<ProfileVisibility> = None;
    let mut f_4: Option<i32> = None;
    let mut f_5: Option<i32> = None;
    let mut f_6: Option<i32> = None;
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
          let val = ProfileVisibility::read_from_in_protocol(i_prot)?;
          f_2 = Some(val);
        },
        3 => {
          let val = ProfileVisibility::read_from_in_protocol(i_prot)?;
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
        6 => {
          let val = i_prot.read_i32()?;
          f_6 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ProfileBirthdate.value", &f_1)?;
    verify_required_field_exists("ProfileBirthdate.visibility", &f_2)?;
    verify_required_field_exists("ProfileBirthdate.birth_year_visibility", &f_3)?;
    let ret = ProfileBirthdate {
      value: f_1.expect("auto-generated code should have checked for presence of required fields"),
      visibility: f_2.expect("auto-generated code should have checked for presence of required fields"),
      birth_year_visibility: f_3.expect("auto-generated code should have checked for presence of required fields"),
      day: f_4,
      month: f_5,
      year: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ProfileBirthdate");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("value", TType::String, 1))?;
    o_prot.write_string(&self.value)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("visibility", TType::I32, 2))?;
    self.visibility.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("birth_year_visibility", TType::I32, 3))?;
    self.birth_year_visibility.write_to_out_protocol(o_prot)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.day {
      o_prot.write_field_begin(&TFieldIdentifier::new("day", TType::I32, 4))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.month {
      o_prot.write_field_begin(&TFieldIdentifier::new("month", TType::I32, 5))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.year {
      o_prot.write_field_begin(&TFieldIdentifier::new("year", TType::I32, 6))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExtendedProfile {
  pub birthdate: Option<ProfileBirthdate>,
  pub has_birthdate: Option<bool>,
  pub age_in_years: Option<i32>,
  pub age_when_created_in_years: Option<i32>,
  pub zip_code: Option<String>,
  pub full_name: Option<String>,
}

impl ExtendedProfile {
  pub fn new<F1, F2, F3, F4, F5, F6>(birthdate: F1, has_birthdate: F2, age_in_years: F3, age_when_created_in_years: F4, zip_code: F5, full_name: F6) -> ExtendedProfile where F1: Into<Option<ProfileBirthdate>>, F2: Into<Option<bool>>, F3: Into<Option<i32>>, F4: Into<Option<i32>>, F5: Into<Option<String>>, F6: Into<Option<String>> {
    ExtendedProfile {
      birthdate: birthdate.into(),
      has_birthdate: has_birthdate.into(),
      age_in_years: age_in_years.into(),
      age_when_created_in_years: age_when_created_in_years.into(),
      zip_code: zip_code.into(),
      full_name: full_name.into(),
    }
  }
}

impl TSerializable for ExtendedProfile {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ExtendedProfile> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<ProfileBirthdate> = None;
    let mut f_2: Option<bool> = None;
    let mut f_3: Option<i32> = None;
    let mut f_4: Option<i32> = None;
    let mut f_5: Option<String> = None;
    let mut f_6: Option<String> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = ProfileBirthdate::read_from_in_protocol(i_prot)?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_bool()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i32()?;
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
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ExtendedProfile {
      birthdate: f_1,
      has_birthdate: f_2,
      age_in_years: f_3,
      age_when_created_in_years: f_4,
      zip_code: f_5,
      full_name: f_6,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ExtendedProfile");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.birthdate {
      o_prot.write_field_begin(&TFieldIdentifier::new("birthdate", TType::Struct, 1))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.has_birthdate {
      o_prot.write_field_begin(&TFieldIdentifier::new("has_birthdate", TType::Bool, 2))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.age_in_years {
      o_prot.write_field_begin(&TFieldIdentifier::new("age_in_years", TType::I32, 3))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.age_when_created_in_years {
      o_prot.write_field_begin(&TFieldIdentifier::new("age_when_created_in_years", TType::I32, 4))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.zip_code {
      o_prot.write_field_begin(&TFieldIdentifier::new("zip_code", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.full_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("full_name", TType::String, 6))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TestUserInfo {
  pub contact_email: Option<String>,
  pub created_by: Option<String>,
  pub description: Option<String>,
}

impl TestUserInfo {
  pub fn new<F1, F2, F3>(contact_email: F1, created_by: F2, description: F3) -> TestUserInfo where F1: Into<Option<String>>, F2: Into<Option<String>>, F3: Into<Option<String>> {
    TestUserInfo {
      contact_email: contact_email.into(),
      created_by: created_by.into(),
      description: description.into(),
    }
  }
}

impl TSerializable for TestUserInfo {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<TestUserInfo> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
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
    let ret = TestUserInfo {
      contact_email: f_1,
      created_by: f_2,
      description: f_3,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("TestUserInfo");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.contact_email {
      o_prot.write_field_begin(&TFieldIdentifier::new("contact_email", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.created_by {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_by", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.description {
      o_prot.write_field_begin(&TFieldIdentifier::new("description", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct User {
  pub id: Option<i64>,
  pub created_at_msec: Option<i64>,
  pub updated_at_msec: Option<i64>,
  pub profile: Option<Profile>,
  pub profile_design: Option<ProfileDesign>,
  pub account: Option<Account>,
  pub sleep: Option<Sleep>,
  pub safety: Option<Safety>,
  pub counts: Option<Counts>,
  pub notification: Option<Notification>,
  pub roles: Option<Roles>,
  pub prompts: Option<Prompts>,
  pub view: Option<View>,
  pub takedowns: Option<Takedowns>,
  pub discoverability: Option<Discoverability>,
  pub contribution: Option<Contribution>,
  pub url_entities: Option<UrlEntities>,
  pub saved_searches: Option<SavedSearches>,
            pub storage_state: Option<i32>,
  pub facebook_connections: Option<FacebookConnections>,
  pub devices: Option<Devices>,
  pub public_location: Option<PublicLocation>,
  pub media_view: Option<MediaView>,
  pub dm_view: Option<DirectMessageView>,
  pub perspective: Option<Perspective>,
  pub profile_location: Option<ProfileLocation>,
  pub derived: Option<Derived>,
  pub labels: Option<Labels>,
  pub advertiser_account: Option<AdvertiserAccount>,
  pub extended_profile: Option<ExtendedProfile>,
  pub annotations: Option<Annotations>,
        pub extensions_reply: Option<Vec<u8>>,
  pub third_party_connections: Option<ThirdPartyConnections>,
  pub mention_entities: Option<MentionEntities>,
  pub hashtag_entities: Option<HashtagEntities>,
  pub cashtag_entities: Option<CashtagEntities>,
  pub test_user_status: Option<TestUserStatus>,
  pub mute_settings: Option<MuteSettings>,
  pub test_user_info: Option<TestUserInfo>,
  pub compliance: Option<Compliance>,
  pub extended_account: Option<ExtendedAccount>,
}

impl User {
  pub fn new<F1, F2, F3, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F20, F21, F22, F23, F24, F25, F26, F27, F28, F29, F30, F31, F32, F33, F35, F36, F37, F38, F39, F40, F41, F42, F43, F44>(id: F1, created_at_msec: F2, updated_at_msec: F3, profile: F5, profile_design: F6, account: F7, sleep: F8, safety: F9, counts: F10, notification: F11, roles: F12, prompts: F13, view: F14, takedowns: F15, discoverability: F16, contribution: F17, url_entities: F18, saved_searches: F20, storage_state: F21, facebook_connections: F22, devices: F23, public_location: F24, media_view: F25, dm_view: F26, perspective: F27, profile_location: F28, derived: F29, labels: F30, advertiser_account: F31, extended_profile: F32, annotations: F33, extensions_reply: F35, third_party_connections: F36, mention_entities: F37, hashtag_entities: F38, cashtag_entities: F39, test_user_status: F40, mute_settings: F41, test_user_info: F42, compliance: F43, extended_account: F44) -> User where F1: Into<Option<i64>>, F2: Into<Option<i64>>, F3: Into<Option<i64>>, F5: Into<Option<Profile>>, F6: Into<Option<ProfileDesign>>, F7: Into<Option<Account>>, F8: Into<Option<Sleep>>, F9: Into<Option<Safety>>, F10: Into<Option<Counts>>, F11: Into<Option<Notification>>, F12: Into<Option<Roles>>, F13: Into<Option<Prompts>>, F14: Into<Option<View>>, F15: Into<Option<Takedowns>>, F16: Into<Option<Discoverability>>, F17: Into<Option<Contribution>>, F18: Into<Option<UrlEntities>>, F20: Into<Option<SavedSearches>>, F21: Into<Option<i32>>, F22: Into<Option<FacebookConnections>>, F23: Into<Option<Devices>>, F24: Into<Option<PublicLocation>>, F25: Into<Option<MediaView>>, F26: Into<Option<DirectMessageView>>, F27: Into<Option<Perspective>>, F28: Into<Option<ProfileLocation>>, F29: Into<Option<Derived>>, F30: Into<Option<Labels>>, F31: Into<Option<AdvertiserAccount>>, F32: Into<Option<ExtendedProfile>>, F33: Into<Option<Annotations>>, F35: Into<Option<Vec<u8>>>, F36: Into<Option<ThirdPartyConnections>>, F37: Into<Option<MentionEntities>>, F38: Into<Option<HashtagEntities>>, F39: Into<Option<CashtagEntities>>, F40: Into<Option<TestUserStatus>>, F41: Into<Option<MuteSettings>>, F42: Into<Option<TestUserInfo>>, F43: Into<Option<Compliance>>, F44: Into<Option<ExtendedAccount>> {
    User {
      id: id.into(),
      created_at_msec: created_at_msec.into(),
      updated_at_msec: updated_at_msec.into(),
      profile: profile.into(),
      profile_design: profile_design.into(),
      account: account.into(),
      sleep: sleep.into(),
      safety: safety.into(),
      counts: counts.into(),
      notification: notification.into(),
      roles: roles.into(),
      prompts: prompts.into(),
      view: view.into(),
      takedowns: takedowns.into(),
      discoverability: discoverability.into(),
      contribution: contribution.into(),
      url_entities: url_entities.into(),
      saved_searches: saved_searches.into(),
      storage_state: storage_state.into(),
      facebook_connections: facebook_connections.into(),
      devices: devices.into(),
      public_location: public_location.into(),
      media_view: media_view.into(),
      dm_view: dm_view.into(),
      perspective: perspective.into(),
      profile_location: profile_location.into(),
      derived: derived.into(),
      labels: labels.into(),
      advertiser_account: advertiser_account.into(),
      extended_profile: extended_profile.into(),
      annotations: annotations.into(),
      extensions_reply: extensions_reply.into(),
      third_party_connections: third_party_connections.into(),
      mention_entities: mention_entities.into(),
      hashtag_entities: hashtag_entities.into(),
      cashtag_entities: cashtag_entities.into(),
      test_user_status: test_user_status.into(),
      mute_settings: mute_settings.into(),
      test_user_info: test_user_info.into(),
      compliance: compliance.into(),
      extended_account: extended_account.into(),
    }
  }
}

impl TSerializable for User {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<User> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<i64> = Some(0);
    let mut f_3: Option<i64> = Some(0);
    let mut f_5: Option<Profile> = None;
    let mut f_6: Option<ProfileDesign> = None;
    let mut f_7: Option<Account> = None;
    let mut f_8: Option<Sleep> = None;
    let mut f_9: Option<Safety> = None;
    let mut f_10: Option<Counts> = None;
    let mut f_11: Option<Notification> = None;
    let mut f_12: Option<Roles> = None;
    let mut f_13: Option<Prompts> = None;
    let mut f_14: Option<View> = None;
    let mut f_15: Option<Takedowns> = None;
    let mut f_16: Option<Discoverability> = None;
    let mut f_17: Option<Contribution> = None;
    let mut f_18: Option<UrlEntities> = None;
    let mut f_20: Option<SavedSearches> = None;
    let mut f_21: Option<i32> = None;
    let mut f_22: Option<FacebookConnections> = None;
    let mut f_23: Option<Devices> = None;
    let mut f_24: Option<PublicLocation> = None;
    let mut f_25: Option<MediaView> = None;
    let mut f_26: Option<DirectMessageView> = None;
    let mut f_27: Option<Perspective> = None;
    let mut f_28: Option<ProfileLocation> = None;
    let mut f_29: Option<Derived> = None;
    let mut f_30: Option<Labels> = None;
    let mut f_31: Option<AdvertiserAccount> = None;
    let mut f_32: Option<ExtendedProfile> = None;
    let mut f_33: Option<Annotations> = None;
    let mut f_35: Option<Vec<u8>> = None;
    let mut f_36: Option<ThirdPartyConnections> = None;
    let mut f_37: Option<MentionEntities> = None;
    let mut f_38: Option<HashtagEntities> = None;
    let mut f_39: Option<CashtagEntities> = None;
    let mut f_40: Option<TestUserStatus> = None;
    let mut f_41: Option<MuteSettings> = None;
    let mut f_42: Option<TestUserInfo> = None;
    let mut f_43: Option<Compliance> = None;
    let mut f_44: Option<ExtendedAccount> = None;
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
        5 => {
          let val = Profile::read_from_in_protocol(i_prot)?;
          f_5 = Some(val);
        },
        6 => {
          let val = ProfileDesign::read_from_in_protocol(i_prot)?;
          f_6 = Some(val);
        },
        7 => {
          let val = Account::read_from_in_protocol(i_prot)?;
          f_7 = Some(val);
        },
        8 => {
          let val = Sleep::read_from_in_protocol(i_prot)?;
          f_8 = Some(val);
        },
        9 => {
          let val = Safety::read_from_in_protocol(i_prot)?;
          f_9 = Some(val);
        },
        10 => {
          let val = Counts::read_from_in_protocol(i_prot)?;
          f_10 = Some(val);
        },
        11 => {
          let val = Notification::read_from_in_protocol(i_prot)?;
          f_11 = Some(val);
        },
        12 => {
          let val = Roles::read_from_in_protocol(i_prot)?;
          f_12 = Some(val);
        },
        13 => {
          let val = Prompts::read_from_in_protocol(i_prot)?;
          f_13 = Some(val);
        },
        14 => {
          let val = View::read_from_in_protocol(i_prot)?;
          f_14 = Some(val);
        },
        15 => {
          let val = Takedowns::read_from_in_protocol(i_prot)?;
          f_15 = Some(val);
        },
        16 => {
          let val = Discoverability::read_from_in_protocol(i_prot)?;
          f_16 = Some(val);
        },
        17 => {
          let val = Contribution::read_from_in_protocol(i_prot)?;
          f_17 = Some(val);
        },
        18 => {
          let val = UrlEntities::read_from_in_protocol(i_prot)?;
          f_18 = Some(val);
        },
        20 => {
          let val = SavedSearches::read_from_in_protocol(i_prot)?;
          f_20 = Some(val);
        },
        21 => {
          let val = i_prot.read_i32()?;
          f_21 = Some(val);
        },
        22 => {
          let val = FacebookConnections::read_from_in_protocol(i_prot)?;
          f_22 = Some(val);
        },
        23 => {
          let val = Devices::read_from_in_protocol(i_prot)?;
          f_23 = Some(val);
        },
        24 => {
          let val = PublicLocation::read_from_in_protocol(i_prot)?;
          f_24 = Some(val);
        },
        25 => {
          let val = MediaView::read_from_in_protocol(i_prot)?;
          f_25 = Some(val);
        },
        26 => {
          let val = DirectMessageView::read_from_in_protocol(i_prot)?;
          f_26 = Some(val);
        },
        27 => {
          let val = Perspective::read_from_in_protocol(i_prot)?;
          f_27 = Some(val);
        },
        28 => {
          let val = ProfileLocation::read_from_in_protocol(i_prot)?;
          f_28 = Some(val);
        },
        29 => {
          let val = Derived::read_from_in_protocol(i_prot)?;
          f_29 = Some(val);
        },
        30 => {
          let val = Labels::read_from_in_protocol(i_prot)?;
          f_30 = Some(val);
        },
        31 => {
          let val = AdvertiserAccount::read_from_in_protocol(i_prot)?;
          f_31 = Some(val);
        },
        32 => {
          let val = ExtendedProfile::read_from_in_protocol(i_prot)?;
          f_32 = Some(val);
        },
        33 => {
          let val = Annotations::read_from_in_protocol(i_prot)?;
          f_33 = Some(val);
        },
        35 => {
          let val = i_prot.read_bytes()?;
          f_35 = Some(val);
        },
        36 => {
          let val = ThirdPartyConnections::read_from_in_protocol(i_prot)?;
          f_36 = Some(val);
        },
        37 => {
          let val = MentionEntities::read_from_in_protocol(i_prot)?;
          f_37 = Some(val);
        },
        38 => {
          let val = HashtagEntities::read_from_in_protocol(i_prot)?;
          f_38 = Some(val);
        },
        39 => {
          let val = CashtagEntities::read_from_in_protocol(i_prot)?;
          f_39 = Some(val);
        },
        40 => {
          let val = TestUserStatus::read_from_in_protocol(i_prot)?;
          f_40 = Some(val);
        },
        41 => {
          let val = MuteSettings::read_from_in_protocol(i_prot)?;
          f_41 = Some(val);
        },
        42 => {
          let val = TestUserInfo::read_from_in_protocol(i_prot)?;
          f_42 = Some(val);
        },
        43 => {
          let val = Compliance::read_from_in_protocol(i_prot)?;
          f_43 = Some(val);
        },
        44 => {
          let val = ExtendedAccount::read_from_in_protocol(i_prot)?;
          f_44 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = User {
      id: f_1,
      created_at_msec: f_2,
      updated_at_msec: f_3,
      profile: f_5,
      profile_design: f_6,
      account: f_7,
      sleep: f_8,
      safety: f_9,
      counts: f_10,
      notification: f_11,
      roles: f_12,
      prompts: f_13,
      view: f_14,
      takedowns: f_15,
      discoverability: f_16,
      contribution: f_17,
      url_entities: f_18,
      saved_searches: f_20,
      storage_state: f_21,
      facebook_connections: f_22,
      devices: f_23,
      public_location: f_24,
      media_view: f_25,
      dm_view: f_26,
      perspective: f_27,
      profile_location: f_28,
      derived: f_29,
      labels: f_30,
      advertiser_account: f_31,
      extended_profile: f_32,
      annotations: f_33,
      extensions_reply: f_35,
      third_party_connections: f_36,
      mention_entities: f_37,
      hashtag_entities: f_38,
      cashtag_entities: f_39,
      test_user_status: f_40,
      mute_settings: f_41,
      test_user_info: f_42,
      compliance: f_43,
      extended_account: f_44,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("User");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.created_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("created_at_msec", TType::I64, 2))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.updated_at_msec {
      o_prot.write_field_begin(&TFieldIdentifier::new("updated_at_msec", TType::I64, 3))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.profile {
      o_prot.write_field_begin(&TFieldIdentifier::new("profile", TType::Struct, 5))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.profile_design {
      o_prot.write_field_begin(&TFieldIdentifier::new("profile_design", TType::Struct, 6))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.account {
      o_prot.write_field_begin(&TFieldIdentifier::new("account", TType::Struct, 7))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.sleep {
      o_prot.write_field_begin(&TFieldIdentifier::new("sleep", TType::Struct, 8))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.safety {
      o_prot.write_field_begin(&TFieldIdentifier::new("safety", TType::Struct, 9))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.counts {
      o_prot.write_field_begin(&TFieldIdentifier::new("counts", TType::Struct, 10))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.notification {
      o_prot.write_field_begin(&TFieldIdentifier::new("notification", TType::Struct, 11))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.roles {
      o_prot.write_field_begin(&TFieldIdentifier::new("roles", TType::Struct, 12))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.prompts {
      o_prot.write_field_begin(&TFieldIdentifier::new("prompts", TType::Struct, 13))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.view {
      o_prot.write_field_begin(&TFieldIdentifier::new("view", TType::Struct, 14))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.takedowns {
      o_prot.write_field_begin(&TFieldIdentifier::new("takedowns", TType::Struct, 15))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.discoverability {
      o_prot.write_field_begin(&TFieldIdentifier::new("discoverability", TType::Struct, 16))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.contribution {
      o_prot.write_field_begin(&TFieldIdentifier::new("contribution", TType::Struct, 17))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.url_entities {
      o_prot.write_field_begin(&TFieldIdentifier::new("url_entities", TType::Struct, 18))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.saved_searches {
      o_prot.write_field_begin(&TFieldIdentifier::new("saved_searches", TType::Struct, 20))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.storage_state {
      o_prot.write_field_begin(&TFieldIdentifier::new("storage_state", TType::I32, 21))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.facebook_connections {
      o_prot.write_field_begin(&TFieldIdentifier::new("facebook_connections", TType::Struct, 22))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.devices {
      o_prot.write_field_begin(&TFieldIdentifier::new("devices", TType::Struct, 23))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.public_location {
      o_prot.write_field_begin(&TFieldIdentifier::new("public_location", TType::Struct, 24))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.media_view {
      o_prot.write_field_begin(&TFieldIdentifier::new("media_view", TType::Struct, 25))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.dm_view {
      o_prot.write_field_begin(&TFieldIdentifier::new("dm_view", TType::Struct, 26))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.perspective {
      o_prot.write_field_begin(&TFieldIdentifier::new("perspective", TType::Struct, 27))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.profile_location {
      o_prot.write_field_begin(&TFieldIdentifier::new("profile_location", TType::Struct, 28))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.derived {
      o_prot.write_field_begin(&TFieldIdentifier::new("derived", TType::Struct, 29))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.labels {
      o_prot.write_field_begin(&TFieldIdentifier::new("labels", TType::Struct, 30))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.advertiser_account {
      o_prot.write_field_begin(&TFieldIdentifier::new("advertiser_account", TType::Struct, 31))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.extended_profile {
      o_prot.write_field_begin(&TFieldIdentifier::new("extended_profile", TType::Struct, 32))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.annotations {
      o_prot.write_field_begin(&TFieldIdentifier::new("annotations", TType::Struct, 33))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.extensions_reply {
      o_prot.write_field_begin(&TFieldIdentifier::new("extensions_reply", TType::String, 35))?;
      o_prot.write_bytes(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.third_party_connections {
      o_prot.write_field_begin(&TFieldIdentifier::new("third_party_connections", TType::Struct, 36))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.mention_entities {
      o_prot.write_field_begin(&TFieldIdentifier::new("mention_entities", TType::Struct, 37))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.hashtag_entities {
      o_prot.write_field_begin(&TFieldIdentifier::new("hashtag_entities", TType::Struct, 38))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.cashtag_entities {
      o_prot.write_field_begin(&TFieldIdentifier::new("cashtag_entities", TType::Struct, 39))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.test_user_status {
      o_prot.write_field_begin(&TFieldIdentifier::new("test_user_status", TType::I32, 40))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.mute_settings {
      o_prot.write_field_begin(&TFieldIdentifier::new("mute_settings", TType::Struct, 41))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.test_user_info {
      o_prot.write_field_begin(&TFieldIdentifier::new("test_user_info", TType::Struct, 42))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.compliance {
      o_prot.write_field_begin(&TFieldIdentifier::new("compliance", TType::Struct, 43))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.extended_account {
      o_prot.write_field_begin(&TFieldIdentifier::new("extended_account", TType::Struct, 44))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

