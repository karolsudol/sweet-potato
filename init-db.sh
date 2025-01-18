#!/bin/bash
set -e

# Wait for ClickHouse to be ready
until clickhouse-client --query "SELECT 1"; do
    echo "Waiting for ClickHouse to be ready..."
    sleep 1
done

# Run the SQL files in order
clickhouse-client --multiquery < /docker-entrypoint-initdb.d/cleanup.sql
clickhouse-client --multiquery < /docker-entrypoint-initdb.d/load_blocks.sql
clickhouse-client --multiquery < /docker-entrypoint-initdb.d/load_transactions.sql
clickhouse-client --multiquery < /docker-entrypoint-initdb.d/load_receipts.sql 