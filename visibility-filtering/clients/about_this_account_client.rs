use crate::clients::read_fields;
use anyhow::bail;
use thrift::protocol::{TInputProtocol, TOutputProtocol, TSerializable, TType};
use xai_strato::strato_thrift::{strato_decode, StratoResult};
use xai_strato::StratoGrpc;

const TFE_TOP_COUNTRY_COLUMN: &str = "about_this_account/tfe_top_country.User";

pub struct AboutThisAccountClient {
    grpc: StratoGrpc,
}

impl AboutThisAccountClient {
    pub fn new(grpc: StratoGrpc) -> Self {
        Self { grpc }
    }

    pub async fn tfe_top_country(&self, user_id: u64) -> anyhow::Result<Option<String>> {
        let bytes = self
            .grpc
            .call(
                TFE_TOP_COUNTRY_COLUMN,
                "fetch",
                vec![xai_strato::encode(&(user_id.cast_signed(), ()))],
                None,
            )
            .await?;
        match strato_decode::<UserTfeTopCountry>(&bytes)? {
            StratoResult::Ok { value, .. } => Ok(value.and_then(|row| row.weighted_top_country)),
            StratoResult::Err { code, message } => bail!("Strato error (code {code}): {message}"),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct UserTfeTopCountry {
    weighted_top_country: Option<String>,
}

const WEIGHTED_TOP_COUNTRY: i16 = 2;
const COUNTRY_ID: i16 = 1;

impl TSerializable for UserTfeTopCountry {
    fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Self> {
        let mut weighted_top_country = None;
        read_fields(i_prot, |i_prot, field| {
            if field.id != Some(WEIGHTED_TOP_COUNTRY) || field.field_type != TType::Struct {
                return Ok(false);
            }
            read_fields(i_prot, |i_prot, field| {
                if field.id != Some(COUNTRY_ID) || field.field_type != TType::String {
                    return Ok(false);
                }
                weighted_top_country = Some(i_prot.read_string()?);
                Ok(true)
            })?;
            Ok(true)
        })?;
        Ok(Self {
            weighted_top_country,
        })
    }

    fn write_to_out_protocol(&self, _o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
        Err(thrift::new_protocol_error(
            thrift::ProtocolErrorKind::NotImplemented,
            "UserTfeTopCountry is decode-only",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::strato_answer::Thrift::{Bool, Str, Struct, I64};
    use thrift::protocol::TBinaryInputProtocol;

    fn decode_row(details: &[(i16, &str)]) -> UserTfeTopCountry {
        let details = details.iter().map(|&(id, country_id)| {
            let country = Struct(vec![(1, Str(country_id.to_owned())), (2, Bool(true))]);
            (id, country)
        });
        let fields = [(1, I64(7))]
            .into_iter()
            .chain(details)
            .chain([(5, I64(1))]);
        let bytes = Struct(fields.collect()).bytes();
        let mut i_prot = TBinaryInputProtocol::new(&bytes[..], false);
        UserTfeTopCountry::read_from_in_protocol(&mut i_prot).unwrap()
    }

    #[test]
    fn decodes_only_the_weighted_top_country() {
        let full = decode_row(&[(3, "FR"), (2, "nz"), (4, "DE")]);
        assert_eq!(full.weighted_top_country.as_deref(), Some("nz"));
        assert_eq!(decode_row(&[(3, "FR")]), UserTfeTopCountry::default());
    }
}
