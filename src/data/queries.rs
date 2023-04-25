#[allow(unused_imports)]
use std::process::exit;
use mysql_async::Conn;
use mysql_async::prelude::Queryable;
use mysql_async::prelude::FromRow;
use mysql_common::chrono;
use mysql_common::row::convert::FromRowError;
use mysql_common::row::Row;
use serde::{Serialize, Deserialize};
use crate::datatypes::structs::{Account};
use crate::datatypes::system_datatypes::{AccountIdType, ProductIdType};
use crate::extract_value;

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

#[allow(dead_code)]
pub async fn select_json_from_db(conn: &mut Conn) {

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

pub struct AccountChargesData {
    accounts: Account,
    account_statement: Option<AccountStatements>
}

#[derive(Debug)]
pub struct AccountStatements {
    accounts_id: AccountIdType,
    balances_date: chrono::NaiveDate
}

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

impl FromRow for AccountStatements {
    fn from_row(row: Row) -> Self where Self: Sized {
        AccountStatements {
            accounts_id: extract_value!(row, "accounts_id", "account_statements", AccountIdType),
            balances_date: extract_value!(row, "balances_date", "account_statements", chrono::NaiveDate)
        }
    }

    fn from_row_opt(row: Row) -> Result<Self, FromRowError> where Self: Sized { unimplemented!() }
}