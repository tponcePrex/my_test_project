pub mod db_conn;
mod macros;
pub mod queries;
pub mod queries_transactions_confirmed;

use lazy_static::lazy_static;
use mysql_async::{Conn, Pool};
use tokio::sync::RwLock;
use logger::ErrorTypes;
use crate::datatypes::system_codes::MySystemError;
use crate::utils::MyResult;

const MYSQL_DSN: &str = "mysql://root:@127.0.0.1:3306/processor";

lazy_static! {
    static ref DB_POOL: RwLock<Option<Pool>> = RwLock::new(None);
}

pub async fn init_pool() -> MyResult<()> {
    let pool = create_pool().await?;
    let mut w = DB_POOL.write().await;
    *w = Some(pool);
    Ok(())
}

pub async fn create_pool() -> crate::utils::CoreResult<Pool>{
    let opts: mysql_async::Opts = mysql_async::Opts::from_url(MYSQL_DSN)
            .map_err(
                |e| crate::utils::CoreError::new(
                    "data::create_pool()",
                    e.to_string(),
                    ErrorTypes::DbPool
                )
            )?;
    let opts = opts;
    Ok(Pool::new(opts))
}

pub async fn get_conn() -> crate::utils::CoreResult<Conn> {
    DB_POOL
        .read().await
        .as_ref()
        .ok_or_else(
            || crate::utils::CoreError::new(
                "data::get_conn",
                "DB_POOL disconnected".to_string(),
                ErrorTypes::DbNoConn
            )
        )?
        .get_conn().await
        .map_err(|e| crate::utils::CoreError::new("data::get_conn",e.to_string(),ErrorTypes::DbNoConn))
}