use std::collections::{BTreeMap, HashMap};
use mysql_common::bigdecimal03::Zero;
use mysql_common::chrono::{NaiveDate, NaiveDateTime};
use mysql_common::row::convert::{FromRow, FromRowError};
use mysql_common::row::Row;
use mysql_common::rust_decimal::Decimal;
use crate::datatypes::system_datatypes::{AccountIdType, AccountParameterIdType, AffinityGroupIdType, BlockIdType, CurrenciesIdType, FraudGroupsId, ParameterValueDate, ParameterValueDateTime, ParameterValueDecimal, ParameterValueInteger, ParameterValueRange, ProductIdType, TransactionsIdType, WalletIdType};
use serde::{Deserialize, Serialize};
use tokio::io::Interest;
use crate::data::queries::TransactionConfirmed;
use crate::extract_value;


pub type AccountsStatementsResult = Vec<AccountStatementsResult>;

/// HashMap that takes a TransactionsIdType as a key and InterestForTransaction for a single
/// transaction as a payload
pub type InterestsForTransactions = HashMap<TransactionsIdType, InterestForTransaction>;

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
    pub id: WalletIdType,
    pub currencies_id: CurrenciesIdType,
    pub charge_priority: i16,
}

pub struct AccountStatementsResult {
    pub accounts_id: AccountIdType,
    pub wallet_statements: HashMap<WalletIdType, WalletStatementsResult>
}

/// Struct that contains all relevant data for interests calculations, balances, transactions
/// details and dates and minimum payment values
pub struct WalletStatementsResult {
    ///  Current balance for the linked wallets_id
    balance: Decimal,
    /// Previous balance for the linked wallets_id
    previous_balance: Decimal,
    /// Minimum payment for the current statement
    minimum_payment: Decimal,
    /// Sum of all the purchases for the calculated period
    total_purchases: Decimal,
    /// Sum of all the payments for the calculated period
    total_payments: Decimal,
    /// Total financial interests for the calculated period
    total_daily_interest: Decimal,
    /// Total penalty interests for the calculated period
    total_penalty_interest: Decimal,
    /// Statement day to calculate the (potential) interests
    statement_day: NaiveDate,
    /// Total interests, sum of all the interests
    transactions_details: InterestsForTransactions,
}

/// Struct that contains interests details for a single transaction
#[derive(Debug, Clone, Copy)]
pub struct InterestForTransaction {
    /// Transaction amount
    transaction_amount: Decimal,
    /// Transaction amount relative to the payments made by the client. Ex: if purchase is fully covered by payment, effective_transaction_amount will be zero
    effective_transaction_amount: Decimal,
    /// True if transaction was a purchase, false if it was a payment
    is_transaction_purchase: bool,
    /// Financial Daily interest rate determined by the type of purchased
    daily_interest_rate: Decimal,
    /// Total financial daily interest for the purchase
    total_daily_interest: Decimal,
    /// True if client is in penalty, false if they're not
    //  TODO: RENOMBRAR A DEFAULT
    is_client_in_penalty: bool,
    /// Penalty interest rate
    penalty_interest_rate: Decimal,
    /// Total penalty interest for the purchase transaction
    total_penalty_interest: Decimal,
    /// Date when the transaction took place
    balances_date: NaiveDate
}

#[derive(Clone, Debug)]
pub struct AccountData {
    pub id: AccountIdType,
    pub last_account_statement: Option<LastAccountStatement>,
    pub wallets_data: HashMap<WalletIdType, (Wallet, Option<LastWalletStatement>)>, // TODO Implement this
}

#[derive(Debug, Clone)]
pub struct LastAccountStatement {
    pub accounts_id: AccountIdType,
    pub balances_date: NaiveDate,
    pub wallets_id: WalletIdType,
    pub balance: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}

#[derive(Debug, Clone)]
pub struct LastWalletStatement {
    pub wallet_id: WalletIdType,
    pub balances_date: NaiveDate,
    pub accounts_id: AccountIdType,
    pub balance: Decimal,
    pub minimum_payment: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}

#[derive(Debug, Clone)]
pub struct WalletData {
    pub wallets_id: WalletIdType,
    pub currencies_id: CurrenciesIdType,
    pub charge_priority: i16,
    pub balances_date: NaiveDate,
    pub accounts_id: AccountIdType,
    pub balance: Decimal,
    pub minimum_payment: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}

