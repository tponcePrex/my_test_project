use std::collections::HashMap;
use logger::ErrorTypes;
use mysql_async::Conn;
use mysql_async::prelude::Queryable;
use mysql_common::bigdecimal03::Zero;
use mysql_common::chrono::NaiveDate;
use mysql_common::rust_decimal::Decimal;
use crate::data::queries::TransactionConfirmed;
use crate::datatypes::structs::{
    ClientBalanceCaseType,
    InterestForTransaction,
    InterestsForTransactions,
    WalletStatementsResult
};
use crate::new_error;
use crate::utils::MyResult;


////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   QUERIES   ////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn get_transactions_confirmed(conn: &mut Conn, env_payment: Decimal) -> MyResult<()>{

    let wallets_id: u16 = 129;
    let stmt = format!(
        "SELECT * FROM transactions_confirmed WHERE wallets_ID = {} ORDER BY ID ASC", wallets_id);
    //  @TODO ASK: que transaction_codes_id para transaction_categories_id = 4 se tienen que usar para calcular esta mierda
    //  TODO: limitar fechas de fetch en el SELECT
    //  @TODO: filtrar cuotas futuras a la fecha de analisis
    let transactions = conn.query::<TransactionConfirmed, _>(stmt)
        .await
        .map_err(|e| {
            println!("{}", e);
            new_error!(e.to_string(), ErrorTypes::DbConn)
        })?;

    //  PLACEHOLDERS
    //  Today's 2023/05/30 for interests calculations
    let statement_day = NaiveDate::from_ymd_opt(2023, 5, 30).unwrap();

    //  Interests rates. Placeholders for calculations
    let daily_interest_rate = Decimal::new(65, 2) / Decimal::new(365, 0);
    let penalty_interest_rate = Decimal::new(85, 2) / Decimal::new(365, 0);

    let previous_balance = Decimal::new(0, 0);
    //let mut payments = Decimal::new(0, 0);
    //let mut effective_payments = Decimal::new(0, 0);
    //  Payment from console params

    let mut transaction_details: InterestsForTransactions = HashMap::with_capacity(transactions.len());

    let mut wallet_statements = WalletStatementsResult::new(
        transaction_details.clone(),
        statement_day,
        previous_balance
    );

    let payments = &mut wallet_statements.get_total_payments();
    let mut effective_payments = wallet_statements.get_total_payments();
    let purchases = &mut wallet_statements.get_total_purchases();

    //  TODO: remove, this variable is for testing only
    *payments = env_payment;
    effective_payments = env_payment;

    //let mut purchases = Decimal::new(0, 0);

    //  Calculation total value of payments and purchases
    for transaction in &transactions {
        if transaction.debit_credit == -1 {
            effective_payments += transaction.amount;
            *payments += transaction.amount;
        } else if transaction.debit_credit == 1 {
            *purchases += transaction.amount;
        }
    }

    wallet_statements.set_total_payments(*payments);
    wallet_statements.set_total_purchases(*purchases);

    let minimum_payment = *purchases * Decimal::new(25, 2);

    //  Determine the balance case for this wallets_id
    let client_case = calculate_client_balance_case(
        purchases,
        &effective_payments,
        &previous_balance,
        &minimum_payment
    );

    let total_daily_interests = &mut wallet_statements.get_total_daily_interest();
    let total_penalty_interests = &mut wallet_statements.get_total_penalty_interest();

    //  Iterate through the transactions vector
    for transaction in transactions {

        //  Creating new InterestForTransaction struct from required data
        let mut interest_for_transaction = InterestForTransaction::new(
            &transaction,
            &daily_interest_rate,
            &penalty_interest_rate,
            &client_case
        );

        interest_for_transaction.set_transaction_amount(transaction.amount);
        interest_for_transaction.set_effective_transaction_amount(transaction.amount);

        //  Determines if transaction is purchase or payment
        if transaction.debit_credit == -1 {
            //payments += transaction.amount;
            interest_for_transaction.set_is_transaction_purchase(false);
        }
        else if transaction.debit_credit == 1 {
            //purchases += transaction.amount;
            interest_for_transaction.set_is_transaction_purchase(true);
        //  Defaulting to true for cases when transaction amount is zero
        } else { interest_for_transaction.set_is_transaction_purchase(true) }

        //  Set interest rates for transaction
        interest_for_transaction.set_daily_interest_rate(daily_interest_rate);
        interest_for_transaction.set_penalty_interest_rate(penalty_interest_rate);

        //  Determine the transaction date
        interest_for_transaction.set_balances_date(transaction.balances_date);

        //  Calculate interests based on the client's case
        //  For this function I have to calculate what the interests will be if the client doesn't
        // pay the purchases until the next statement day
        if interest_for_transaction.get_is_transaction_purchase() {
            interest_for_transaction.calculate_daily_interest_rate(
                &mut effective_payments,
                &client_case,
                &wallet_statements.get_statement_day()
            );
        }

        //  Updating both financial and penalty interests in wallet_statements
        *total_daily_interests += interest_for_transaction.get_total_daily_interest();
        *total_penalty_interests += interest_for_transaction.get_total_penalty_interest();

        println!("{}  -------------------------", interest_for_transaction.get_balances_date());
        println!("Transaction amount: {}", interest_for_transaction.get_transaction_amount());
        println!("Effective transaction amount: {}", interest_for_transaction.get_effective_transaction_amount());
        println!("Daily Interest: {}", interest_for_transaction.get_total_daily_interest());
        println!("Penalty interest: {}", interest_for_transaction.get_total_penalty_interest());
        println!("Daily interest rate: {}", interest_for_transaction.get_daily_interest_rate());
        println!("Penalty interest rate: {}", interest_for_transaction.get_penalty_interest_rate());
        println!(" ");

        //  Editing the HashMap created before this iterator, to insert it into wallet_statements later
        transaction_details.insert(transaction.id, interest_for_transaction);
    }

    //  Inserting the transaction_details HashMap into wallet_statements
    wallet_statements.set_transactions_details(transaction_details);

    let total_balance = *total_penalty_interests + *total_daily_interests + *purchases + *payments;

    println!("Purchases: {}", purchases);
    println!("Payments: {}", payments);
    println!("Effective payments: {}", effective_payments);
    println!("Minimum payment: {}", minimum_payment);
    println!("Total daily interests: {}", total_daily_interests);
    println!("Total penalty interests: {}", total_penalty_interests);
    println!(" ");
    println!("Client case: {:?}", client_case);
    println!(" ");
    println!(" ");
    println!("Total balance: {}", total_balance);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   FUNCTIONS   //////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

fn calculate_client_balance_case(
    purchases: &Decimal,
    payments: &Decimal,
    previous_balance: &Decimal,
    minimum_payment: &Decimal
) -> ClientBalanceCaseType {

    //  If payments is greater or equal than positive_amount, debt is fully covered
    if payments.abs() >= *purchases + *previous_balance { ClientBalanceCaseType::UpToDate }

    //  If negative_amount is zero, then no debt payment was registered
    else if *payments == Decimal::zero() { return ClientBalanceCaseType::NoPayment }

    //  If negative_amount is lesser than previous_balance + purchases then two cases can be applied
    else if payments.abs() < *purchases + *previous_balance {
        //  If balance is greater or equal than minimum_payment then minimum is covered
        //  The zero - negative_amount is because the minimum_payment is positive
        if payments.abs() >= *minimum_payment { ClientBalanceCaseType::MinimumCovered }

        //  Else, debt payed by client is lesser than minimum_payment, then client's in penalty
        else { ClientBalanceCaseType::Penalty }
    }

    else { ClientBalanceCaseType::Undetermined }
    //  TwoDaysGrace is only available for Uruguay
    //  TODO: implement twoDaysGrace for Uruguay
}
