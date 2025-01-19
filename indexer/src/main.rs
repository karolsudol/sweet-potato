use anyhow::Result;
use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, time::Duration};

#[derive(Debug, Serialize, Deserialize, Row)]
struct Block {
    timestamp: DateTime<Utc>,
    number: i64,
    base_fee_per_gas: Option<i128>,
    difficulty: Option<i128>,
    extra_data: Option<String>,
    gas_limit: Option<i128>,
    gas_used: Option<i128>,
    hash: String,
    logs_bloom: Option<String>,
    miner: Option<String>,
    mix_hash: Option<String>,
    nonce: Option<String>,
    parent_hash: Option<String>,
    receipts_root: Option<String>,
    sha3_uncles: Option<String>,
    size: Option<i128>,
    state_root: Option<String>,
    total_difficulty: Option<i128>,
    transactions_root: Option<String>,
    uncles: Vec<Option<String>>,
}

#[derive(Debug, Serialize, Deserialize, Row)]
struct Receipt {
    block_number: i64,
    block_timestamp: DateTime<Utc>,
    block_hash: String,
    contract_address: String,
    cumulative_gas_used: i128,
    effective_gas_price: i128,
    from: String,
    gas_used: i128,
    logs: Vec<Log>,
    logs_bloom: String,
    status: String,
    to: String,
    transaction_hash: String,
    transaction_index: i64,
    type_field: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Log {
    address: String,
    block_hash: String,
    block_number: i64,
    data: String,
    log_index: String,
    removed: bool,
    topics: Vec<String>,
    transaction_hash: String,
    transaction_index: String,
}

struct Indexer {
    client: Client,
    print_output: bool,
}

impl Indexer {
    async fn new(clickhouse_url: &str, print_output: bool) -> Result<Self> {
        let username = env::var("CLICKHOUSE_USER")
            .unwrap_or_else(|_| "default".to_string());
        let password = env::var("CLICKHOUSE_PASSWORD")
            .unwrap_or_else(|_| "password".to_string());

        let client = Client::default()
            .with_url(clickhouse_url)
            .with_user(username)
            .with_password(password);
        
        // Create database first
        client.query(include_str!("../../sql/create_database.sql"))
            .execute()
            .await?;

        // Update client to use the raw database
        let client = client.with_database("raw");
        
        // Initialize tables
        let table_queries = [
            include_str!("../../sql/create_blocks_table.sql"), 
            include_str!("../../sql/create_receipts_table.sql"),
            include_str!("../../sql/create_transactions_table.sql"),
        ];

        for query in table_queries {
            client.query(query).execute().await?;
        }

        println!("Database and tables initialized");
        
        Ok(Self {
            client,
            print_output,
        })
    }

    async fn process_blocks(&self, start: u64, count: u64) -> Result<()> {
        let mut block_inserter = self.client.inserter("blocks")?
            .with_timeouts(Some(Duration::from_secs(5)), Some(Duration::from_secs(20)));

        let mut receipt_inserter = self.client.inserter("receipts")?
            .with_timeouts(Some(Duration::from_secs(5)), Some(Duration::from_secs(20)));

        for block_number in start..start + count {
            println!("\nProcessing block {}", block_number);
            
            match get_block(block_number).await {
                Ok(block_data) => {
                    if self.print_output {
                        println!("Block data: {}", serde_json::to_string_pretty(&block_data)?);
                    }
                    
                    let block: Block = serde_json::from_value(block_data)?;
                    // Use load_blocks.sql query template
                    block_inserter.write(&block).await?;
                    println!("Block {} inserted into database", block_number);
                },
                Err(e) => {
                    eprintln!("Error fetching block {}: {}", block_number, e);
                    continue;
                }
            };

            if let Ok(receipts_data) = get_block_receipts(block_number).await {
                if self.print_output {
                    println!("Block receipts: {}", serde_json::to_string_pretty(&receipts_data)?);
                }
                
                let receipts: Vec<Receipt> = serde_json::from_value(receipts_data)?;
                // Use load_receipts.sql query template
                for receipt in receipts {
                    receipt_inserter.write(&receipt).await?;
                }
                println!("Receipts for block {} inserted into database", block_number);
            }
        }

        // Commit remaining data
        let block_stats = block_inserter.end().await?;
        let receipt_stats = receipt_inserter.end().await?;

        println!("Blocks inserted: {} entries", block_stats.entries);
        println!("Receipts inserted: {} entries", receipt_stats.entries);

        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        self.client
            .query(include_str!("../../sql/cleanup.sql"))
            .execute()
            .await?;
        Ok(())
    }
}

fn get_rpc_url() -> String {
    env::var("RPC_URL")
        .unwrap_or_else(|_| "https://rpc.sepolia.linea.build".to_string())
}

async fn get_block(number: u64) -> Result<Value> {
    let client = reqwest::Client::new();
    let hex_number = format!("0x{:x}", number);
    
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
        Some(result) => Ok(result.clone()),
        None => Err(anyhow::anyhow!("No result field in response"))
    }
}

async fn get_block_receipts(number: u64) -> Result<Value> {
    let client = reqwest::Client::new();
    let hex_number = format!("0x{:x}", number);
    
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
        Some(result) => Ok(result.clone()),
        None => Err(anyhow::anyhow!("No result field in response"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env from project root
    dotenv::from_path("../.env").ok();
    
    let start = env::var("START")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()?;
    
    let count = env::var("COUNT")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()?;

    let clickhouse_url = env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8123".to_string());

    let print_output = env::var("PRINT_OUTPUT")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()?;

    println!("Starting indexing from block {} for {} blocks", start, count);

    let indexer = Indexer::new(&clickhouse_url, print_output).await?;
    indexer.process_blocks(start, count).await?;
    // indexer.cleanup().await?;

    Ok(())
}