////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   ENUMS   //////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterData {
    Integer(ParameterValueInteger),
    Decimal(ParameterValueDecimal),
    Date(ParameterValueDate),
    Datetime(ParameterValueDateTime),
    Range(ParameterValueRange),
    Unset,
}

#[derive(Debug, PartialEq)]
pub enum ClientBalanceCaseType{
    UpToDate,
    MinimumCovered,
    Penalty,
    NoPayment,
    TwoDaysGrace,
    Undetermined
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

    pub fn new_empty() -> Self{
        Self{
            id: 0,
            currencies_id: 0,
            charge_priority: 0
        }
    }
}

impl WalletStatementsResult {
    /// Creates a new WalletStatementsResult struct.
    ///
    ///Parameters: InterestForTransactions HashMap, statement_day with NaiveDate format,
    /// and previous_balance with Decimal format
    ///
    /// Returns: a new instance of WalletStatementsResult
    pub fn new(
        interests_for_transaction: InterestsForTransactions,
        statement_day: NaiveDate,
        previous_balance: Decimal
    ) -> Self {
        WalletStatementsResult {
            balance: Decimal::zero(),
            previous_balance,
            minimum_payment: Decimal::zero(),
            total_purchases: Decimal::zero(),
            total_payments: Decimal::zero(),
            total_daily_interest: Decimal::zero(),
            total_penalty_interest: Decimal::zero(),
            statement_day,
            transactions_details: interests_for_transaction,
        }
    }

    //  Getters and setters
    pub fn get_balance(&self) -> Decimal { self.balance }
    pub fn set_balance(&mut self, balance: Decimal) {
        self.balance = balance
    }

    pub fn get_previous_balance(&self) -> Decimal { self.previous_balance }
    pub fn set_previous_balance(&mut self, previous_balance: Decimal) {
        self.previous_balance = previous_balance
    }

    pub fn get_minimum_payment(&self) -> Decimal { self.minimum_payment }
    pub fn set_minimum_payment(&mut self, minimum_payment: Decimal) {
        self.minimum_payment = minimum_payment
    }

    pub fn get_total_purchases(&self) -> Decimal { self.total_purchases }
    pub fn set_total_purchases(&mut self, total_purchases: Decimal) {
        self.total_purchases = total_purchases
    }

    pub fn get_total_payments(&self) -> Decimal { self.total_payments }
    pub fn set_total_payments(&mut self, total_payments: Decimal) {
        self.total_payments = total_payments
    }

    pub fn get_transactions_details(&self) -> &InterestsForTransactions { &self.transactions_details }
    pub fn set_transactions_details(&mut self, total_interests: InterestsForTransactions) {
        self.transactions_details = total_interests
    }

    //  Direct getters and setters for WalletStatementsResult sub-members
    pub fn get_total_daily_interest(&self) -> Decimal {
        self.total_daily_interest
    }
    pub fn set_total_daily_interest(&mut self, total_daily_interest: Decimal) {
        self.total_daily_interest = total_daily_interest
    }

    pub fn get_total_penalty_interest(&self) -> Decimal {
        self.total_penalty_interest
    }
    pub fn set_total_penalty_interest(&mut self, total_penalty_interest: Decimal) {
        self.total_penalty_interest = total_penalty_interest
    }

    pub fn get_statement_day(&self) -> NaiveDate {
        self.statement_day
    }
    pub fn set_statement_day(&mut self, statement_day: NaiveDate) {
        self.statement_day = statement_day
    }
}

impl InterestForTransaction{
    /// Creates a new InterestForTransaction struct.
    ///
    ///Parameters: TransactionConfirmed struct, daily_interest_rate in Decimal format,
    /// penalty_interest_rate with Decimal format and client_case ClientBalanceCaseType enum value
    ///
    /// Returns: a new instance of InterestForTransaction
    pub fn new(
        s: &TransactionConfirmed,
        daily_interest_rate: &Decimal,
        penalty_interest_rate: &Decimal,
        client_case: &ClientBalanceCaseType
    ) -> InterestForTransaction {
        InterestForTransaction{
            transaction_amount: s.amount,
            effective_transaction_amount: s.amount,
            is_transaction_purchase: s.debit_credit != -1,
            daily_interest_rate: *daily_interest_rate,
            total_daily_interest: Decimal::zero(),
            is_client_in_penalty: {
                *client_case == ClientBalanceCaseType::Penalty
            },
            penalty_interest_rate: *penalty_interest_rate,
            total_penalty_interest: Decimal::zero(),
            balances_date: s.balances_date
        }
    }
    //  Getters and setters for the InterestForTransaction private members
    pub fn get_transaction_amount(&self) -> Decimal { self.transaction_amount }
    pub fn set_transaction_amount(&mut self, transaction_amount: Decimal) {
        self.transaction_amount = transaction_amount
    }

