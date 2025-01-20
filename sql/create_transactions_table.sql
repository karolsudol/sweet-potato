CREATE TABLE IF NOT EXISTS raw.transactions
(
    `blockTimestamp` DateTime,
    `blockHash` Nullable(String),
    `blockNumber` Int64,
    `from` String,
    `gas` String,
    `gasPrice` String,
    `hash` String,
    `input` String,
    `nonce` String,
    `to` Nullable(String),
    `transactionIndex` Int64,
    `value` String,
    `type` String,
    `v` String,
    `r` String,
    `s` String,
    `maxFeePerGas` Nullable(String),
    `maxPriorityFeePerGas` Nullable(String)
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(blockTimestamp)
ORDER BY blockTimestamp;