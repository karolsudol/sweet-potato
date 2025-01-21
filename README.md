# sweet-potato
Node Indexing Pipeline - EVM

## Database Operations

### Start the Database
```bash
# Start the database
docker compose up -d

# Verify it's running
curl 'http://localhost:8123/?query=SHOW%20DATABASES'
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

## Running the Indexer

### Environment Variables
The indexer supports the following environment variables:
- `START`: Starting block number (default: 1)
- `COUNT`: Number of blocks to process (default: 1)
- `CLICKHOUSE_URL`: ClickHouse database URL (default: http://localhost:8123)
- `PRINT_OUTPUT`: Whether to print detailed output (default: false)

### Run the Indexer
```bash
# Basic usage (will process 1 block starting from block 1)
cd indexer && cargo run

# Process specific block range
cd indexer && START=1000 COUNT=100 cargo run

# Process blocks with detailed output
cd indexer && START=1000 COUNT=10 PRINT_OUTPUT=true cargo run

# Use custom database URL
cd indexer && CLICKHOUSE_URL="http://custom-host:8123" cargo run
```

### Create a Virtual Environment
```bash
python3 -m venv .venv
source .venv/bin/activate
```

### Install Dependencies
```bash
pip install -r requirements.txt
```

## DBT Setup and Operations

### Install DBT Dependencies
```bash
# Make sure you're in the virtual environment
source .venv/bin/activate

# Install dbt-clickhouse
pip install dbt-clickhouse
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
```

### Query Data in ClickHouse

You can check the loaded data using these ClickHouse queries:

```sql
-- View recent blocks
SELECT 
    number,
    hash,
    datetime,
    gas_used,
    transaction_hashes
FROM sweet_potatoe_dbt.raw_blocks
ORDER BY number DESC
LIMIT 5;

-- View recent transactions
SELECT 
    hash,
    block_number,
    datetime,
    `from`,
    `to`,
    value
FROM sweet_potatoe_dbt.raw_transactions
ORDER BY datetime DESC
LIMIT 5;

-- View recent receipts
SELECT 
    transaction_hash,
    block_number,
    datetime,
    status,
    gas_used
FROM sweet_potatoe_dbt.raw_receipts
ORDER BY datetime DESC
LIMIT 5;
```

You can run these queries using curl:
```bash
# Query blocks
curl "http://localhost:8123/?query=SELECT%20number,hash,datetime,gas_used%20FROM%20sweet_potatoe_dbt.raw_blocks%20ORDER%20BY%20number%20DESC%20LIMIT%205%20FORMAT%20Pretty"

# Query transactions
curl "http://localhost:8123/?query=SELECT%20hash,block_number,datetime,from,to,value%20FROM%20sweet_potatoe_dbt.raw_transactions%20ORDER%20BY%20datetime%20DESC%20LIMIT%205%20FORMAT%20Pretty"

# Query receipts
curl "http://localhost:8123/?query=SELECT%20transaction_hash,block_number,datetime,status,gas_used%20FROM%20sweet_potatoe_dbt.raw_receipts%20ORDER%20BY%20datetime%20DESC%20LIMIT%205%20FORMAT%20Pretty"
```

### Troubleshooting
- If dbt can't find the JSON files, verify the paths in the model SQL files
- If you get permission errors, ensure ClickHouse has access to the raw data directory
- If you need to reset the tables, you can run:
```sql
DROP TABLE IF EXISTS sweet_potatoe_dbt.raw_blocks;
DROP TABLE IF EXISTS sweet_potatoe_dbt.raw_transactions;
DROP TABLE IF EXISTS sweet_potatoe_dbt.raw_receipts;
```

### Quick Query Scripts
You can use the provided bash scripts to quickly query the data:

```bash
# Make scripts executable
chmod +x query_blocks.sh query_transactions.sh query_receipts.sh

# Query blocks
./query_blocks.sh

# Query transactions
./query_transactions.sh

# Query receipts
./query_receipts.sh
```