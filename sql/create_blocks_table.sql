CREATE DATABASE IF NOT EXISTS raw;

CREATE TABLE IF NOT EXISTS raw.blocks
(
    `timestamp` DateTime,
    `number` Int64,
    `baseFeePerGas` Nullable(Int256),
    `difficulty` Nullable(Int256),
    `extraData` Nullable(String),
    `gasLimit` Nullable(Int256),
    `gasUsed` Nullable(Int256),
    `hash` String,
    `logsBloom` Nullable(String),
    `miner` Nullable(String),
    `mixHash` Nullable(String),
    `nonce` Nullable(String),
    `parentHash` Nullable(String),
    `receiptsRoot` Nullable(String),
    `sha3Uncles` Nullable(String),
    `size` Nullable(Int256),
    `stateRoot` Nullable(String),
    `totalDifficulty` Nullable(Int256),
    `transactionsRoot` Nullable(String),
    `uncles` Array(Nullable(String))
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY timestamp; 