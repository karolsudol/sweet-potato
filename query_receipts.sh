#!/bin/bash

# Load environment variables
source .env

# Execute query
curl "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SELECT%20transaction_hash,block_number,datetime,status,gas_used%20FROM%20sweet_potatoe_dbt.raw_receipts%20ORDER%20BY%20datetime%20DESC%20LIMIT%205%20FORMAT%20Pretty" 