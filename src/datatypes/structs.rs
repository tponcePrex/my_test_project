use std::collections::{BTreeMap, HashMap};
use mysql_common::bigdecimal03::Zero;
use mysql_common::chrono::NaiveDate;
use mysql_common::row::convert::{FromRow, FromRowError};
use mysql_common::row::Row;
use mysql_common::rust_decimal::Decimal;
use crate::datatypes::system_datatypes::{AccountIdType, AccountParameterIdType, AffinityGroupIdType, BlockIdType, CurrenciesIdType, FraudGroupsId, ParameterValueDate, ParameterValueDateTime, ParameterValueDecimal, ParameterValueInteger, ParameterValueRange, ProductIdType, TransactionsIdType, WalletIdType};
use serde::{Deserialize, Serialize};
use tokio::io::Interest;
use crate::extract_value;

////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   STRUCTS   ////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

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

//  HashMap de todos los intereses para todas las transacciones linkeado a un wallet
//  Contiene el accounts_id, wallet_statements compuesto por un hashmap con todas las transacciones
// de credito para ese wallets id, y el calculo de intereses total

pub struct AccountStatementsResult {
    pub accounts_id: AccountIdType,
    pub wallet_statements: HashMap<WalletIdType, WalletStatementsResult>,
    pub total_interest_for_wallet: TotalInterestForWallet,
}

pub struct WalletStatementsResult {
    pub balance: Decimal,
    pub previous_balance: Decimal,
    pub minimum_payment: Decimal,
    pub total_interests: InterestsForTransactions
}

//  Total de intereses esta compuesto por el total de intereses diario, total de intereses de
// penalty (mora), y la fecha de cierre para la cual se esta calculando esos intereses
pub struct TotalInterestForWallet {
    pub total_daily_interest: Decimal,
    pub total_penalty_interest: Decimal,
    pub statement_day: NaiveDate
}

//  HashMap con key = transaction_id y payload = intereses para esa transaccion
pub type InterestsForTransactions = HashMap<TransactionsIdType, InterestForTransaction>;

//  Datos de intereses para cada transaccion en base a la cantidad de dias
pub struct InterestForTransaction {
    is_transaction_purchase: bool,
    daily_interest_rate: Decimal,
    total_daily_interest: Decimal,
    is_client_in_penalty: bool,
    penalty_interest_rate: Decimal,
    total_penalty_interest: Decimal,
    balances_date: NaiveDate
}

impl InterestForTransaction{
    pub fn new() -> InterestForTransaction {
        InterestForTransaction{
            is_transaction_purchase: false,
            daily_interest_rate: Decimal::zero(),
            total_daily_interest: Decimal::zero(),
            is_client_in_penalty: false,
            penalty_interest_rate: Decimal::zero(),
            total_penalty_interest: Decimal::zero(),
            balances_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
        }
    }
    pub fn get_is_transaction_purchase(&self) -> bool { self.is_transaction_purchase }
    pub fn set_is_transaction_purchase(&mut self, is_transaction_purchase: bool) {
        self.is_transaction_purchase = is_transaction_purchase
    }

    pub fn get_daily_interest_rate(&self) -> Decimal { self.daily_interest_rate }
    pub fn set_daily_interest_rate(&mut self, daily_interest_rate: Decimal) {
        self.daily_interest_rate = daily_interest_rate
    }

    pub fn get_total_daily_interest(&self) -> Decimal { self.total_daily_interest }
    pub fn set_total_daily_interest(&mut self, total_daily_interest: Decimal) {
        self.total_daily_interest = total_daily_interest
    }

    pub fn get_is_client_in_penalty(&self) -> bool { self.is_client_in_penalty }
    pub fn set_is_client_in_penalty(&mut self, is_client_in_penalty: bool) {
        self.is_client_in_penalty = is_client_in_penalty
    }

    pub fn get_penalty_interest_rate(&self) -> Decimal { self.penalty_interest_rate }
    pub fn set_penalty_interest_rate(&mut self, penalty_interest_rate: Decimal) {
        self.penalty_interest_rate = penalty_interest_rate
    }

    pub fn get_total_penalty_interest(&self) -> Decimal { self.total_penalty_interest }
    pub fn set_total_penalty_rate(&mut self, total_penalty_rate: Decimal) {
        self.total_penalty_rate = total_penalty_rate
    }

    pub fn get_balances_date(&self) -> NaiveDate { self.balances_date }
    pub fn set_balances_date(&mut self, balances_date: NaiveDate) {
        self.balances_date = balances_date
    }

}

////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////   IMPLEMENTATIONS   ////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

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

impl Wallet {
    //  Creates a ghost wallet from the wallets_id and currencies id type
    pub fn ghost_with_id(id: WalletIdType, currencies_id: CurrenciesIdType) -> Self {
        Self {
            id,
            currencies_id,
            charge_priority: 0,
        }
    }
}