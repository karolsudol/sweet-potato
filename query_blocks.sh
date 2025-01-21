#!/bin/bash

# Load environment variables
source .env

# Execute query
curl "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SELECT%20number,hash,datetime,gas_used%20FROM%20sweet_potatoe_dbt.raw_blocks%20ORDER%20BY%20number%20DESC%20LIMIT%205%20FORMAT%20Pretty" 