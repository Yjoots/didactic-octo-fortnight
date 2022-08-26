pub use client::Client;
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::{btree_map::Values, hash_map::Entry, BTreeMap, HashMap};
pub use transaction::{
    DisputeTransaction, DisputeTransactionType, OperationTransaction, OperationTransactionType,
    Transaction,
};
pub use transcode::transcode;

mod client;
#[cfg(test)]
mod tests;
mod transaction;
mod transcode;

#[derive(thiserror::Error, Debug)]
pub enum OperationError {
    #[error("Transaction with tx: {0} already exists")]
    TransactionExists(u32),
    #[error("Transaction with tx: {0} withdraw: {1} exceeded available units: {2}")]
    WithdrawExceeded(u32, Decimal, Decimal),
    #[error("Account {0} locked")]
    Locked(u16),
}

#[derive(thiserror::Error, Debug)]
pub enum DisputeError {
    #[error("Dispute with tx: {0} already exists")]
    DisputeExists(u32),
    #[error("Dispute with tx: {0} doesn't exist")]
    DisputeDoesntExists(u32),
    #[error("Transaction with tx: {0} doesn't exist")]
    TransactionDoesntExists(u32),
    #[error("Cannot resolve dispute with tx: {0} client: {1} didn't issue themselves")]
    DisputeConflict(u32, u16),
    #[error("Cannot issue chargeback with tx: {0} by client: {1} to {2}")]
    ChargebackConflict(u32, u16, u16),
    #[error("Account {0} locked")]
    Locked(u16),
}

#[derive(Default, Serialize)]
#[serde(transparent)]
pub struct Authority {
    client_state: BTreeMap<u16, Client>,
    #[serde(skip)]
    transaction_ledger: HashMap<u32, OperationTransaction>,
    #[serde(skip)]
    dispute_ledger: HashMap<u32, DisputeTransaction>,
}

impl Authority {
    /// Applies unit withdraw and deposit operations
    fn apply_operation(&mut self, t: OperationTransaction) -> Result<(), OperationError> {
        match self.transaction_ledger.entry(t.tx()) {
            // Ensure transaction doesn't exist already
            Entry::Occupied(_) => return Err(OperationError::TransactionExists(t.tx())),
            Entry::Vacant(v) => {
                let client = self
                    .client_state
                    .entry(t.client())
                    .or_insert_with(|| Client::new(t.client()));

                client.apply_operation_transaction(&t)?;
                // If apply_operation_transaction succeeds only then we can ledge transaction
                v.insert(t);
            }
        }

        Ok(())
    }

    /// Applies dispute operations
    fn apply_dispute(&mut self, t: DisputeTransaction) -> Result<(), DisputeError> {
        // All dispute transactions refer to a transaction
        let disputed_transaction = self
            .transaction_ledger
            .get(&t.tx())
            .ok_or_else(|| DisputeError::TransactionDoesntExists(t.tx()))?;

        // Transaction exists therefore client must also exist in our state
        // as client_state and transaction_state are insert only.
        let client = self
            .client_state
            .get_mut(&disputed_transaction.client())
            .unwrap();

        let entry = self.dispute_ledger.entry(t.tx());
        match t.transaction_type() {
            DisputeTransactionType::Dispute => match entry {
                Entry::Occupied(_) => return Err(DisputeError::DisputeExists(t.tx())),
                Entry::Vacant(v) => {
                    client.apply_dispute(disputed_transaction)?;
                    v.insert(t);
                }
            },
            DisputeTransactionType::Resolve => match entry {
                Entry::Occupied(o) => {
                    let existing_dispute = o.get();

                    // Client may only resolve disputes they issued themselves
                    if t.client() != existing_dispute.client() {
                        return Err(DisputeError::DisputeConflict(t.tx(), t.client()));
                    }

                    client.apply_resolve(disputed_transaction)?;
                    o.remove_entry();
                }
                Entry::Vacant(_) => return Err(DisputeError::DisputeDoesntExists(t.tx())),
            },
            DisputeTransactionType::Chargeback => match entry {
                Entry::Occupied(o) => {
                    // Can only issue chargeback on transactions from own account
                    if t.client() != disputed_transaction.client() {
                        return Err(DisputeError::ChargebackConflict(
                            t.tx(),
                            t.client(),
                            disputed_transaction.client(),
                        ));
                    }

                    client.apply_chargeback(disputed_transaction)?;
                    o.remove_entry();
                }
                Entry::Vacant(_) => return Err(DisputeError::DisputeDoesntExists(t.tx())),
            },
        }

        Ok(())
    }
}

impl Authority {
    /// Allows applying an iterator of transactions to the [Authority]
    ///
    /// In a multi-input environment such as where multiple clients connect to
    /// the authority, an iterator (or stream) which combines the data stream
    /// into one can be created.
    pub fn apply_iter<I>(&mut self, iter: I)
    where
        I: Iterator<Item = Transaction>,
    {
        for t in iter {
            match t {
                Transaction::Operation(o) => {
                    if let Err(e) = self.apply_operation(o) {
                        eprintln!("{}", e);
                    }
                }
                Transaction::Dispute(d) => {
                    if let Err(e) = self.apply_dispute(d) {
                        eprintln!("{}", e);
                    }
                }
            }
        }
    }

    /// Iterator across client state
    pub fn iter_clients(&mut self) -> Values<'_, u16, Client> {
        self.client_state.values()
    }
}

impl FromIterator<Transaction> for Authority {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Transaction>,
    {
        let mut a = Authority::default();
        a.apply_iter(iter.into_iter());
        a
    }
}
