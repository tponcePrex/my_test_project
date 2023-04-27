use std::collections::{BTreeMap, HashMap};
use mysql_common::bigdecimal03::Zero;
use mysql_common::chrono::NaiveDate;
use mysql_common::row::convert::{FromRow, FromRowError};
use mysql_common::row::Row;
use mysql_common::rust_decimal::Decimal;
use crate::datatypes::system_datatypes::{AccountIdType, AccountParameterIdType, AffinityGroupIdType, BlockIdType, CurrenciesIdType, FraudGroupsId, ParameterValueDate, ParameterValueDateTime, ParameterValueDecimal, ParameterValueInteger, ParameterValueRange, ProductIdType, TransactionsIdType, WalletIdType};
use serde::{Deserialize, Serialize};
use tokio::io::Interest;
use crate::data::queries::TransactionConfirmed;
use crate::data::queries_transactions_confirmed::{calculate_days_amount, ClientBalanceCaseType};
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

pub type AccountsStatementsResult = Vec<AccountStatementsResult>;

pub struct AccountStatementsResult {
    pub accounts_id: AccountIdType,
    pub wallet_statements: HashMap<WalletIdType, WalletStatementsResult>,
    pub total_interest_for_wallet: TotalInterestForWallet,
}

//  Global statements previous and current for the wallet_id
pub struct WalletStatementsResult {
    pub balance: Decimal,
    pub previous_balance: Decimal,
    pub minimum_payment: Decimal,
    pub total_interests: InterestsForTransactions
    //  Incluir TotalInterestForWallet aca y sacar de AccountStatementsResult
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
    // RENOMBRAR A DEFAULT
    is_client_in_penalty: bool,
    /// Penalty interest rate
    penalty_interest_rate: Decimal,
    /// Total penalty interest for the purchase transaction
    total_penalty_interest: Decimal,
    /// Date when the transaction took place
    balances_date: NaiveDate
}

impl InterestForTransaction{
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
            println!("New payments value: {}", payments);

        } else if ( payments.abs() < self.get_effective_transaction_amount() )
            && ( payments.abs() > Decimal::zero() ) {
            //  Decreasing the value of effective transaction amount, and setting payments to zero
            let mut effective_transaction_amount = self.get_effective_transaction_amount();
            effective_transaction_amount += *payments;
            self.set_effective_transaction_amount(effective_transaction_amount);
            *payments = Decimal::zero();
            println!("New payments value: {}", payments);

        } else {
            //  PLACEHOLDER TO UNDERSTAND LOGIC
            //  payments is zero, meaning there was no payment, or there's no payment amount left to cover any more purchases
            //  Making sure payments is zero
            *payments = Decimal::zero();
            println!("Payments value: {}", payments);

        }

        //  If the effective value for the transaction is greater than zero (debt not cancelled), calculate interests
        if self.get_effective_transaction_amount() > Decimal::zero() {
            //  Calculate interest according to the client's balance case
            match client_case {
                ClientBalanceCaseType::UpToDate => {
                    //  If client is up to date, the interests will both be zero
                    self.set_total_daily_interest(Decimal::zero());
                    self.set_total_penalty_interest(Decimal::zero());
                },
                ClientBalanceCaseType::NoPayment => {
                    //  If no payment was registered, both financial and penalty interests apply
                    let days_amount = calculate_days_amount(
                        &self.get_balances_date(),
                        statement_day
                    );

                    self.set_total_daily_interest(
                        self.get_effective_transaction_amount() * days_amount * self.get_daily_interest_rate()
                    );
                    self.set_total_penalty_interest(
                        self.get_effective_transaction_amount() * days_amount * self.get_penalty_interest_rate()
                    );
                },
                ClientBalanceCaseType::MinimumCovered => {
                    //  If client has minimum payment covered, only financial interests apply
                    let days_amount = calculate_days_amount(
                        &self.get_balances_date(),
                        statement_day
                    );

                    self.set_total_daily_interest(
                        self.get_effective_transaction_amount() * days_amount * self.get_daily_interest_rate()
                    );
                },
                ClientBalanceCaseType::Penalty => {
                    //  If client is on penalty, both financial and penalty interests apply
                    let days_amount = calculate_days_amount(
                        &self.get_balances_date(),
                        statement_day
                    );

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