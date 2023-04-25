use std::process::exit;
use mysql_async::Conn;
use crate::data;
use crate::data::get_conn;
use crate::utils::MyResult;

pub async fn init_db_conn () -> MyResult<Conn> {
    data::init_pool().await.unwrap_or_else(|e| {
        println!("Failed to initiate db pool: {}", e);
        exit(0)
    });

    let conn = get_conn().await.unwrap_or_else(|e| {
        println!("Failed to get db conn: {}", e);
        exit(0)
    });

    Ok(conn)
}