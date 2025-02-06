# sweet-potato
Node Indexing Pipeline - EVM

![Sweet Potato Pipeline](img/sweet-potato.gif)


This is an example of a node indexing pipeline for an EVM chain.
Rust service extracts data from an EVM node and saves it to a ClickHouse database.
DBT is used to transform the data and load it into a data warehouse.

### Start the Database
```bash
docker compose up -d
```

## Run Indexer
The indexer supports the following environment variables:
- `START`: Starting block number (default: 1)
- `COUNT`: Number of blocks to process (default: 1)
- `CLICKHOUSE_URL`: ClickHouse database URL (default: http://localhost:8123)
- `PRINT_OUTPUT`: Whether to print detailed output (default: false)

```bash
# Basic usage (will process 1 block starting from block 1)
cd indexer && cargo run

# Process specific block range
cd indexer && START=1000 COUNT=100 cargo run

# Process blocks with logs
cd indexer && RUST_LOG=info START=100 COUNT=10 cargo run

# Process blocks with detailed output of the porocessed data
cd indexer && RUST_LOG=debug START=100 COUNT=1 cargo run

# Use custom database URL
cd indexer && CLICKHOUSE_URL="http://custom-host:8123" cargo run
```

### Run DBT
```bash
# Navigate to dbt project directory
cd dbt_clickhouse

# Test the connection
dbt debug

# Run the models
dbt run

# Test the data
dbt test

# Run the models with full refresh and debug logs
dbt run --full-refresh --debug
```

### Query Data in ClickHouse
You can use the provided bash scripts to quickly query the data:

```bash
# Make scripts executable
chmod +x db/sql_queries/query_blocks.sh db/sql_queries/query_transactions.sh db/sql_queries/query_receipts.sh

# Query blocks
./db/sql_queries/query_blocks.sh

# Query transactions
./db/sql_queries/query_transactions.sh

# Query receipts
./db/sql_queries/query_receipts.sh
```


### Tear Down the Database
```bash
# Stop and remove containers
docker compose down

# To completely remove everything including volumes (this will delete all data)
docker compose down -v

# Clean up data directory
rm -rf ./db/user_files/*
```




