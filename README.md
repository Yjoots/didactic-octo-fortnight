# didactic-octo-fortnight

Implements a simple payments engine, `Authority` that reads a series of transactions
from an iterator, updates client accounts, handles disputes and chargebacks.

## Usage

Engine accepts an input `csv` file name, processes transactions, and outputs client states as valid `csv`.

```cargo run -- ./tests/sample.csv```

Any encountered errors are emitted in stderr and can be silenced using:

```cargo run -- ./tests/sample.csv 2> /dev/null```

Additionally, unit and integration tests can be ran using:

```cargo test```

## Transactions

Transactions are categorized into two categories, operation and dispute transactions.
This separation clearly denotes the scope of responsibility of each operation.

Operation transactions entail standard deposit and withdrawal transactions from a given client account, whereas dispute transactions operate on the dispute state of each transaction.

* Operation transactions are interpreted as transactions applied on the account of the provided client.
* Dispute transactions are interpreted as dispute operations on the provided transaction **issued by** the provided client

Transactions are evaluated with regard to the following rules:

1. Withdrawals may not be made if the final state results in a negative account balance
2. Client may dispute any transaction, including ones made on his own account
3. Client may only resolve disputes they themselves issued
4. Client may onnly issue chargebacks on transaction on own account
5. Locked accounts may not perform any operations, however, new disputes may still be opened

## Architecture

`Authority` maintains three ledgers to handle transactions:

* Dense, ordered, `BTree`, map of client state
* Hash map of transactions
* Hash map of opened disputes

Since disputes reference transactions then we must retain them somewhere. It would make sense to use a `BTree` map to store transactions due to its dense and ordered nature, however, access time is more important to us since we do not need to iterate over transactions.

Since we do need to iterate over client state in an ordered manner to facilitate tests, I used a `BTree` map to store client state. A Hash map can be used with a hasher with deterministic ordering, or alternatively even a bare `Vec`.

Since opened disputes are sparse and we also require constant access, I used a Hash map.

In order to facilitate a multi-input environment, such as where multiple clients connect, `Authority` accepts an iterator input, which is also produced in the binary program from the `csv` file.

---

`Authority` and `Client` objects are structured in such a way that each object has strict control over it's internals, such that in order for `Authority` to modify its ledgers, the `Client` must first confirm the applicability of the transaction. This goes a **long** way to making the code maintainable and safe.

Each transaction is then subdidived into a unit operation on `Client` state to make the logic of the application easy to reason about.

Unit operations each expose their unique error types which cover the entirety of error cases under the transaction rules. This would be useful when eventually extending the program, however, for now the errors are simply printed to stderr.

## Tests

A suite of unit tests was created that validates the unit effects of each operation but also their interleaving. A further integration test is also provided that evaluates the file `./tests/sample.csv`.

An interesting note is the use of `debug_assertions` for enforcing that the held number of units is always positive.

## Limitations

The main limitation of the codebase that hinders its maintainability and readability, is the presence of the `transcode` interface. Due to a lack of support for untagged enums in the `csv` library used to deserialize `csv` rows, I had to create an intermediate `RawTransaction` type that could be handled.

I am also not satisfied with the use of `Decimal` types due to their heavy memory use and computational cost, as we can simply use a `u64`. However, its an acceptable tradeof for this project as it greately simplifies decimal handling and formatting. Some care must be taken that decimals are not rescaled.