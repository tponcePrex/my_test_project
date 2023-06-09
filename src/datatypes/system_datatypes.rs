use bucketizer::Ranger;
use mysql_common::chrono;
use mysql_common::rust_decimal::Decimal;
use crate::data::queries::AccountChargesData;

pub type AccountIdType = u32;
pub type ProductIdType =  u16;
pub type BlockIdType = u8;
pub type FraudGroupsId = u16;
pub type AffinityGroupIdType = u16;
pub type WalletIdType = u32;
pub type CurrenciesIdType = u16;
pub type AccountParameterIdType = u16;
pub type ParameterValueInteger = i64;
pub type ParameterValueDecimal = Decimal;
pub type ParameterValueDate = chrono::NaiveDate;
pub type ParameterValueDateTime = chrono::NaiveDateTime;
pub type ParameterValueRange = Ranger<f64>;
pub type AccountsChargesData = Vec<AccountChargesData>;
