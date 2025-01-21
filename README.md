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