    pub fn get_effective_transaction_amount(&self) -> Decimal { self.effective_transaction_amount }
    pub fn set_effective_transaction_amount(&mut self, effective_transaction_amount: Decimal) {
        self.effective_transaction_amount = effective_transaction_amount
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
    pub fn set_total_penalty_interest(&mut self, total_penalty_interest: Decimal) {
        self.total_penalty_interest = total_penalty_interest
    }

    pub fn get_balances_date(&self) -> NaiveDate { self.balances_date }
    pub fn set_balances_date(&mut self, balances_date: NaiveDate) {
        self.balances_date = balances_date
    }

    /// Calculates the daily interest rate for a single transaction.
    ///
    ///Parameters: payments value in Decimal format, client_case ClientBalanceCaseType enum value,
    /// statement_date in NaiveDate format
    ///
    /// Returns: nothing. Parameters that need values modification are received as mutable
    /// references, including the InterestForTransaction calling struct itself
    pub fn calculate_daily_interest_rate(
        &mut self,
        payments: &mut Decimal,
        client_case: &ClientBalanceCaseType,
        statement_day: &NaiveDate
    ) {

        //  Payments es negativo, por eso se suma siempre
        //  Decrease payment value to determine when to start applying interests, or not if purchases are covered
        if payments.abs() >= self.get_transaction_amount() {
            //  Decreasing the value of payments and setting the effective_transaction_amount to zero
            *payments += self.get_effective_transaction_amount();
            self.set_effective_transaction_amount(Decimal::zero());
            println!("Purchase covered");

        } else if ( payments.abs() < self.get_effective_transaction_amount() )
            && ( payments.abs() > Decimal::zero() ) {
            //  Decreasing the value of effective transaction amount, and setting payments to zero
            let mut effective_transaction_amount = self.get_effective_transaction_amount();
            effective_transaction_amount += *payments;
            self.set_effective_transaction_amount(effective_transaction_amount);
            *payments = Decimal::zero();
            println!("Effective debt: {}", effective_transaction_amount);

        } else {
            //  PLACEHOLDER TO UNDERSTAND LOGIC
            //  payments is zero, meaning there was no payment, or there's no payment amount left to cover any more purchases
            //  Making sure payments is zero
            *payments = Decimal::zero();
        }

        //  If the effective value for the transaction is greater than zero (debt not cancelled), calculate interests
        if self.get_effective_transaction_amount() > Decimal::zero() {
            //  Calculate amount of days from statement_day to transaction date
            let days_amount = Decimal::new((*statement_day - self.get_balances_date()).num_days(), 0);
            //  Calculate interest according to the client's balance case
            match client_case {
                ClientBalanceCaseType::UpToDate => {
                    //  If client is up to date, the interests will both be zero
                    self.set_total_daily_interest(Decimal::zero());
                    self.set_total_penalty_interest(Decimal::zero());
                },
                ClientBalanceCaseType::NoPayment => {
                    //  If no payment was registered, both financial and penalty interests apply
                    self.set_total_daily_interest(
                        self.get_effective_transaction_amount() * days_amount * self.get_daily_interest_rate()
                    );
                    self.set_total_penalty_interest(
                        self.get_effective_transaction_amount() * days_amount * self.get_penalty_interest_rate()
                    );
                },
                ClientBalanceCaseType::MinimumCovered => {
                    //  If client has minimum payment covered, only financial interests apply
                    self.set_total_daily_interest(
                        self.get_effective_transaction_amount() * days_amount * self.get_daily_interest_rate()
                    );
                },
                ClientBalanceCaseType::Penalty => {
                    //  If client is on penalty, both financial and penalty interests apply
                    self.set_total_daily_interest(
                        self.get_effective_transaction_amount() * days_amount * self.get_daily_interest_rate()
                    );
                    self.set_total_penalty_interest(
                        self.get_effective_transaction_amount() * days_amount * self.get_penalty_interest_rate()
                    );
                },
                ClientBalanceCaseType::TwoDaysGrace => {
                    //  @TODO: develop two days grace
                },
                ClientBalanceCaseType::Undetermined => {
                    //  Nothing here, case is undetermined, shouldn't come to this
                }
            }
            //  Otherwise, set interests values to zero
        } else {
            self.set_total_daily_interest(Decimal::zero());
            self.set_total_penalty_interest(Decimal::zero());
        }
    }
}

impl FromRow for WalletData {
    fn from_row(row: Row) -> Self where Self: Sized {
        Self{
            wallets_id: extract_value!(row, "ID", "WalletData", WalletIdType),
            currencies_id: extract_value!(row, "currencies_ID", "WalletData", CurrenciesIdType),
            charge_priority: extract_value!(row, "charge_priority", "WalletData", i16),
            balances_date: extract_value!(row, "balances_date", "WalletData", NaiveDate),
            accounts_id: extract_value!(row, "accounts_ID", "WalletData", AccountIdType),
            balance: extract_value!(row, "balance", "WalletData", Decimal),
            minimum_payment: extract_value!(row, "minimum_payment", "WalletData", Decimal),
            created_at: extract_value!(row, "created_at", "WalletData", NaiveDateTime),
            updated_at: extract_value!(row, "updated_at", "WalletData", NaiveDateTime)
        }
    }

