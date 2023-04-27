use std::collections::HashMap;
use logger::ErrorTypes;
use mysql_async::Conn;
use mysql_async::prelude::Queryable;
use mysql_common::bigdecimal03::Zero;
use mysql_common::chrono::NaiveDate;
use mysql_common::rust_decimal::Decimal;
use crate::data::queries::TransactionConfirmed;
use crate::datatypes::structs::{InterestForTransaction, InterestsForTransactions, WalletStatementsResult};
use crate::new_error;
use crate::utils::MyResult;


////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   QUERIES   ////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

//  @TODO: Fetch daily interest rate
//  @TODO: Calculate interest for individual charges
//  @TODO: Add all interests and add penalty value if applicable
//  @TODO: Determine final client account balance


pub async fn get_transactions_confirmed(conn: &mut Conn) -> MyResult<()>{


    //  @TODO ASK: does wallet_id change for a different currency and same client?
    //  Cada wallets_id esta linkeado a una moneda particular, es independiente
    //  @TODO ASK: do different currencies for a same client have different wallets_id?
    //  Ya esta
    //  @TODO ASK: do we have to develop for interests in different currencies separately or all of them together?
    //  Y aesta
    //  @TODO ASK: are those the correct values for the placeholders?
    //  Ya esta
    //  @TODO ASK: check if it is convenient to calculate interest for each purchase separately and add them together
    //  Ya esta

    //  @TODO ASK: que transaction_codes_id para transaction_categories_id = 4 se tienen que usar para calcular esta mierda


    //  My calculations are to notify the client how much he'll owe if he doesn't pay
    //  The result of my calculations wont be the charges he owes
    //  Para linkear intereses a las compras usar un hashmap, con key id de compra
    //  default_interest es interes por mora
    //  daily interest rate: 65%/365
    //  penalty interest rate: %85/365
    //  Peroido de gracia es 7 dias en principio
    //  Amount es el valor final que tengo que analizar

    //  Si el cliente no paga hasta el proximo cierre, va a tener ciertos intereses, que es lo que
    // tengo que calcular aca

    let wallets_id: u16 = 129;
    let stmt = format!(
        "SELECT * FROM transactions_confirmed WHERE wallets_ID = {} ORDER BY ID ASC", wallets_id);
    let transactions = conn.query::<TransactionConfirmed, _>(stmt)
        .await
        .map_err(|e| {
            println!("{}", e);
            new_error!(e.to_string(), ErrorTypes::DbConn)
        })?;

    //  @TODO: filtrar cuotas futuras a la fecha de analisis

    //  PLACEHOLDERS
    //  Today's 2023/05/30 for interests calculations
    let statement_day = NaiveDate::from_ymd_opt(2023, 5, 30).unwrap();

    //  Interests rates. Placeholders for calculations
    let daily_interest_rate = Decimal::new(65, 2) / Decimal::new(365, 0);
    let penalty_interest_rate = Decimal::new(85, 2) / Decimal::new(365, 0);

    let previous_balance = Decimal::new(0, 0);
    let mut effective_payments = Decimal::new(0, 0);
    let mut payments = Decimal::new(0, 0);
    let mut purchases = Decimal::new(0, 0);

    let interests_for_transaction: InterestsForTransactions = HashMap::with_capacity(transactions.len());

    let mut wallet_statements = WalletStatementsResult{
        balance: Decimal::zero(),
        previous_balance,
        minimum_payment: Decimal::zero(),
        total_interests: interests_for_transaction
    };


    //  Calculate client's balance case
    let minimum_payment = Decimal::new(4000, 0);

    //  Calculation total value of payments and purchases
    for transaction in &transactions {
        if transaction.debit_credit == -1 {
            effective_payments += transaction.amount;
            payments += transaction.amount;
        } else if transaction.debit_credit == 1 {
            purchases += transaction.amount;
        }
    }

    //  Determine the balance case for this wallets_id
    let client_case = calculate_client_balance_case(
        &purchases,
        &effective_payments,
        &previous_balance,
        &minimum_payment
    );

    // let mut is_minimum_covered = false;
    //
    // if Decimal::zero() - payments >= minimum_payment { is_minimum_covered = true }

    let mut total_daily_interests = Decimal::zero();
    let mut total_penalty_interests = Decimal::zero();

    //  Iterate through the transactions vector
    for transaction in transactions {

        //  Implementar From para inicializar struct
        let mut interest_for_transaction = InterestForTransaction::new();

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
            calculate_daily_interest_rate(
                &mut interest_for_transaction,
                &mut effective_payments,
                &ClientBalanceCaseType::NoPayment,
                &statement_day
            );
        }

        total_daily_interests += interest_for_transaction.get_total_daily_interest();
        total_penalty_interests += interest_for_transaction.get_total_penalty_interest();

        println!("{}  -------------------------", interest_for_transaction.get_balances_date());
        println!("Transaction amount: {}", interest_for_transaction.get_transaction_amount());
        println!("Effective transaction amount: {}", interest_for_transaction.get_effective_transaction_amount());
        println!("Daily Interest: {}", interest_for_transaction.get_total_daily_interest());
        println!("Penalty interest: {}", interest_for_transaction.get_total_penalty_interest());
        println!("Daily interest rate: {}", interest_for_transaction.get_daily_interest_rate());
        println!("Penalty interest rate: {}", interest_for_transaction.get_penalty_interest_rate());
        println!(" ");

        wallet_statements.total_interests.insert(transaction.id, interest_for_transaction);
    }

    println!("Purchases: {}", purchases);
    println!("Payments: {}", payments);
    println!("Effective payments: {}", effective_payments);
    println!("Minimum payment: {}", minimum_payment);
    println!("Total daily interests: {}", total_daily_interests);
    println!("Total penalty interests: {}", total_penalty_interests);

    /*
    Calcular intereses:
    1-obtener todas las compras ordenadas por fecha
    2-obtener todos los pagos ordenados por fecha
    3-obtener el balance anterior
    4-sumar los pagos hechos dentro del periodo de gracia
    5-determinar si el total de pagos es igual o mayor al total de compras + balance anterior
        A-si el pago es mayor a compras + balance anterior, entonces esta saldado, mover resultado al balance actual
        B-si es menor entonces van a haber intereses (ver anexo B)
    6-definir el balance total
    7-determinar el caso del cliente en base a balance total

    B-Intereses:
    1-determinar si el pago es mayor al minimo, si no es, el cliente esta en penalty
    2-cancelar las deudas desde la mas vieja hasta la mas nueva
    3-al restante, ir por cada transaccion calculando intereses segun los dias transcurridos
    4-sumar intereses financieros
     */




    // println!("Expenses: {} \nDebt payed: {} \n", positive_amount, negative_amount);
    // println!("Balance: {} \nBalance Case: {:?} \n", balance, client_case);


    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   FUNCTIONS   //////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClientBalanceCaseType{
    UpToDate,
    MinimumCovered,
    Penalty,
    NoPayment,
    TwoDaysGrace,
    Undetermined
}

