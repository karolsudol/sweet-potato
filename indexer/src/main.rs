use anyhow::Result;
use serde_json::{json, Value};
use std::env;

const RPC_URL: &str = "https://rpc.sepolia.linea.build";

async fn get_block(number: u64) -> Result<Value> {
    let client = reqwest::Client::new();
    let hex_number = format!("0x{:x}", number);
    
    log::info!("Fetching block {}", number);
    
    let response = client
        .post(&get_rpc_url())
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getBlockByNumber",
            "params": [hex_number, true]
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
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
        .post(&get_rpc_url())
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getBlockReceipts",
            "params": [hex_number]
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
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
    // Get START and COUNT from environment variables
    let start = env::var("START")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()?;
    
    let count = env::var("COUNT")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()?;

    println!("Starting indexing from block {} for {} blocks", start, count);

    for block_number in start..start + count {
        println!("\nProcessing block {}", block_number);
        
        // Get block data and handle errors more gracefully
        match get_block(block_number).await {
            Ok(_block) => {
                println!("Block data: {}", serde_json::to_string_pretty(&_block)?);
            },
            Err(e) => {
                eprintln!("Error fetching block {}: {}", block_number, e);
                continue;
            }
        };

        // Only fetch receipts if we successfully got the block
        if let Err(e) = get_block_receipts(block_number).await.and_then(|receipts| {
            println!("Block receipts: {}", serde_json::to_string_pretty(&receipts)?);
            Ok(())
        }) {
            eprintln!("Error fetching receipts for block {}: {}", block_number, e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_hex_to_u64() {
        assert_eq!(hex_to_u64("0x0"), 0);
        assert_eq!(hex_to_u64("0x1"), 1);
        assert_eq!(hex_to_u64("0xa"), 10);
        assert_eq!(hex_to_u64("0xff"), 255);
        assert_eq!(hex_to_u64("ff"), 255); // Test without 0x prefix
        assert_eq!(hex_to_u64("invalid"), 0); // Test invalid input
    }

    #[test]
    fn test_hex_to_bool() {
        assert_eq!(hex_to_bool("0x0"), false);
        assert_eq!(hex_to_bool("0x1"), true);
        assert_eq!(hex_to_bool("0x2"), false); // Any non-1 value should be false
        assert_eq!(hex_to_bool("invalid"), false); // Invalid input should return false
    }

    #[test]
    fn test_block_transformation() {
        let block = Block {
            base_fee_per_gas: Some("0xa".to_string()),
            difficulty: "0x5".to_string(),
            extra_data: "0x".to_string(),
            gas_limit: "0x1234".to_string(),
            gas_used: "0x1000".to_string(),
            hash: "0xabc".to_string(),
            logs_bloom: "0x0".to_string(),
            miner: "0xdef".to_string(),
            mix_hash: "0x123".to_string(),
            nonce: "0x1".to_string(),
            number: "0x1".to_string(),
            parent_hash: "0x456".to_string(),
            receipts_root: "0x789".to_string(),
            sha3_uncles: "0x111".to_string(),
            size: "0x100".to_string(),
            state_root: "0x222".to_string(),
            timestamp: "0x60000000".to_string(), // Unix timestamp in hex
            total_difficulty: "0x10".to_string(),
            transaction_hashes: vec!["0xtx1".to_string()],
            transactions_root: "0x333".to_string(),
            uncles: vec![],
        };

        let transformed = TransformedBlock {
            base_fee_per_gas: Some(10),
            difficulty: 5,
            extra_data: "0x".to_string(),
            gas_limit: 0x1234,
            gas_used: 0x1000,
            hash: "0xabc".to_string(),
            logs_bloom: "0x0".to_string(),
            miner: "0xdef".to_string(),
            mix_hash: "0x123".to_string(),
            nonce: "0x1".to_string(),
            number: 1,
            parent_hash: "0x456".to_string(),
            receipts_root: "0x789".to_string(),
            sha3_uncles: "0x111".to_string(),
            size: 0x100,
            state_root: "0x222".to_string(),
            datetime: Utc.timestamp_opt(0x60000000, 0).unwrap(),
            total_difficulty: 16,
            transaction_hashes: vec!["0xtx1".to_string()],
            transactions_root: "0x333".to_string(),
            uncles: vec![],
        };

        let ts = hex_to_u64(&block.timestamp);
        let datetime = Utc.timestamp_opt(ts as i64, 0).single().unwrap_or_default();
        let result = TransformedBlock {
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
        };

        assert_eq!(result.base_fee_per_gas, transformed.base_fee_per_gas);
        assert_eq!(result.difficulty, transformed.difficulty);
        assert_eq!(result.gas_limit, transformed.gas_limit);
        assert_eq!(result.gas_used, transformed.gas_used);
        assert_eq!(result.number, transformed.number);
        assert_eq!(result.size, transformed.size);
        assert_eq!(result.total_difficulty, transformed.total_difficulty);
    }

    #[test]
    fn test_ensure_directory() {
        let test_dir = "test_dir";
        
        // Clean up any existing test directory
        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir).unwrap();
        }
        
        // Test directory creation
        assert!(!Path::new(test_dir).exists());
        ensure_directory(test_dir).unwrap();
        assert!(Path::new(test_dir).exists());
        
        // Clean up
        fs::remove_dir_all(test_dir).unwrap();
    }
}