    fn from_row_opt(_row: Row) -> Result<Self, FromRowError> where Self: Sized { unimplemented!() }
}

impl LastWalletStatement {

    pub fn new_empty() -> LastWalletStatement {
        LastWalletStatement {
            wallet_id: 0,
            balances_date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            accounts_id: 0,
            balance: Decimal::zero(),
            minimum_payment: Decimal::zero(),
            created_at: NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            updated_at: NaiveDateTime::from_timestamp_opt(0, 0).unwrap()
        }
    }

    pub fn get_statement_day(&self) -> NaiveDateTime {
        self.created_at
    }
}


impl LastAccountStatement {

    pub fn new_empty() -> LastAccountStatement {
        LastAccountStatement {
            accounts_id: 0,
            balances_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            wallets_id: 0,
            balance: Decimal::zero(),
            created_at: NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            updated_at: NaiveDateTime::from_timestamp_opt(0, 0).unwrap()
        }
    }

    pub fn get_balances_date(&self) -> NaiveDate { self.balances_date }
}

impl FromRow for LastWalletStatement {
    fn from_row(row: Row) -> Self where Self: Sized {
        LastWalletStatement {
            wallet_id: extract_value!(row, "wallets_ID", "wallet_statements", WalletIdType),
            balances_date: extract_value!(row, "balances_date", "wallet_statements", NaiveDate),
            accounts_id: extract_value!(row, "accounts_ID", "wallet_statements", AccountIdType),
            balance: extract_value!(row, "balance", "wallet_statements", Decimal),
            minimum_payment: extract_value!(row, "minimum_payment", "wallet_statements", Decimal),
            created_at: extract_value!(row, "created_at", "wallet_statements", NaiveDateTime),
            updated_at: extract_value!(row, "updated_at", "wallet_statements", NaiveDateTime)
        }
    }

    fn from_row_opt(_row: Row) -> Result<Self, FromRowError> where Self: Sized { unimplemented!() }
}

impl FromRow for LastAccountStatement {
    fn from_row(row: Row) -> Self where Self: Sized {
        LastAccountStatement {
            accounts_id: extract_value!(row, "accounts_id", "account_statements", AccountIdType),
            balances_date: extract_value!(row, "balances_date", "account_statements", NaiveDate),
            wallets_id: extract_value!(row, "wallets_id", "account_statements", WalletIdType),
            balance: extract_value!(row, "balance", "account_statements", Decimal),
            created_at: extract_value!(row, "created_at", "account_statements", NaiveDateTime),
            updated_at: extract_value!(row, "updated_at", "account_statements", NaiveDateTime)
        }
    }

    fn from_row_opt(_row: Row) -> Result<Self, FromRowError> where Self: Sized { unimplemented!() }
}