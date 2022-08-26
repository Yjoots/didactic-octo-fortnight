use crate::{
    Authority, Client, DisputeTransaction,
    DisputeTransactionType::{self, *},
    OperationTransaction,
    OperationTransactionType::{self, *},
    Transaction,
};
use pretty_assertions::assert_eq;
use rust_decimal::Decimal;

fn operation(tt: OperationTransactionType, client: u16, tx: u32, amount: Decimal) -> Transaction {
    Transaction::Operation(OperationTransaction::new(tt, client, tx, amount))
}

fn dispute(tt: DisputeTransactionType, client: u16, tx: u32) -> Transaction {
    Transaction::Dispute(DisputeTransaction::new(tt, client, tx))
}

fn d(number: i64) -> Decimal {
    Decimal::from(number)
}

#[test]
fn deposit() {
    let mut a = Authority::default();
    a.apply_iter(
        vec![
            operation(Deposit, 1, 1, d(1)),
            operation(Deposit, 1, 2, d(1)),
            operation(Deposit, 2, 3, d(1)),
        ]
        .into_iter(),
    );

    assert_eq!(
        vec![
            &Client::test(1, 2, 0, 2, false),
            &Client::test(2, 1, 0, 1, false),
        ],
        a.iter_clients().collect::<Vec<&Client>>()
    );
}

#[test]
fn withdraw() {
    let mut a = Authority::default();
    a.apply_iter(
        vec![
            operation(Deposit, 1, 1, d(1)),
            operation(Deposit, 1, 2, d(2)),
            operation(Deposit, 2, 3, d(2)),
            operation(Withdrawal, 1, 4, d(2)),
            operation(Withdrawal, 2, 5, d(2)),
        ]
        .into_iter(),
    );

    assert_eq!(
        vec![
            &Client::test(1, 1, 0, 1, false),
            &Client::test(2, 0, 0, 0, false),
        ],
        a.iter_clients().collect::<Vec<&Client>>()
    );
}

#[test]
fn withdraw_error() {
    let mut a = Authority::default();
    a.apply_iter(
        vec![
            operation(Deposit, 1, 1, d(1)),
            // Send transaction with conflicting tx
            operation(Deposit, 1, 1, d(2)),
            // Send transaction with withdraw exceeded
            operation(Withdrawal, 1, 2, d(3)),
        ]
        .into_iter(),
    );

    assert_eq!(
        vec![&Client::test(1, 1, 0, 1, false),],
        a.iter_clients().collect::<Vec<&Client>>()
    );
}

#[test]
fn withdraw_dispute() {
    let mut a = Authority::default();
    a.apply_iter(
        vec![
            operation(Deposit, 1, 1, d(1)),
            operation(Deposit, 1, 2, d(2)),
            dispute(Dispute, 1, 2),
        ]
        .into_iter(),
    );

    assert_eq!(
        vec![&Client::test(1, 1, 2, 3, false),],
        a.iter_clients().collect::<Vec<&Client>>()
    );
}

#[test]
fn dispute_resolve() {
    let mut a = Authority::default();
    a.apply_iter(
        vec![
            operation(Deposit, 1, 1, d(1)),
            operation(Deposit, 1, 2, d(2)),
            operation(Withdrawal, 1, 3, d(2)),
            dispute(Dispute, 2, 2),
            dispute(Resolve, 2, 2),
            dispute(Dispute, 2, 3),
            dispute(Resolve, 2, 3),
        ]
        .into_iter(),
    );

    assert_eq!(
        vec![&Client::test(1, 1, 0, 1, false),],
        a.iter_clients().collect::<Vec<&Client>>()
    );
}

#[test]
fn dispute_resolve_error_0() {
    let mut a = Authority::default();
    a.apply_iter(
        vec![
            operation(Deposit, 1, 1, d(1)),
            operation(Withdrawal, 1, 2, d(2)),
            // Cannot dispute transaction that doesn't exist
            dispute(Dispute, 1, 2),
            operation(Withdrawal, 1, 2, d(1)),
            dispute(Dispute, 2, 2),
        ]
        .into_iter(),
    );

    assert_eq!(
        vec![&Client::test(1, 0, 1, 1, false),],
        a.iter_clients().collect::<Vec<&Client>>()
    );
}

#[test]
fn dispute_resolve_error_1() {
    let mut a = Authority::default();
    a.apply_iter(
        vec![
            operation(Deposit, 1, 1, d(1)),
            operation(Deposit, 1, 2, d(2)),
            operation(Withdrawal, 1, 3, d(2)),
            dispute(Dispute, 1, 2),
            // Dispute already exists
            dispute(Dispute, 2, 2),
            // Cannot resolve dispute client themselves didn't issue
            dispute(Resolve, 2, 2),
            dispute(Dispute, 3, 3),
        ]
        .into_iter(),
    );

    assert_eq!(
        vec![&Client::test(1, -1, 4, 3, false),],
        a.iter_clients().collect::<Vec<&Client>>()
    );
}

#[test]
fn dispute_chargeback() {
    let mut a = Authority::default();
    a.apply_iter(
        vec![
            operation(Deposit, 1, 1, d(1)),
            operation(Deposit, 1, 2, d(2)),
            operation(Withdrawal, 1, 3, d(2)),
            dispute(Dispute, 2, 2),
            dispute(Chargeback, 1, 2),
            // Account locked
            dispute(Dispute, 3, 1),
        ]
        .into_iter(),
    );

    assert_eq!(
        vec![&Client::test(1, -2, 1, -1, true),],
        a.iter_clients().collect::<Vec<&Client>>()
    );
}

#[test]
fn dispute_chargeback_error() {
    let mut a = Authority::default();
    a.apply_iter(
        vec![
            operation(Deposit, 1, 1, d(1)),
            operation(Deposit, 1, 2, d(2)),
            operation(Withdrawal, 1, 3, d(2)),
            dispute(Dispute, 2, 2),
            // Cannot chargeback in an account that isn't yours
            dispute(Chargeback, 2, 2),
            dispute(Dispute, 3, 3),
            dispute(Chargeback, 1, 3),
            // Account locked
            operation(Deposit, 1, 4, d(1)),
        ]
        .into_iter(),
    );

    assert_eq!(
        vec![&Client::test(1, 1, 2, 3, true),],
        a.iter_clients().collect::<Vec<&Client>>()
    );
}
