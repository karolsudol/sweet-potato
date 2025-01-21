#!/bin/bash

# Load environment variables
source .env

echo "1. Checking raw_blocks table count..."
curl -s "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SELECT%20COUNT(*)%20as%20block_count%20FROM%20sweet_potatoe_dbt.raw_blocks%20FORMAT%20Pretty"

echo -e "\n2. Checking raw_transactions table count..."
curl -s "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SELECT%20COUNT(*)%20as%20tx_count%20FROM%20sweet_potatoe_dbt.raw_transactions%20FORMAT%20Pretty"

echo -e "\n3. Checking raw_receipts table count..."
curl -s "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SELECT%20COUNT(*)%20as%20receipt_count%20FROM%20sweet_potatoe_dbt.raw_receipts%20FORMAT%20Pretty"

echo -e "\n4. Checking raw_blocks table structure..."
curl -s "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SHOW%20CREATE%20TABLE%20sweet_potatoe_dbt.raw_blocks%20FORMAT%20Pretty"

echo -e "\n5. Checking if any data exists in raw_blocks (first row)..."
curl -s "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SELECT%20*%20FROM%20sweet_potatoe_dbt.raw_blocks%20LIMIT%201%20FORMAT%20Pretty" 