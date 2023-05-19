use std::collections::HashMap;
use std::process::exit;
use logger::ErrorTypes;
use mysql_async::{Conn, Transaction};
use mysql_async::prelude::Queryable;
use mysql_async::prelude::FromRow;
use mysql_common::bigdecimal03::Zero;
use mysql_common::chrono;
use mysql_common::chrono::{Datelike, NaiveDate, NaiveDateTime};
use mysql_common::row::convert::FromRowError;
use mysql_common::row::Row;
use mysql_common::rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use crate::datatypes::structs::{Account, AccountData, InterestForTransaction, InterestsForTransactions, LastAccountStatement, LastWalletStatement, Wallet, WalletData, WalletStatementsResult};
use crate::datatypes::system_datatypes::*;
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
    pub id: TransactionsIdType,
    pub wallet: Wallet,
    pub balances_date: NaiveDate,
    pub transaction_code: TransactionCodeType,
    pub debit_credit: i8,
    pub amount: Decimal,
    pub authorization_amount: Option<Decimal>,
    pub part: usize,
    pub reference: usize,
    pub is_arbitration: bool,
    pub transactions_pending_id: Option<TransactionsIdType>,
    pub installment_number: u8,
    pub installments_count: u8
}

#[derive(Debug)]
pub struct Currencies {
    id: CurrenciesIdType,
    iso_3: Iso3IdType,
    name: String
}

#[derive(Debug)]
pub struct TransactionCodes {
    id: TransactionCodeType,
    transaction_categories_id: TransactionCategoriesIdType,
    name: String,
    description: String
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

impl FromRow for Currencies {
    fn from_row(row: Row) -> Self where Self: Sized {
        Currencies{
            id: extract_value!(row, "ID", "currencies", CurrenciesIdType),
            iso_3: extract_value!(row, "iso_3", "currencies", Iso3IdType),
            name: extract_value!(row, "name", "currencies", String)
        }
    }

