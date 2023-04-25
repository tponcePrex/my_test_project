
use std::process::exit;
use logger::ErrorTypes;
use mysql_async::{Conn, Transaction};
use mysql_async::prelude::Queryable;
use mysql_async::prelude::FromRow;
use mysql_common::chrono;
use mysql_common::chrono::NaiveDate;
use mysql_common::row::convert::FromRowError;
use mysql_common::row::Row;
use mysql_common::rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use crate::datatypes::structs::{Account, Wallet};
use crate::datatypes::system_datatypes::{AccountIdType, ProductIdType, TransactionCodeType, TransactionsIdType, WalletIdType};
use crate::{extract_bool, extract_decimal, extract_decimal_opt, extract_value, new_error};
use crate::utils::MyResult;

////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   STRUCTS   ////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
struct Pids {
    pub pid0: u64,
    pub pid1: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Processes {
    pub process2: u64,
    pub process3: u64,
    pub process4: u64,
    pub process5: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FullQuery {
    products_id: ProductIdType,
    pids_json: Option<Pids>,
    processes_json: Option<Processes>,
    created_at: String,
    updated_at: String
}

#[derive(Debug)]
pub struct AccountChargesData {
    accounts: Account,
    account_statement: Option<AccountStatements>
}

#[derive(Debug)]
pub struct AccountStatements {
    accounts_id: AccountIdType,
    balances_date: chrono::NaiveDate
}

#[derive(Debug)]
pub struct TransactionConfirmed {
    pub(super) id: TransactionsIdType,
    pub(super) wallet: Wallet,
    pub(super) balances_date: NaiveDate,
    pub(super) transaction_code: TransactionCodeType,
    pub(super) debit_credit: i8,
    pub(super) amount: Decimal,
    pub(super) authorization_amount: Option<Decimal>,
    pub(super) part: usize,
    pub(super) reference: usize,
    pub(super) is_arbitration: bool,
    pub(super) transactions_pending_id: Option<TransactionsIdType>,
    pub(super) installment_number: u8,
    pub(super) installments_count: u8
}

////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////   IMPLEMENTATIONS   ////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

impl FromRow for FullQuery {
    fn from_row(row: Row) -> Self where Self: Sized {

        let pids_json = extract_value!(row, "pids", "products_statements_configurations", Option<String>);
        let processes_json = extract_value!(row, "processes", "products_statements_configurations", Option<String>);

        FullQuery {
            products_id: extract_value!(row, "products_ID", "products_statements_configurations", ProductIdType),
            pids_json: serde_json::from_str(
                &{if let Some(..) = pids_json {pids_json.unwrap()}
                    else {"".to_string()}}
                ).unwrap_or(None),
            processes_json: serde_json::from_str(
                &{if let Some(..) = processes_json {processes_json.unwrap()}
                    else {"".to_string()}}
                ).unwrap_or(None),
            created_at: extract_value!(row, "created_at", "products_statements_configurations", String),
            updated_at: extract_value!(row, "updated_at", "products_statements_configurations", String)
        }
    }

    fn from_row_opt(_row: Row) -> Result<Self, FromRowError> where Self: Sized { unimplemented!() }
}

impl FromRow for AccountStatements {
    fn from_row(row: Row) -> Self where Self: Sized {
        AccountStatements {
            accounts_id: extract_value!(row, "accounts_id", "account_statements", AccountIdType),
            balances_date: extract_value!(row, "balances_date", "account_statements", chrono::NaiveDate)
        }
    }

    fn from_row_opt(_row: Row) -> Result<Self, FromRowError> where Self: Sized { unimplemented!() }
}

impl FromRow for TransactionConfirmed {
    fn from_row(row: Row) -> Self where Self: Sized {
        TransactionConfirmed {
            id: extract_value!(row, "ID", "TransactionConfirmed"),
            wallet: Wallet::ghost_with_id(
                extract_value!(row, "wallets_ID", "TransactionConfirmed"),
                extract_value!(row, "currencies_ID", "TransactionConfirmed")
            ),
            balances_date: extract_value!(row, "balances_date", "TransactionConfirmed"),
            transaction_code: extract_value!(row, "transaction_codes_ID", "TransactionConfirmed"),
            transactions_pending_id: extract_value!(row, "transactions_pending_ID", "TransactionConfirmed"),
            amount: extract_decimal!(row, "amount", "TransactionConfirmed"),
            authorization_amount: extract_decimal_opt!(row, "authorization_amount", "TransactionConfirmed"),
            part: extract_value!(row, "part", "TransactionConfirmed"),
            reference: extract_value!(row, "part_reference", "TransactionConfirmed"),
            is_arbitration: extract_bool!(row, "is_arbitration", "TransactionConfirmed"),
            debit_credit: extract_value!(row, "debit_credit", "TransactionConfirmed"),
            installment_number: extract_value!(row, "installment_number", "TransactionConfirmed"),
            installments_count: extract_value!(row, "installments_count", "TransactionConfirmed"),
        }
    }

    fn from_row_opt(_row: Row) -> Result<Self, FromRowError> where Self: Sized { unimplemented!() }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   QUERIES   ////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn get_account_charges_data(conn: &mut Conn) {
    let accounts = conn.query_first::<Account, _>(
        "SELECT * FROM accounts"
    ).await.unwrap_or(None);

    let accounts_statemens = conn.query_first::<AccountStatements, _>(
        "SELECT * FROM account_statements"
    ).await.unwrap_or(None);

    if accounts.is_some() { println!("{:?}", accounts.unwrap()) }
    if accounts_statemens.is_some() { println!("{:?}", accounts_statemens.unwrap())}

}

pub async fn get_json_from_db(conn: &mut Conn) {

    let json_vec = conn.query::<FullQuery,_>(
        "SELECT * FROM products_statements_configurations"
    ).await.unwrap();

    for json in json_vec {
        // if json.is_none() {
        //     println!("json was none");
        //     exit(0)
        // } else {
        //     println!("{:?}", json.unwrap());
        // }
        println!("{:?}", json);
    }
}

pub async fn get_transactions_confirmed(conn: &mut Conn) -> MyResult<()>{

    let stmt = format!("SELECT * FROM transactions_confirmed WHERE ID = {}", 1);
    let result = conn.query_first::<TransactionConfirmed, _>(stmt)
        .await
        .map_err(|e| {
            println!("{}", e.to_string());
            new_error!(e.to_string(), ErrorTypes::DbConn)
        })?;

    println!("{:?}", result.unwrap());

    Ok(())
}