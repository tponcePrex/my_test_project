#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_must_use)]
use crate::data::db_conn::init_db_conn;
use crate::data::queries::{get_json_from_db, get_account_charges_data, get_transactions_confirmed};

mod data;
mod utils;
mod datatypes;

#[actix_rt::main]
async fn main() {

    let mut conn = init_db_conn().await.unwrap();

    //get_json_from_db(&mut conn).await;
    //get_account_charges_data(&mut conn).await;
    get_transactions_confirmed(&mut conn).await;
}
