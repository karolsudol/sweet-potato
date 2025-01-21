{{ config(
    materialized='table',
    engine='MergeTree()',
    order_by=['block_number', 'hash'],
    unique_key='hash'
) }}

-- Debug information
{{ log("Starting raw_transactions model", info=True) }}

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

select 
    *,
    '{{ invocation_id }}' as _invocation_id
from source
{% if is_incremental() %}
where datetime > (select max(datetime) from {{ this }})
{% endif %} 