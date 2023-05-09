#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_must_use)]
use crate::data::db_conn::init_db_conn;
use crate::data::queries::{get_json_from_db, get_account_charges_data, get_currencies, get_transaction_codes, get_last_statements};
use crate::data::queries_transactions_confirmed::get_transactions_confirmed;
use std::env;
use mysql_common::{rust_decimal::Decimal, bigdecimal03::Zero};

mod data;
mod utils;
mod datatypes;

#[actix_rt::main]
async fn main() {

    let mut args: Vec<String> = env::args().collect();
    let env_var = Decimal::zero() - Decimal::new(
        args.pop().unwrap_or("0".to_string()).parse::<i64>().unwrap_or(0),
        0
    );

    let mut conn = init_db_conn().await.unwrap();

    //get_json_from_db(&mut conn).await;
    //get_account_charges_data(&mut conn).await;
    //get_transactions_confirmed(&mut conn, env_var).await;
    //get_currencies(&mut conn).await;
    //get_transaction_codes(&mut conn).await;
     get_last_statements(&mut conn).await;

}
