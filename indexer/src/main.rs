use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::time::Instant;

const RPC_URL: &str = "https://rpc.sepolia.linea.build";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Transaction {
    #[serde(rename = "blockHash")]
    block_hash: String,
    #[serde(rename = "blockNumber")]
    block_number: String,
    #[serde(rename = "chainId")]
    chain_id: String,
    from: String,
    gas: String,
    #[serde(rename = "gasPrice")]
    gas_price: String,
    hash: String,
    input: String,
    nonce: String,
    r: String,
    s: String,
    to: Option<String>,
    #[serde(rename = "transactionIndex")]
    transaction_index: String,
    #[serde(rename = "type")]
    tx_type: String,
    v: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    #[serde(rename = "baseFeePerGas")]
    base_fee_per_gas: Option<String>,
    difficulty: String,
    #[serde(rename = "extraData")]
    extra_data: String,
    #[serde(rename = "gasLimit")]
    gas_limit: String,
    #[serde(rename = "gasUsed")]
    gas_used: String,
    hash: String,
    #[serde(rename = "logsBloom")]
    logs_bloom: String,
    miner: String,
    #[serde(rename = "mixHash")]
    mix_hash: String,
    nonce: String,
    number: String,
    #[serde(rename = "parentHash")]
    parent_hash: String,
    #[serde(rename = "receiptsRoot")]
    receipts_root: String,
    #[serde(rename = "sha3Uncles")]
    sha3_uncles: String,
    size: String,
    #[serde(rename = "stateRoot")]
    state_root: String,
    timestamp: String,
    #[serde(rename = "totalDifficulty")]
    total_difficulty: String,
    #[serde(rename = "transactions")]
    transaction_hashes: Vec<String>,
    #[serde(rename = "transactionsRoot")]
    transactions_root: String,
    uncles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Receipt {
    #[serde(rename = "blockHash")]
    block_hash: String,
    #[serde(rename = "blockNumber")]
    block_number: String,
    #[serde(rename = "contractAddress")]
    contract_address: Option<String>,
    #[serde(rename = "cumulativeGasUsed")]
    cumulative_gas_used: String,
    #[serde(rename = "effectiveGasPrice")]
    effective_gas_price: String,
    from: String,
    #[serde(rename = "gasUsed")]
    gas_used: String,
    logs: Vec<Value>,
    #[serde(rename = "logsBloom")]
    logs_bloom: String,
    status: String,
    to: Option<String>,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    #[serde(rename = "transactionIndex")]
    transaction_index: String,
    #[serde(rename = "type")]
    tx_type: String,
}

async fn get_block(number: u64) -> Result<(Block, Vec<Transaction>)> {
    let start = Instant::now();
    let client = reqwest::Client::new();
    let hex_number = format!("0x{:x}", number);
    
    log::info!("Fetching block {}", number);
    
    let response = client
        .post(RPC_URL)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getBlockByNumber",
            "params": [hex_number, true]
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    let elapsed = start.elapsed();
    
    match data.get("result") {
        Some(result) => {
            // First, parse the full response to get transactions
            let full_block: Value = result.clone();
            let transactions: Vec<Transaction> = serde_json::from_value(full_block["transactions"].clone())?;
            
            // Then modify the transactions field to only contain hashes
            let mut block_value = result.clone();
            if let Some(txs) = block_value.as_object_mut() {
                let tx_hashes: Vec<String> = transactions.iter()
                    .map(|tx| tx.hash.clone())
                    .collect();
                txs["transactions"] = json!(tx_hashes);
            }
            
            let block: Block = serde_json::from_value(block_value)?;
            log::info!("Block {} fetched in {:?}", number, elapsed);
            Ok((block, transactions))
        },
        None => Err(anyhow::anyhow!("No result field in response"))
    }
}

async fn get_block_receipts(number: u64) -> Result<Vec<Receipt>> {
    let start = Instant::now();
    let client = reqwest::Client::new();
    let hex_number = format!("0x{:x}", number);
    
    log::info!("Fetching receipts for block {}", number);
    
    let response = client
        .post(RPC_URL)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getBlockReceipts",
            "params": [hex_number]
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    let elapsed = start.elapsed();
    
    match data.get("result") {
        Some(result) => {
            let receipts: Vec<Receipt> = serde_json::from_value(result.clone())?;
            log::info!("Receipts for block {} fetched in {:?}", number, elapsed);
            Ok(receipts)
        },
        None => Err(anyhow::anyhow!("No result field in response"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let start_time = Instant::now();
    
    let start = env::var("START")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()?;
    
    let count = env::var("COUNT")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()?;

    log::info!("Starting indexing from block {} for {} blocks", start, count);

    // Create vectors to store all data
    let mut all_blocks = Vec::new();
    let mut all_transactions = Vec::new();
    let mut all_receipts = Vec::new();

    for block_number in start..start + count {
        let block_start = Instant::now();
        log::info!("Processing block {}", block_number);
        
        let (block_result, receipts_result) = tokio::join!(
            get_block(block_number),
            get_block_receipts(block_number)
        );

        match (block_result, receipts_result) {
            (Ok((block, block_transactions)), Ok(receipts)) => {
                log::info!("Block {} processed in {:?}", block_number, block_start.elapsed());
                
                // Store the results
                all_transactions.extend(block_transactions);
                all_blocks.push(block);
                all_receipts.push(receipts);
            },
            (Err(e), _) => {
                log::error!("Error fetching block {}: {}", block_number, e);
            },
            (_, Err(e)) => {
                log::error!("Error fetching receipts for block {}: {}", block_number, e);
            }
        }
    }

    // Print summary with logging levels
    log::info!("=== Processing Summary ===");
    log::info!("Total execution time: {:?}", start_time.elapsed());
    log::info!("Blocks processed: {}", all_blocks.len());
    log::info!("Transactions processed: {}", all_transactions.len());
    log::info!("Total receipts processed: {}", all_receipts.iter().map(|r| r.len()).sum::<usize>());
    
    // Only print detailed data when debug level is enabled
    log::debug!("\n=== All Blocks ===");
    log::debug!("{:#?}", all_blocks);
    
    log::debug!("\n=== All Transactions ===");
    log::debug!("{:#?}", all_transactions);
    
    log::debug!("\n=== All Receipts ===");
    log::debug!("{:#?}", all_receipts);

    Ok(())
}
