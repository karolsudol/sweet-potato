{{ config(
    materialized='table',
    engine='MergeTree()',
    order_by=['block_number', 'transaction_hash'],
    unique_key='transaction_hash'
) }}

with source as (
    select * from file('../indexer/raw_data/receipts/*.json', 'JSONEachRow',
    'block_hash String,
     block_number UInt64,
     contract_address Nullable(String),
     cumulative_gas_used UInt64,
     effective_gas_price UInt64,
     `from` String,
     gas_used UInt64,
     logs Array(String),
     logs_bloom String,
     status Bool,
     `to` Nullable(String),
     transaction_hash String,
     transaction_index UInt64,
     tx_type UInt64,
     datetime DateTime')
)

select * from source 