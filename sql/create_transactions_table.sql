CREATE DATABASE IF NOT EXISTS raw;

CREATE TABLE IF NOT EXISTS raw.transactions
(
    `blockTimestamp` DateTime,
    `blockHash` Nullable(String),
    `blockNumber` Int64,
    `from` String,
    `gas` Int256,
    `gasPrice` Int256,
    `hash` String,
    `input` String,
    `nonce` String,
    `to` String,
    `transactionIndex` Int64,
    `value` Int256,
    `type` Nullable(String),
    `v` String,
    `r` String,
    `s` String,
    `maxFeePerGas` Nullable(Int256),
    `maxPriorityFeePerGas` Nullable(Int256)
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(blockTimestamp)
ORDER BY blockTimestamp; 