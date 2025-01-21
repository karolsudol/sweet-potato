{{ config(
    materialized='table',
    engine='MergeTree()',
    order_by=['block_number', 'hash'],
    unique_key='hash'
) }}

with source as (
    select * from file('../indexer/raw_data/transactions/*.json', 'JSONEachRow',
    'block_hash String,
     block_number UInt64,
     chain_id UInt64,
     `from` String,
     gas UInt64,
     gas_price UInt64,
     hash String,
     input String,
     nonce UInt64,
     r String,
     s String,
     `to` Nullable(String),
     transaction_index UInt64,
     tx_type UInt64,
     v String,
     value UInt64,
     datetime DateTime')
)

select * from source 