/*
    Determinar el caso del cliente:
    1-obtener todas las compras
    2-obtener el balance anterior
    3-obtener todos los pagos
    4-hacer balance_nuevo = compras + balance_anterior + pagos (pagos es negativo)
    5-obtener el minimo
    6-determinar el caso en base al minimo, balance y/o pago
    */
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

    //  If negative_amount is lesser than positive_amount then two cases can be applied
        //  Revisar
    else if *payments.abs() < Decimal::zero() {
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

fn calculate_days_amount(purchase_date: &NaiveDate, statement_day: &NaiveDate) -> Decimal{

    //  Amount of days between the specific purchase and the statement day for the calculation
    let days_amount = statement_day.signed_duration_since(*purchase_date).num_days();

    //  Returning amount of days as Decimal
    Decimal::new(days_amount, 0)
}

//  Probar que esto sea una implementacion en vez de una funcion standalone
fn calculate_daily_interest_rate(
    interest_for_transaction: &mut InterestForTransaction,
    payments: &mut Decimal,
    client_case: &ClientBalanceCaseType,
    statement_day: &NaiveDate
) {

    //  Payments es negativo, por eso se suma siempre
    //  Decrease payment value to determine when to start applying interests, or not if purchases are covered
    if payments.abs() >= interest_for_transaction.get_transaction_amount() {
        //  Decreasing the value of payments and setting the effective_transaction_amount to zero
        *payments += interest_for_transaction.get_effective_transaction_amount();
        interest_for_transaction.set_effective_transaction_amount(Decimal::zero());
        println!("New payments value: {}", payments);

    } else if ( payments.abs() < interest_for_transaction.get_effective_transaction_amount() )
        && ( payments.abs() > Decimal::zero() ) {
        //  Decreasing the value of effective transaction amount, and setting payments to zero
        let mut effective_transaction_amount = interest_for_transaction.get_effective_transaction_amount();
        effective_transaction_amount += *payments;
        interest_for_transaction.set_effective_transaction_amount(effective_transaction_amount);
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
    if interest_for_transaction.get_effective_transaction_amount() > Decimal::zero() {
        //  Calculate interest according to the client's balance case
        match client_case {
            ClientBalanceCaseType::UpToDate => {
                //  If client is up to date, the interests will both be zero
                interest_for_transaction.set_total_daily_interest(Decimal::zero());
                interest_for_transaction.set_total_penalty_interest(Decimal::zero());
            },
            ClientBalanceCaseType::NoPayment => {
                //  If no payment was registered, both financial and penalty interests apply
                let days_amount = calculate_days_amount(
                    &interest_for_transaction.get_balances_date(),
                    statement_day
                );

                interest_for_transaction.set_total_daily_interest(
                    interest_for_transaction.get_effective_transaction_amount() * days_amount * interest_for_transaction.get_daily_interest_rate()
                );
                interest_for_transaction.set_total_penalty_interest(
                    interest_for_transaction.get_effective_transaction_amount() * days_amount * interest_for_transaction.get_penalty_interest_rate()
                );
            },
            ClientBalanceCaseType::MinimumCovered => {
                //  If client has minimum payment covered, only financial interests apply
                let days_amount = calculate_days_amount(
                    &interest_for_transaction.get_balances_date(),
                    statement_day
                );

                interest_for_transaction.set_total_daily_interest(
                    interest_for_transaction.get_effective_transaction_amount() * days_amount * interest_for_transaction.get_daily_interest_rate()
                );
            },
            ClientBalanceCaseType::Penalty => {
                //  If client is on penalty, both financial and penalty interests apply
                let days_amount = calculate_days_amount(
                    &interest_for_transaction.get_balances_date(),
                    statement_day
                );

                interest_for_transaction.set_total_daily_interest(
                    interest_for_transaction.get_effective_transaction_amount() * days_amount * interest_for_transaction.get_daily_interest_rate()
                );
                interest_for_transaction.set_total_penalty_interest(
                    interest_for_transaction.get_effective_transaction_amount() * days_amount * interest_for_transaction.get_penalty_interest_rate()
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
        interest_for_transaction.set_total_daily_interest(Decimal::zero());
        interest_for_transaction.set_total_penalty_interest(Decimal::zero());
    }
}
