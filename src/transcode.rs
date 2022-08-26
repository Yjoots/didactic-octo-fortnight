use crate::{
    transaction::OperationTransactionType, DisputeTransaction, DisputeTransactionType,
    OperationTransaction, Transaction,
};
use csv::Reader;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Represents the raw serde validated data entering the program
///
/// Optional `amount` property is used in order to accept csv files which might
/// be formatted with variable row lengths.
#[derive(Deserialize)]
struct RawTransaction {
    #[serde(alias = "type")]
    transaction_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

impl TryFrom<RawTransaction> for Transaction {
    type Error = String;

    fn try_from(transaction: RawTransaction) -> Result<Self, Self::Error> {
        let client = transaction.client;
        let tx = transaction.tx;
        let amount = transaction.amount;

        let res = match transaction.transaction_type {
            TransactionType::Deposit => Transaction::Operation({
                let mut amount =
                    amount.ok_or_else(|| "Missing amount for deposit operation".to_string())?;
                amount.rescale(4);

                OperationTransaction::new(OperationTransactionType::Deposit, client, tx, amount)
            }),
            TransactionType::Withdrawal => Transaction::Operation({
                let mut amount =
                    amount.ok_or_else(|| "Missing amount for deposit operation".to_string())?;
                amount.rescale(4);

                OperationTransaction::new(OperationTransactionType::Withdrawal, client, tx, amount)
            }),
            TransactionType::Dispute => Transaction::Dispute(DisputeTransaction::new(
                DisputeTransactionType::Dispute,
                client,
                tx,
            )),
            TransactionType::Resolve => Transaction::Dispute(DisputeTransaction::new(
                DisputeTransactionType::Resolve,
                client,
                tx,
            )),
            TransactionType::Chargeback => Transaction::Dispute(DisputeTransaction::new(
                DisputeTransactionType::Chargeback,
                client,
                tx,
            )),
        };
        Ok(res)
    }
}

/// Produce an iterator of [Transactions](Transaction)
pub fn transcode<T>(rdr: Reader<T>) -> impl IntoIterator<Item = Transaction>
where
    T: Read,
{
    // While it would theoretically be possible to directly deserialize
    // [Transaction], unfortunately the csv [Deserializer] does not support
    // untagged unions.
    //
    // https://github.com/BurntSushi/rust-csv/issues/211
    //
    // Of course an alternative is implementing [Deserialize] ourselves, but
    // for the purpose of this work it should be enough.
    rdr.into_deserialize::<RawTransaction>()
        .filter_map(|r| match r {
            Ok(rt) => match Transaction::try_from(rt) {
                Ok(t) => Some(t),
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        })
}
