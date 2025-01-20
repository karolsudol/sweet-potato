# sweet-potato
Node Indexing Pipeline - EVM


## Usage

Basic usage:
```bash
START=100 COUNT=5 cargo run
```

Logging levels:
```bash
# No logging output (except errors)
START=100 COUNT=5 cargo run

# Summary information only
RUST_LOG=info START=100 COUNT=5 cargo run

# Full debug output including data structures
RUST_LOG=debug START=100 COUNT=5 cargo run
```
