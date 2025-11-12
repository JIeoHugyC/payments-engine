# Payments Engine

A simple transaction processing engine that reads CSV transactions,
updates client accounts, handles disputes and chargebacks,
and outputs account states.

## Usage

Basic usage:
```bash
cargo run -- transactions.csv > accounts.csv
```

With logging disabled:
```bash
RUST_LOG=off cargo run -- transactions.csv > accounts.csv
```

Input CSV format:
```
type,client,tx,amount
deposit,1,1,10.0
withdrawal,1,2,5.0
dispute,1,1,
```

Output CSV format:
```
client,available,held,total,locked
1,5.0,0.0,5.0,false
```

## Testing

Run unit tests:
```bash
cargo test --workspace
```

Run with sample data:
```bash
cargo run -- tests/fixtures/basic.csv
cargo run -- tests/fixtures/disputes.csv
cargo run -- tests/fixtures/chargebacks.csv
cargo run -- tests/fixtures/invalid_data.csv
```

## Design Decisions

**Modular architecture**: The transaction processor is a separate crate,
making it reusable as a library without coupling to CLI concerns.

**Streaming processing**: CSV is read and processed line-by-line using iterators,
keeping memory usage constant regardless of input size.

**Sequential processing**: Transactions are processed in order as they appear 
in the file. This ensures correct account state and enables proper dispute handling.

**Type safety**: Uses `rust_decimal::Decimal` for precise financial calculations 
(4 decimal places). The type system prevents incorrect operations through 
strongly-typed transaction types.

**Error handling**: Individual transaction errors are logged but don't halt processing.
The engine continues processing subsequent transactions.

**Logging**: Structured logging via `tracing` provides visibility into invalid CSV records,
transaction processing errors, and processing statistics. 
Configurable via `RUST_LOG` environment variable.

## Invariants

The engine maintains these invariants:
- `total = available + held` (always)
- Disputes only reference existing transactions
- Chargebacks only apply to disputed transactions
- Locked accounts reject new transactions
- Withdrawals cannot exceed available funds

## Assumptions

Based on the specification:

1. **Single asset account**: Each client has one account for all transactions

2. **Transaction uniqueness**: Transaction IDs (tx) are globally unique

3. **Chronological order**: Transactions in the input file are ordered chronologically

4. **Error handling**: Invalid or malformed transactions are skipped with logging, processing continues

5. **Precision**: All amounts use 4 decimal places maximum

6. **Idempotency**: Duplicate transaction IDs are treated as errors and skipped

## Performance Considerations

**Streaming**:
- CSV reading is streaming (doesn't load entire file)
- Processing is O(1) per transaction
- CSV writing is streaming (accounts written directly to stdout)
