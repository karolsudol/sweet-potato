#!/bin/bash

# Load environment variables
source .env

echo "1. Checking databases..."
curl -s "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SHOW%20DATABASES%20FORMAT%20Pretty"

echo -e "\n2. Checking if our database exists..."
curl -s "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SELECT%20name%20FROM%20system.databases%20WHERE%20name='sweet_potatoe_dbt'%20FORMAT%20Pretty"

echo -e "\n3. Checking all tables in sweet_potatoe_dbt..."
curl -s "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=SHOW%20TABLES%20FROM%20sweet_potatoe_dbt%20FORMAT%20Pretty"

echo -e "\n4. Checking table structure..."
curl -s "http://localhost:8123/?user=$CLICKHOUSE_USER&password=$CLICKHOUSE_PASSWORD&query=DESCRIBE%20sweet_potatoe_dbt.raw_blocks%20FORMAT%20Pretty" 