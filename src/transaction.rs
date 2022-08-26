use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperationTransactionType {
    /// Unit deposit transaction
    Deposit,
    /// Unit withdrawal transaction
    Withdrawal,
}

/// Represents transactions which are entered into the transaction ledger
/// which can be indexed by their `id`.
///
/// The choice to use [Decimal] for amount is for ease of parsing, however,
/// a u64 fixed precision type will  
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct OperationTransaction {
    transaction_type: OperationTransactionType,
    client: u16,
    tx: u32,
    amount: Decimal,
}

impl OperationTransaction {
    pub fn new(
        transaction_type: OperationTransactionType,
        client: u16,
        tx: u32,
        amount: Decimal,
    ) -> Self {
        Self {
            transaction_type,
            client,
            tx,
            amount,
        }
    }

    pub fn transaction_type(&self) -> OperationTransactionType {
        self.transaction_type
    }

    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn tx(&self) -> u32 {
        self.tx
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisputeTransactionType {
    /// Disputes the referenced transaction and opens a dispute resolution
    Dispute,
    /// Resolves previously opened dispute
    Resolve,
    /// Resolves previously opened dispute via a charge back
    Chargeback,
}

/// Represents transactions which refer to
/// [OperationTransactions](OperationTransaction) and change their dispute
/// state.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct DisputeTransaction {
    transaction_type: DisputeTransactionType,
    client: u16,
    tx: u32,
}

impl DisputeTransaction {
    pub fn new(transaction_type: DisputeTransactionType, client: u16, tx: u32) -> Self {
        Self {
            transaction_type,
            client,
            tx,
        }
    }

    pub fn transaction_type(&self) -> DisputeTransactionType {
        self.transaction_type
    }

    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn tx(&self) -> u32 {
        self.tx
    }
}

/// Normalized representation of possible transactions
///
/// What this particular form allows us to do is validate that all the
/// necessary data is available once the transaction must be processed. This is
/// contrary to [RawTransaction](crate::transcode::RawTransaction) which has an
/// optional `amount` property.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Transaction {
    Operation(OperationTransaction),
    Dispute(DisputeTransaction),
}
