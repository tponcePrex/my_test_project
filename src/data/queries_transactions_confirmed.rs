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

    //  Today's 2023/05/30 for interests calculations
    let today_date = NaiveDate::from_ymd_opt(2023, 05, 30);

    //  Interests rates. Placeholders for calculations
    let daily_interest_rate = Decimal::new(65, 2) / Decimal::new(365, 0);
    let penalty_interest_rate = Decimal::new(85, 2) / Decimal::new(365, 0);

    let previous_balance = Decimal::new(0, 0);

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
        "SELECT * FROM transactions_confirmed WHERE wallets_ID = {} ORDER BY balances_date ASC", wallets_id);
    let transactions = conn.query::<TransactionConfirmed, _>(stmt)
        .await
        .map_err(|e| {
            println!("{}", e.to_string());
            new_error!(e.to_string(), ErrorTypes::DbConn)
        })?;

    let interests_for_transaction: InterestsForTransactions = HashMap::with_capacity(transactions.len());

    let mut wallet_statements = WalletStatementsResult{
        balance: Decimal::zero(),
        previous_balance,
        minimum_payment: Decimal::zero(),
        total_interests: interests_for_transaction
    };

    let mut payments = Decimal::new(0, 0);
    let mut purchases = Decimal::new(0, 0);


    for transaction in transactions {
        let mut interest_for_transaction = InterestForTransaction::new();

        if transaction.debit_credit == -1 {
            payments += transaction.amount;
            interest_for_transaction.set_is_transaction_purchase(false);
        }
        else if transaction.debit_credit == 1 {
            purchases += transaction.amount;
            interest_for_transaction.set_is_transaction_purchase(true);
        }



        wallet_statements.total_interests.insert(transaction.id, interest_for_transaction);
    }

    let minimum_payment = purchases * Decimal::new(3, 1);

    let client_case = calculate_client_balance_case(
        &purchases,
        &payments,
        &previous_balance,
        &minimum_payment
    );

    println!("Purchases: {}", purchases);
    println!("Payments: {}", payments);
    println!("Minimum payment: {}", minimum_payment);
    println!("Client Case: {:?}", client_case);

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

    let minimum_payment = Decimal::new(5000, 0);



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
) -> (ClientBalanceCaseType) {
    //TODO usar previous_balance
    //  If payments is greater or equal than positive_amount, debt is fully covered
    if Decimal::zero() - payments >= *purchases { return ClientBalanceCaseType::UpToDate }

    //  If negative_amount is zero, then no debt payment was registered
    else if *payments == Decimal::zero() { return ClientBalanceCaseType::NoPayment }

    //  If negative_amount is lesser than positive_amount then two cases can be applied
    else if *payments < Decimal::zero() {
        //  If balance is greater or equal than minimum_payment then minimum is covered
        //  The zero - negative_amount is because the minimum_payment is positive
        if Decimal::zero() - payments >= *minimum_payment { ClientBalanceCaseType::MinimumCovered }

        //  Else, debt payed by client is lesser than minimum_payment, then client's in penalty
        else { ClientBalanceCaseType::Penalty }
    }

    else { ClientBalanceCaseType::Undetermined }
    //  TwoDaysGrace is only available for Uruguay
    //  TODO: implement twoDaysGrace for Uruguay
}

fn calculate_days_amount(purchase_date: &NaiveDate, statement_date: &NaiveDate) -> Decimal{

    //  Amount of days between the specific purchase and the statement day for the calculation
    let days_amount = statement_date.signed_duration_since(*purchase_date).num_days();

    //  Returning amount of days as Decimal
    Decimal::new(days_amount, 0)
}

fn calculate_daily_interest_rate(
    daily_interest_rate: &Decimal,
    penalty_interest_rate: &Decimal,
    is_client_in_penalty: bool
) -> Decimal {
    let total_interest_amount = Decimal::zero();

    //  Recibir una estructura que contenga tanto el wallet_id que no voy a usar para una pija,
    // como los intereses daily y penalty, y la cantidad de dias para calcular intereses,
    // tambien si el cliente esta en mora. Todo esto dentro de una estructura

    if is_client_in_penalty {

    }

    total_interest_amount
}

