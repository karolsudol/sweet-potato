use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::time::Instant;
use chrono::{DateTime, Utc, TimeZone};
use std::fs;
use std::path::Path;

const RPC_URL: &str = match option_env!("RPC_URL") {
    Some(url) => url,
    None => "https://rpc.sepolia.linea.build",
};

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

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct TransformedReceipt {
    block_hash: String,
    block_number: u64,
    contract_address: Option<String>,
    cumulative_gas_used: u64,
    effective_gas_price: u64,
    from: String,
    gas_used: u64,
    logs: Vec<Value>,
    logs_bloom: String,
    status: bool,
    to: Option<String>,
    transaction_hash: String,
    transaction_index: u64,
    tx_type: u64,
    datetime: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct TransformedTransaction {
    block_hash: String,
    block_number: u64,
    chain_id: u64,
    from: String,
    gas: u64,
    gas_price: u64,
    hash: String,
    input: String,
    nonce: u64,
    r: String,
    s: String,
    to: Option<String>,
    transaction_index: u64,
    tx_type: u64,
    v: String,
    value: u64,
    datetime: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct TransformedBlock {
    base_fee_per_gas: Option<u64>,
    difficulty: u64,
    extra_data: String,
    gas_limit: u64,
    gas_used: u64,
    hash: String,
    logs_bloom: String,
    miner: String,
    mix_hash: String,
    nonce: String,
    number: u64,
    parent_hash: String,
    receipts_root: String,
    sha3_uncles: String,
    size: u64,
    state_root: String,
    datetime: DateTime<Utc>,
    total_difficulty: u64,
    transaction_hashes: Vec<String>,
    transactions_root: String,
    uncles: Vec<String>,
}

// Helper functions
fn hex_to_u64(hex: &str) -> u64 {
    if let Some(hex_str) = hex.strip_prefix("0x") {
        u64::from_str_radix(hex_str, 16).unwrap_or(0)
    } else {
        u64::from_str_radix(hex, 16).unwrap_or(0)
    }
}

fn hex_to_bool(hex: &str) -> bool {
    hex_to_u64(hex) == 1
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

// Add this function near other helper functions
fn ensure_directory(path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::from_path("../.env").ok();
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
    log::info!("Original Receipt Sets: {} (containing {} total receipts) | Transformed Receipt Sets: {} (containing {} total receipts)", 
        all_receipts.len(),
        all_receipts.iter().map(|r| r.len()).sum::<usize>(),
        all_receipts.len(),
        all_receipts.iter().map(|r| r.len()).sum::<usize>());
    
    // Only print detailed data when debug level is enabled
    log::debug!("\n=== All Blocks ===");
    log::debug!("{:#?}", all_blocks);
    
    log::debug!("\n=== All Transactions ===");
    log::debug!("{:#?}", all_transactions);
    
    log::debug!("\n=== All Receipts ===");
    log::debug!("{:#?}", all_receipts);

    // Transform the data
    log::info!("Converting hex values to appropriate types...");
    
    let transformed_blocks: Vec<TransformedBlock> = all_blocks.iter().map(|block| {
        let ts = hex_to_u64(&block.timestamp);
        let datetime = Utc.timestamp_opt(ts as i64, 0).single().unwrap_or_default();
        TransformedBlock {
            base_fee_per_gas: block.base_fee_per_gas.as_ref().map(|x| hex_to_u64(x)),
            difficulty: hex_to_u64(&block.difficulty),
            extra_data: block.extra_data.clone(),
            gas_limit: hex_to_u64(&block.gas_limit),
            gas_used: hex_to_u64(&block.gas_used),
            hash: block.hash.clone(),
            logs_bloom: block.logs_bloom.clone(),
            miner: block.miner.clone(),
            mix_hash: block.mix_hash.clone(),
            nonce: block.nonce.clone(),
            number: hex_to_u64(&block.number),
            parent_hash: block.parent_hash.clone(),
            receipts_root: block.receipts_root.clone(),
            sha3_uncles: block.sha3_uncles.clone(),
            size: hex_to_u64(&block.size),
            state_root: block.state_root.clone(),
            datetime,
            total_difficulty: hex_to_u64(&block.total_difficulty),
            transaction_hashes: block.transaction_hashes.clone(),
            transactions_root: block.transactions_root.clone(),
            uncles: block.uncles.clone(),
        }
    }).collect();

    let transformed_transactions: Vec<TransformedTransaction> = all_transactions.iter().map(|tx| {
        let block_number = hex_to_u64(&tx.block_number);
        let block = transformed_blocks
            .iter()
            .find(|b| b.number == block_number)
            .unwrap_or(&transformed_blocks[0]);
        
        TransformedTransaction {
            block_hash: tx.block_hash.clone(),
            block_number: hex_to_u64(&tx.block_number),
            chain_id: hex_to_u64(&tx.chain_id),
            from: tx.from.clone(),
            gas: hex_to_u64(&tx.gas),
            gas_price: hex_to_u64(&tx.gas_price),
            hash: tx.hash.clone(),
            input: tx.input.clone(),
            nonce: hex_to_u64(&tx.nonce),
            r: tx.r.clone(),
            s: tx.s.clone(),
            to: tx.to.clone(),
            transaction_index: hex_to_u64(&tx.transaction_index),
            tx_type: hex_to_u64(&tx.tx_type),
            v: tx.v.clone(),
            value: hex_to_u64(&tx.value),
            datetime: block.datetime,
        }
    }).collect();

    let transformed_receipts: Vec<Vec<TransformedReceipt>> = all_receipts.iter().map(|block_receipts| {
        block_receipts.iter().map(|receipt| {
            let block_number = hex_to_u64(&receipt.block_number);
            let block = transformed_blocks
                .iter()
                .find(|b| b.number == block_number)
                .unwrap_or(&transformed_blocks[0]);

            TransformedReceipt {
                block_hash: receipt.block_hash.clone(),
                block_number: hex_to_u64(&receipt.block_number),
                contract_address: receipt.contract_address.clone(),
                cumulative_gas_used: hex_to_u64(&receipt.cumulative_gas_used),
                effective_gas_price: hex_to_u64(&receipt.effective_gas_price),
                from: receipt.from.clone(),
                gas_used: hex_to_u64(&receipt.gas_used),
                logs: receipt.logs.clone(),
                logs_bloom: receipt.logs_bloom.clone(),
                status: hex_to_bool(&receipt.status),
                to: receipt.to.clone(),
                transaction_hash: receipt.transaction_hash.clone(),
                transaction_index: hex_to_u64(&receipt.transaction_index),
                tx_type: hex_to_u64(&receipt.tx_type),
                datetime: block.datetime,
            }
        }).collect()
    }).collect();

    // Print comparison of original and transformed data
    log::info!("\n=== Data Transformation Results ===");
    log::info!("Original Blocks: {} | Transformed Blocks: {}", 
        all_blocks.len(), transformed_blocks.len());
    log::info!("Original Transactions: {} | Transformed Transactions: {}", 
        all_transactions.len(), transformed_transactions.len());
    log::info!("Original Receipt Sets: {} (containing {} total receipts) | Transformed Receipt Sets: {} (containing {} total receipts)", 
        all_receipts.len(),
        all_receipts.iter().map(|r| r.len()).sum::<usize>(),
        transformed_receipts.len(),
        transformed_receipts.iter().map(|r| r.len()).sum::<usize>());

    // Print detailed transformed data when debug is enabled
    log::debug!("\n=== Transformed Blocks ===");
    log::debug!("{:#?}", transformed_blocks);
    
    log::debug!("\n=== Transformed Transactions ===");
    log::debug!("{:#?}", transformed_transactions);
    
    log::debug!("\n=== Transformed Receipts ===");
    log::debug!("{:#?}", transformed_receipts);

    // Get RAW_DATA_PATH from environment
    let raw_data_path = env::var("RAW_DATA_PATH")
        .unwrap_or_else(|_| "./raw_data".to_string());

    // Ensure directories exist
    let blocks_dir = format!("{}/blocks", raw_data_path);
    let transactions_dir = format!("{}/transactions", raw_data_path);
    let receipts_dir = format!("{}/receipts", raw_data_path);

    ensure_directory(&blocks_dir)?;
    ensure_directory(&transactions_dir)?;
    ensure_directory(&receipts_dir)?;

    // Save transformed blocks
    for block in &transformed_blocks {
        let filename = format!("{}/block_{}.json", blocks_dir, block.number);
        let json = serde_json::to_string_pretty(block)?;
        fs::write(filename, json)?;
    }

    // Save transformed transactions
    for tx in &transformed_transactions {
        let filename = format!("{}/tx_{}.json", transactions_dir, tx.hash);
        let json = serde_json::to_string_pretty(tx)?;
        fs::write(filename, json)?;
    }

    // Save transformed receipts
    for (block_index, receipt_block) in transformed_receipts.iter().enumerate() {
        for receipt in receipt_block {
            let filename = format!("{}/receipt_{}.json", receipts_dir, receipt.transaction_hash);
            let json = serde_json::to_string_pretty(receipt)?;
            fs::write(filename, json)?;
        }
    }

    log::info!("Data saved to directories:");
    log::info!("  Blocks: {}", blocks_dir);
    log::info!("  Transactions: {}", transactions_dir);
    log::info!("  Receipts: {}", receipts_dir);

    Ok(())
}