    fn from_row_opt(_row: Row) -> Result<Self, FromRowError> where Self: Sized { unimplemented!() }
}

impl FromRow for TransactionCodes {
    fn from_row(row: Row) -> Self where Self: Sized {
        TransactionCodes{
            id: extract_value!(row, "ID", "transaction_codes", TransactionCodeType),
            transaction_categories_id: extract_value!(row, "transaction_categories_ID", "transaction_codes", TransactionCategoriesIdType),
            name: extract_value!(row, "name", "transaction_codes", String),
            description: extract_value!(row, "description", "transaction_codes", String)
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

pub async fn get_currencies(conn: &mut Conn) -> MyResult<()> {
    let stmt = "SELECT * FROM currencies".to_string();

    let currencies_vec = conn.query::<Currencies, _>(stmt)
        .await
        .map_err(|e| new_error!(e.to_string(), ErrorTypes::DbConn))?;

    for currency in currencies_vec {
        if (currency.id == 840) | (currency.id == 604) | (currency.id == 32) {
            println!("{}: {} -> {}", currency.id, currency.iso_3, currency.name);
        }
    }

    Ok(())
}

pub async fn get_transaction_codes(conn: &mut Conn) -> MyResult<()> {

    let stmt = "SELECT * FROM transaction_codes WHERE transaction_categories_ID IN (0,4)".to_string();

    let transaction_codes = conn.query::<TransactionCodes, _>(stmt)
        .await
        .map_err(|e| new_error!(e.to_string(), ErrorTypes::DbConn))?;

    for transaction_code in transaction_codes {
        if transaction_code.transaction_categories_id == 4 {
            println!("{}: {} -> {}", transaction_code.id, transaction_code.name, transaction_code.description);
        }
        if transaction_code.id >= 1000 {
            println!("{}: {} -> {}", transaction_code.id, transaction_code.name, transaction_code.description);
        }
    }

    Ok(())
}

pub struct AccountDataFromDb {
    id: AccountIdType,
    wallet_id: WalletIdType
}

pub async fn get_last_statements(conn: &mut Conn) {

    let mut accounts_from_db = Vec::<AccountDataFromDb>::with_capacity(8);
    accounts_from_db.push(AccountDataFromDb{
        id: 2,
        wallet_id: 1
    });
    accounts_from_db.push(AccountDataFromDb{
        id: 2,
        wallet_id: 2
    });
    accounts_from_db.push(AccountDataFromDb{
        id: 3,
        wallet_id: 3
    });

    //  Creates the accounts_id vector that contains all account ids
    let mut accounts_id = Vec::<AccountIdType>::with_capacity(accounts_from_db.len());

    let mut accounts_string = String::with_capacity(accounts_id.len() * 2);

    let mut accounts_map = HashMap::<AccountIdType, AccountData>::with_capacity(accounts_from_db.len());

    //  Initiates the accounts_db HashMap and forms the string to fetch the statements from db
    init_hash_and_string(
        &accounts_from_db,
        &mut accounts_id,
        &mut accounts_map,
        &mut accounts_string
    );

    //  Selecting all account_statements first, but only the last for each accounts_id
    let acc_stmt = format!("SELECT accounts_id, MAX(balances_date) AS balances_date, \
                    wallets_id, balance, created_at, updated_at FROM account_statements \
                    WHERE accounts_id IN ({}) \
                    GROUP BY accounts_id \
                    ORDER BY accounts_id", accounts_string);

    //  Selects all wallet_statements for each accounts_id, but only the last wallet statement for each wallet
    // let wal_stmt = format!("SELECT wallets_ID, MAX(balances_date) AS balances_date, \
    //                 accounts_ID, balance, minimum_payment, created_at, updated_at FROM wallet_statements \
    //                 WHERE accounts_ID IN ({}) \
    //                 GROUP BY wallets_ID \
    //                 ORDER BY accounts_ID", accounts_string);
    let wal_stmt = format!("SELECT * FROM \
                    ( \
                        SELECT ID, currencies_ID, charge_priority FROM wallets \
                    ) AS wal \
                    inner JOIN \
                    ( \
                        SELECT wallets_ID, MAX(balances_date) AS balances_date, accounts_ID, balance, minimum_payment, created_at, updated_at FROM wallet_statements \
                        WHERE accounts_ID IN ({}) \
                        GROUP BY wallets_id \
                    ) AS stat \
                    ON wal.ID = stat.wallets_ID", accounts_string);

    //  Executes both queries
    let accounts_statements = conn.query::<LastAccountStatement, _>(
        acc_stmt
    ).await.map_err(|e| println!("Esto no sirve numero 1: {}", e)).unwrap();

    let wallets_data = conn.query::<WalletData, _>(
        wal_stmt
    ).await.map_err(|e| println!("Esto no sirve numero 2: {}", e)).unwrap();

    let wallets_data_original = wallets_data.clone();

    fill_hash_map(accounts_statements.clone(), wallets_data.clone(), &mut accounts_map);

    //let result = accounts_map.into_values().collect();















    println!("Accounts statements:\n");
    for statement in accounts_statements {
        println!("{:?}", statement);
    }

    println!("\n\nWallets data:\n");
    for wallet in wallets_data_original {
        println!("{:?}", wallet);
    }

    println!("\n\n");

    for (account, account_data) in accounts_map {
        println!("AccountData: {:?}\n\n", account_data);
        // for (_, (wallet, payload)) in account_data.wallets_data {
        //     println!("Wallet: {:?}", wallet);
        //     println!("Wallet_statement: {:?}", payload.unwrap());
        // }
    }


}

pub async fn insert_test(conn: &mut Conn) {

    let stmt = "INSERT INTO zz_test(col1,col2,col3,col4,col5)\
                    VALUES\
                    (1,2,3,4,5),\
                    (5,2,NULL,3,4),\
                    (5,4,3,2,NULL)".to_string();

    conn.query_drop(
        stmt
    ).await.map_err(|e| println!("Error: {}", e)).unwrap();

}

////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   FUNCTIONS   //////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

fn init_hash_and_string(
    accounts_from_db: &Vec<AccountDataFromDb>,
    accounts_id: &mut Vec<AccountIdType>,
    accounts_map: &mut HashMap<AccountIdType, AccountData>,
    accounts_string: &mut String
) {

    //  Pushes into the accounts_id vector all accounts_id retrieved from db
    for account in accounts_from_db {
        if !accounts_id.contains(&account.id) {
            //  If no account is found for the current accounts_id then no wallets have been inserted either
            //  Then, insert them both into the corresponding HashMaps
            //  Mock capacity value for wallet_statement HashMap
            let mut wallet_empty_map = HashMap::<WalletIdType, (Wallet, Option<LastWalletStatement>)>::with_capacity(3);
            //  Inserting an empty payload into the HashMap with the corresponding wallets_id value
            wallet_empty_map.insert(
                account.wallet_id,
                (Wallet::new_empty(), None)
            );
            //  Inserting the account id and AccountData struct into the accounts_map hash
            accounts_map.insert(
                account.id,
                AccountData{
                    id: account.id,
                    last_account_statement: None,
                    wallets_data: wallet_empty_map
                }
            );

            //  Putting together the accounts_id vector
            accounts_id.push(account.id);

            //  Putting together the accounts_string to fetch last statements from db
            accounts_string.push_str(&account.id.to_string());
            accounts_string.push(',');
        } else {
            //  If an account id already exists in the accounts_id vector, then a wallet has
            // already been inserted, insert this wallet into the wallet_statements HashMap
            let account_data_payload = accounts_map.get_mut(&account.id).unwrap();
            // account_data_payload.last_wallet_statements.insert(
            //     account.wallet_id,
            //     None
            // );
            account_data_payload.wallets_data.insert(
                account.wallet_id,
                (Wallet::new_empty(), None)
            );
        }
    }
    accounts_string.pop();

}

fn fill_hash_map(
    accounts_statements: Vec<LastAccountStatement>,
    wallets_data: Vec<WalletData>,
    accounts_map: &mut HashMap<AccountIdType, AccountData>
) {
    //  Iterates through the accounts_statements vector and inserts the statements into the HashMap
    for account_statement in accounts_statements {
        if let Some(account_data_payload) = accounts_map.get_mut(&account_statement.accounts_id) {
            account_data_payload.last_account_statement = Some(account_statement);
        } else {

        }
    }

    //  Iterates through the wallets_statements vector and inserts all wallet statements for each
    // accounts_id into the existing HashMap
    for wallet_data in wallets_data {
        if let Some(account_data_payload) = accounts_map.get_mut(&wallet_data.accounts_id) {
            if let Some(mut wallet_payload) = account_data_payload.wallets_data.remove(&wallet_data.wallets_id) {
                println!("Found wallets_id: {}", wallet_data.wallets_id);
                //  Updates the wallet_data_payload members
                let wallet = Wallet{
                    id: wallet_data.wallets_id,
                    currencies_id: wallet_data.currencies_id,
                    charge_priority: wallet_data.charge_priority
                };
                let wallet_statement = LastWalletStatement{
                    wallet_id: wallet_data.wallets_id,
                    balances_date: wallet_data.balances_date,
                    accounts_id: wallet_data.accounts_id,
                    balance: wallet_data.balance,
                    minimum_payment: wallet_data.minimum_payment,
                    created_at: wallet_data.created_at,
                    updated_at: wallet_data.updated_at
                };

                wallet_payload = (wallet, Some(wallet_statement));
                account_data_payload.wallets_data.insert(wallet_data.wallets_id, wallet_payload);

            } else {

            }
        } else {

        }
    }
}
