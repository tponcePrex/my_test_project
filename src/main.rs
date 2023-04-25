#[allow(unused_imports)]
use crate::data::db_conn::init_db_conn;
#[allow(unused_imports)]
use crate::data::queries::{
    select_json_from_db,
    get_account_charges_data
};

mod data;
mod utils;
mod datatypes;

#[actix_rt::main]
async fn main() {

    let mut conn = init_db_conn().await.unwrap();

    //select_json_from_db(&mut conn).await;
    get_account_charges_data(&mut conn).await;

}
