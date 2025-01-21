{{ config(
    materialized='table',
    engine='MergeTree()',
    order_by=['number', 'hash'],
    unique_key='hash'
) }}

with source as (
    select * from file('../indexer/raw_data/blocks/*.json', 'JSONEachRow', 
    'base_fee_per_gas Nullable(UInt64),
     difficulty UInt64,
     extra_data String,
     gas_limit UInt64,
     gas_used UInt64,
     hash String,
     logs_bloom String,
     miner String,
     mix_hash String,
     nonce String,
     number UInt64,
     parent_hash String,
     receipts_root String,
     sha3_uncles String,
     size UInt64,
     state_root String,
     datetime DateTime,
     total_difficulty UInt64,
     transaction_hashes Array(String),
     transactions_root String,
     uncles Array(String)')
)

select * from source 