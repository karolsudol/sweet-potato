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

-- Debug information
{{ log("Starting raw_transactions model", info=True) }}
{{ log("Source query file count: " ~ run_query("SELECT count(*) FROM source").columns[0].values()[0], info=True) }}

select 
    *,
    '{{ invocation_id }}' as _invocation_id
from source
{% if is_incremental() %}
where datetime > (select max(datetime) from {{ this }})
{% endif %} 