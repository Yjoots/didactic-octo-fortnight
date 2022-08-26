use crate::{DisputeError, OperationError, OperationTransaction, OperationTransactionType};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Client {
    #[serde(rename = "client")]
    id: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            // Should create specialized type that enforces scale invariant
            available: Decimal::new(0, 4),
            held: Decimal::new(0, 4),
            total: Decimal::new(0, 4),
            locked: false,
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn available(&self) -> &Decimal {
        &self.available
    }

    pub fn held(&self) -> &Decimal {
        &self.held
    }

    pub fn total(&self) -> &Decimal {
        &self.total
    }

    pub fn locked(&self) -> bool {
        self.locked
    }
}

impl Client {
    /// Applies a deposit or withdrawal transaction to the client
    pub fn apply_operation_transaction(
        &mut self,
        t: &OperationTransaction,
    ) -> Result<(), OperationError> {
        if self.locked {
            return Err(OperationError::Locked(self.id));
        }

        let amount = t.amount();
        match t.transaction_type() {
            OperationTransactionType::Deposit => {
                self.available += amount;
                self.total += amount;
            }
            OperationTransactionType::Withdrawal => {
                if self.available < amount {
                    return Err(OperationError::WithdrawExceeded(
                        t.tx(),
                        amount,
                        self.available,
                    ));
                }

                self.available -= amount;
                self.total -= amount;
            }
        }

        Ok(())
    }

    /// Applies a dispute transaction to the client
    pub fn apply_dispute(&mut self, t: &OperationTransaction) -> Result<(), DisputeError> {
        let amount = t.amount();
        self.held += amount;

        match t.transaction_type() {
            OperationTransactionType::Deposit => {
                // It is valid to potentially go into the negative as a deposit
                // transaction can always be disputed
                self.available -= amount;
            }
            OperationTransactionType::Withdrawal => {
                self.total += amount;
            }
        }

        Ok(())
    }

    /// Applies a dispute resolve transaction to the client
    pub fn apply_resolve(&mut self, t: &OperationTransaction) -> Result<(), DisputeError> {
        if self.locked {
            return Err(DisputeError::Locked(self.id));
        }

        let amount = t.amount();

        debug_assert!(self.held >= amount);
        self.held -= amount;

        match t.transaction_type() {
            OperationTransactionType::Deposit => {
                self.available += amount;
            }
            OperationTransactionType::Withdrawal => {
                self.total -= amount;
            }
        }

        Ok(())
    }

    /// Applies a transaction chargeback to the client
    pub fn apply_chargeback(&mut self, t: &OperationTransaction) -> Result<(), DisputeError> {
        if self.locked {
            return Err(DisputeError::Locked(self.id));
        }

        let amount = t.amount();

        debug_assert!(self.held >= amount);
        self.held -= amount;

        match t.transaction_type() {
            OperationTransactionType::Deposit => {
                self.total -= amount;
            }
            OperationTransactionType::Withdrawal => {
                self.available += amount;
            }
        }

        self.locked = true;

        Ok(())
    }
}

#[cfg(test)]
impl Client {
    pub fn test(id: u16, available: i64, held: i64, total: i64, locked: bool) -> Self {
        Self {
            id,
            available: Decimal::from(available),
            held: Decimal::from(held),
            total: Decimal::from(total),
            locked,
        }
    }
}
