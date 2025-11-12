# Payments Engine

A simple transaction processing engine that reads CSV transactions, handles deposits, withdrawals, disputes, and chargebacks.

## Usage

Basic usage:
```bash
cargo run -- transactions.csv > accounts.csv
```

With logging disabled:
```bash
RUST_LOG=off cargo run -- transactions.csv > accounts.csv
```

With detailed debug logging:
```bash
RUST_LOG=debug cargo run -- transactions.csv > accounts.csv
```

Input CSV format:
```csv
type,client,tx,amount
deposit,1,1,10.0
withdrawal,1,2,5.0
dispute,1,1,
```

Output CSV format:
```csv
client,available,held,total,locked
1,5.0,0.0,5.0,false
```

## Testing

Run unit tests:
```bash
cargo test
```

Run with sample data:
```bash
cargo run -- tests/fixtures/basic.csv
cargo run -- tests/fixtures/disputes.csv
cargo run -- tests/fixtures/chargebacks.csv
```

## Architecture

### Workspace Structure
```
payments-engine/
├── src/                           # CLI binary
│   ├── main.rs                   # Entry point
│   └── config.rs                 # CLI argument parsing
└── crates/
    └── transaction-processor/    # Core business logic
        ├── transaction.rs        # Domain types
        ├── engine.rs            # Processing engine
        └── lib.rs               # Public API
```

### Design Decisions

**Modular architecture**: The transaction processor is a separate crate, making it reusable as a library without coupling to CLI concerns.

**Streaming processing**: CSV is read and processed line-by-line using iterators, keeping memory usage constant regardless of input size.

**Sequential processing**: Transactions are processed in order as they appear in the file, as specified in requirements. This ensures correct account state and enables proper dispute handling.

**Type safety**: Uses `rust_decimal::Decimal` for precise financial calculations (4 decimal places). The type system prevents incorrect operations through strongly-typed transaction types.

**Error handling**: Individual transaction errors are logged but don't halt processing (as per spec: "you can ignore it and assume this is an error on our partner's side"). The engine continues processing subsequent transactions.

**Logging**: Structured logging via `tracing` provides visibility into:
- Invalid CSV records with detailed error messages
- Transaction processing errors (insufficient funds, locked accounts, etc.)
- Processing statistics (total processed, skipped)
- Configurable via `RUST_LOG` environment variable

## Correctness

### Testing Strategy

1. **Unit tests** cover each transaction type and edge cases:
   - Basic deposits and withdrawals
   - Insufficient funds handling
   - Dispute → resolve flow
   - Dispute → chargeback flow with account locking

2. **Integration tests** via sample CSV files testing:
   - Basic transaction flows
   - Complex dispute scenarios
   - Edge cases (precision, insufficient funds, locked accounts)

3. **Type system guarantees**:
   - Transaction IDs are u32, client IDs are u16 (as specified)
   - Account states cannot be partially updated (mutations are atomic)
   - Decimal precision ensures no floating-point errors

### Invariants

The engine maintains these invariants:
- `total = available + held` (always)
- Disputes only reference existing transactions
- Chargebacks only apply to disputed transactions
- Locked accounts reject new transactions
- Withdrawals cannot exceed available funds

## Assumptions

Based on the specification, this implementation assumes:

1. **Single asset account**: Each client has one account for all transactions

2. **Transaction uniqueness**: Transaction IDs (tx) are globally unique

3. **Chronological order**: Transactions in the input file are ordered chronologically

4. **Dispute scope**: Only deposits can be disputed (withdrawals are final once processed)

5. **Error handling**: Invalid or malformed transactions are skipped with logging, processing continues

6. **Precision**: All amounts use 4 decimal places maximum

7. **Idempotency**: Duplicate transaction IDs are treated as errors and skipped

## Performance Considerations

**Memory usage**:
- Client accounts: O(number of unique clients), max ~64KB with u16 client IDs
- Transaction history: O(number of transactions), needed for dispute handling
- For the expected scale (u32 transaction IDs ≈ 4 billion max), this fits comfortably in memory

**Streaming**:
- CSV reading is streaming (doesn't load entire file)
- Processing is O(1) per transaction
- CSV writing is streaming (accounts written directly to stdout)

**Scalability**:
The current architecture could be extended for higher scale:
- Partition by client ID for parallel processing
- Use external storage for transaction history
- Add metrics and observability

## Dependencies

- `rust_decimal`: Precise decimal arithmetic for financial calculations
- `serde`: CSV serialization/deserialization
- `csv`: Efficient CSV parsing with streaming support
- `clap`: Command-line argument parsing with derive macros
- `anyhow`: Ergonomic error handling and context
- `tracing` / `tracing-subscriber`: Structured logging and diagnostics

