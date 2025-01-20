CREATE TABLE IF NOT EXISTS raw.blocks
(
    `timestamp` DateTime,
    `number` Int64,
    `baseFeePerGas` Nullable(String),
    `difficulty` Nullable(String),
    `extraData` Nullable(String),
    `gasLimit` Nullable(String),
    `gasUsed` Nullable(String),
    `hash` String,
    `logsBloom` Nullable(String),
    `miner` Nullable(String),
    `mixHash` Nullable(String),
    `nonce` Nullable(String),
    `parentHash` Nullable(String),
    `receiptsRoot` Nullable(String),
    `sha3Uncles` Nullable(String),
    `size` Nullable(String),
    `stateRoot` Nullable(String),
    `totalDifficulty` Nullable(String),
    `transactionsRoot` Nullable(String),
    `uncles` Array(Nullable(String))
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY timestamp 