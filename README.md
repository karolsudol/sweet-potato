# sweet-potato
Node Indexing Pipeline - EVM


## Indexing

```bash
# Basic run
START=1 COUNT=100 cargo run

# With debug output
START=1 COUNT=100 PRINT_OUTPUT=true cargo run

# Custom ClickHouse URL
START=1 COUNT=100 CLICKHOUSE_URL="http://clickhouse:8123" cargo run
```

## DB

```bash
docker compose up -d

curl 'http://localhost:8123/?query=SHOW%20DATABASES'
```