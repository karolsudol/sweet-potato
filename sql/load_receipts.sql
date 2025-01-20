INSERT INTO raw.receipts
SELECT
    block_number AS blockNumber,
    block_timestamp AS blockTimestamp,
    block_hash AS blockHash,
    contract_address AS contractAddress,
    cumulative_gas_used AS cumulativeGasUsed,
    effective_gas_price AS effectiveGasPrice,
    from,
    gas_used AS gasUsed,
    logs,
    logs_bloom AS logsBloom,
    status,
    to,
    transaction_hash AS transactionHash,
    transaction_index AS transactionIndex,
    type_field AS type


