CREATE TABLE IF NOT EXISTS raw.receipts
(
    `blockNumber` Int64,
    `blockTimestamp` DateTime,
    `blockHash` String,
    `contractAddress` Nullable(String),
    `cumulativeGasUsed` String,
    `effectiveGasPrice` String,
    `from` String,
    `gasUsed` String,
    `logs` Array(Tuple(
        address String,
        blockHash String,
        blockNumber Int64,
        data String,
        logIndex String,
        removed Bool,
        topics Array(String),
        transactionHash String,
        transactionIndex String)),
    `logsBloom` String,
    `status` String,
    `to` String,
    `transactionHash` String,
    `transactionIndex` Int64,
    `type` String
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(blockTimestamp)
ORDER BY blockTimestamp; 