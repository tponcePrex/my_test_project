use std::collections::{BTreeMap, HashMap};
use mysql_common::row::convert::{FromRow, FromRowError};
use mysql_common::row::Row;
use mysql_common::rust_decimal::Decimal;
use crate::datatypes::system_datatypes::{AccountIdType, AccountParameterIdType, AffinityGroupIdType, BlockIdType, CurrenciesIdType, FraudGroupsId, ParameterValueDate, ParameterValueDateTime, ParameterValueDecimal, ParameterValueInteger, ParameterValueRange, ProductIdType, WalletIdType};
use serde::{Deserialize, Serialize};
use crate::extract_value;

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    #[serde(skip_serializing)]
    id: AccountIdType,
    number: AccountIdType,
    products_id: ProductIdType,
    blocks_id: BlockIdType,
    fraud_groups_id: FraudGroupsId,
    affinity_groups_id: AffinityGroupIdType,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    wallets: HashMap<WalletIdType, Wallet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<BTreeMap<AccountParameterIdType, ParameterData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    statement_day: Option<u8>, // this determines if the account has a credit line (if is_some())
    credit_amount: Decimal,
    future_balance_coefficient: f32,
    grace_period_coefficient: f32,
    withdrawal_coefficient: f32,
}

#[derive(Serialize, Debug, Copy, Clone, Default)]
pub struct Wallet {
    id: WalletIdType,
    currencies_id: CurrenciesIdType,
    charge_priority: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterData {
    Integer(ParameterValueInteger),
    Decimal(ParameterValueDecimal),
    Date(ParameterValueDate),
    Datetime(ParameterValueDateTime),
    Range(ParameterValueRange),
    Unset,
}

impl FromRow for Account {
    fn from_row(row: Row) -> Self
        where
            Self: Sized,
    {
        Account {
            id: extract_value!(row, "ID", "Account"),
            number: extract_value!(row, "number", "Account"),
            products_id: extract_value!(row, "products_ID", "Account"),
            blocks_id: extract_value!(row, "blocks_ID", "Account"),
            fraud_groups_id: extract_value!(row, "fraud_groups_ID", "Account"),
            affinity_groups_id: extract_value!(row, "affinity_groups_ID", "Account"),
            wallets: HashMap::with_capacity(12),
            parameters: None,
            statement_day: extract_value!(row, "statement_day", "Account"),
            credit_amount: extract_value!(row, "credit_amount", "Account"),
            future_balance_coefficient: extract_value!(row, "future_balance_coefficient", "Account"),
            grace_period_coefficient: extract_value!(row, "grace_period_coefficient", "Account"),
            withdrawal_coefficient: extract_value!(row, "withdrawal_coefficient", "Account"),
        }
    }

    fn from_row_opt(_row: Row) -> Result<Self, FromRowError>
        where
            Self: Sized,
    {
        unimplemented!()
    }
}