use anyhow::Result;
use serde_json::{json, Value};
use std::env;

const RPC_URL: &str = "https://rpc.sepolia.linea.build";

async fn get_block(number: u64) -> Result<Value> {
    let client = reqwest::Client::new();
    let hex_number = format!("0x{:x}", number);
    
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
    match data.get("result") {
        Some(result) => Ok(result.clone()),
        None => Err(anyhow::anyhow!("No result field in response"))
    }
}

async fn get_block_receipts(number: u64) -> Result<Value> {
    let client = reqwest::Client::new();
    let hex_number = format!("0x{:x}", number);
    
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
    match data.get("result") {
        Some(result) => Ok(result.clone()),
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
