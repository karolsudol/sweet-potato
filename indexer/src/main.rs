use anyhow::Result;
use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, time::{Duration, Instant}};
use tracing::{info, error, Level};
use tracing_subscriber::{FmtSubscriber, EnvFilter};

#[derive(Debug, Serialize, Deserialize, Row)]
struct Block {
    timestamp: DateTime<Utc>,
    number: i64,
    #[serde(rename = "baseFeePerGas")]
    base_fee_per_gas: Option<i128>,
    difficulty: Option<i128>,
    #[serde(rename = "extraData")]
    extra_data: Option<String>,
    #[serde(rename = "gasLimit")]
    gas_limit: Option<i128>,
    #[serde(rename = "gasUsed")]
    gas_used: Option<i128>,
    hash: String,
    #[serde(rename = "logsBloom")]
    logs_bloom: Option<String>,
    miner: Option<String>,
    #[serde(rename = "mixHash")]
    mix_hash: Option<String>,
    nonce: Option<String>,
    #[serde(rename = "parentHash")]
    parent_hash: Option<String>,
    #[serde(rename = "receiptsRoot")]
    receipts_root: Option<String>,
    #[serde(rename = "sha3Uncles")]
    sha3_uncles: Option<String>,
    size: Option<i128>,
    #[serde(rename = "stateRoot")]
    state_root: Option<String>,
    #[serde(rename = "totalDifficulty")]
    total_difficulty: Option<i128>,
    #[serde(rename = "transactionsRoot")]
    transactions_root: Option<String>,
    uncles: Vec<Option<String>>,
}

#[derive(Debug, Serialize, Deserialize, Row)]
struct Receipt {
    #[serde(rename = "blockNumber")]
    block_number: i64,
    #[serde(rename = "blockTimestamp")]
    block_timestamp: DateTime<Utc>,
    #[serde(rename = "blockHash")]
    block_hash: String,
    #[serde(rename = "contractAddress")]
    contract_address: Option<String>,
    #[serde(rename = "cumulativeGasUsed")]
    cumulative_gas_used: i128,
    #[serde(rename = "effectiveGasPrice")]
    effective_gas_price: i128,
    from: String,
    #[serde(rename = "gasUsed")]
    gas_used: i128,
    logs: Vec<Log>,
    #[serde(rename = "logsBloom")]
    logs_bloom: String,
    status: String,
    to: Option<String>,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    #[serde(rename = "transactionIndex")]
    transaction_index: i64,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Log {
    address: String,
    #[serde(rename = "blockHash")]
    block_hash: String,
    #[serde(rename = "blockNumber")]
    block_number: i64,
    data: String,
    #[serde(rename = "logIndex")]
    log_index: String,
    removed: bool,
    topics: Vec<String>,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    #[serde(rename = "transactionIndex")]
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
            let block_time = Instant::now();
            info!(block_number, "Processing block");
            
            match get_block(block_number).await {
                Ok(block_data) => {
                    if self.print_output {
                        info!(block_number, "Retrieved block data");
                    }
                    
                    let block_data = convert_block_hex_to_decimal(block_data);
                    let block: Block = serde_json::from_value(block_data)?;
                    block_inserter.write(&block).await?;
                    info!(
                        block_number,
                        elapsed_ms = block_time.elapsed().as_millis(),
                        "Block inserted into database"
                    );
                },
                Err(e) => {
                    error!(
                        block_number,
                        error = %e,
                        "Failed to fetch block"
                    );
                    continue;
                }
            };

            let receipt_time = Instant::now();
            match get_block_receipts(block_number).await {
                Ok(receipts_data) => {
                    if self.print_output {
                        info!(block_number, "Retrieved block receipts");
                    }
                    
                    let receipts_data = match convert_receipts_hex_to_decimal(receipts_data, block_number).await {
                        Ok(data) => data,
                        Err(e) => {
                            error!(
                                block_number,
                                error = %e,
                                "Failed to convert receipts hex to decimal"
                            );
                            continue;
                        }
                    };

                    let receipts: Vec<Receipt> = match serde_json::from_value(receipts_data.clone()) {
                        Ok(r) => r,
                        Err(e) => {
                            error!(
                                block_number,
                                error = %e,
                                raw_data = ?receipts_data,
                                "Failed to deserialize receipts data"
                            );
                            continue;
                        }
                    };

                    for receipt in receipts {
                        receipt_inserter.write(&receipt).await?;
                    }
                    info!(
                        block_number,
                        elapsed_ms = receipt_time.elapsed().as_millis(),
                        "Receipts inserted into database"
                    );
                }
                Err(e) => {
                    error!(
                        block_number,
                        error = %e,
                        "Failed to fetch block receipts"
                    );
                }
            }
        }

        // Commit remaining data
        let block_stats = block_inserter.end().await?;
        let receipt_stats = receipt_inserter.end().await?;

        info!(
            blocks_inserted = block_stats.entries,
            receipts_inserted = receipt_stats.entries,
            "Insertion completed"
        );

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
    
    // Add debug logging for raw response
    info!("Raw block response: {}", serde_json::to_string_pretty(&data).unwrap());
    
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
    
    // Add debug logging for raw response
    info!("Raw receipts response: {}", serde_json::to_string_pretty(&data).unwrap());
    
    match data.get("result") {
        Some(result) => Ok(result.clone()),
        None => Err(anyhow::anyhow!("No result field in response"))
    }
}

// Add this new function to convert hex values to decimals
fn convert_block_hex_to_decimal(mut block: Value) -> Value {
    if let Some(obj) = block.as_object_mut() {
        // Convert numeric fields from hex to decimal
        if let Some(difficulty) = obj.get_mut("difficulty") {
            if let Some(hex) = difficulty.as_str() {
                if let Ok(val) = i128::from_str_radix(&hex[2..], 16) {
                    *difficulty = json!(val);
                }
            }
        }
        
        if let Some(base_fee) = obj.get_mut("baseFeePerGas") {
            if let Some(hex) = base_fee.as_str() {
                if let Ok(val) = i128::from_str_radix(&hex[2..], 16) {
                    *base_fee = json!(val);
                }
            }
        }

        if let Some(gas_limit) = obj.get_mut("gasLimit") {
            if let Some(hex) = gas_limit.as_str() {
                if let Ok(val) = i128::from_str_radix(&hex[2..], 16) {
                    *gas_limit = json!(val);
                }
            }
        }

        if let Some(gas_used) = obj.get_mut("gasUsed") {
            if let Some(hex) = gas_used.as_str() {
                if let Ok(val) = i128::from_str_radix(&hex[2..], 16) {
                    *gas_used = json!(val);
                }
            }
        }

        if let Some(number) = obj.get_mut("number") {
            if let Some(hex) = number.as_str() {
                if let Ok(val) = i64::from_str_radix(&hex[2..], 16) {
                    *number = json!(val);
                }
            }
        }

        if let Some(size) = obj.get_mut("size") {
            if let Some(hex) = size.as_str() {
                if let Ok(val) = i128::from_str_radix(&hex[2..], 16) {
                    *size = json!(val);
                }
            }
        }

        if let Some(total_difficulty) = obj.get_mut("totalDifficulty") {
            if let Some(hex) = total_difficulty.as_str() {
                if let Ok(val) = i128::from_str_radix(&hex[2..], 16) {
                    *total_difficulty = json!(val);
                }
            }
        }

        if let Some(timestamp) = obj.get_mut("timestamp") {
            if let Some(hex) = timestamp.as_str() {
                if let Ok(val) = i64::from_str_radix(&hex[2..], 16) {
                    let datetime = DateTime::from_timestamp(val, 0)
                        .unwrap_or_default();
                    *timestamp = json!(datetime);
                }
            }
        }
    }
    block
}

// Change function signature to async
async fn convert_receipts_hex_to_decimal(receipts: Value, block_number: u64) -> Result<Value> {
    let mut receipts_array = receipts.as_array().cloned()
        .ok_or_else(|| anyhow::anyhow!("Receipts data is not an array"))?;

    // Get block timestamp first
    let block_data = get_block(block_number).await?;
    let block_data = convert_block_hex_to_decimal(block_data);
    let timestamp = block_data["timestamp"].clone();

    for receipt in receipts_array.iter_mut() {
        if let Some(obj) = receipt.as_object_mut() {
            // Remove the old fields first
            obj.remove("block_number");
            obj.remove("block_timestamp");
            
            // Add block_number and blockTimestamp fields with correct names
            obj.insert("blockNumber".to_string(), json!(block_number as i64));
            obj.insert("blockTimestamp".to_string(), timestamp.clone());

            // Convert numeric fields from hex to decimal
            if let Some(cumulative_gas) = obj.get_mut("cumulativeGasUsed") {
                if let Some(hex) = cumulative_gas.as_str() {
                    if let Ok(val) = i128::from_str_radix(&hex[2..], 16) {
                        *cumulative_gas = json!(val);
                    }
                }
            }

            if let Some(effective_gas) = obj.get_mut("effectiveGasPrice") {
                if let Some(hex) = effective_gas.as_str() {
                    if let Ok(val) = i128::from_str_radix(&hex[2..], 16) {
                        *effective_gas = json!(val);
                    }
                }
            }

            if let Some(gas_used) = obj.get_mut("gasUsed") {
                if let Some(hex) = gas_used.as_str() {
                    if let Ok(val) = i128::from_str_radix(&hex[2..], 16) {
                        *gas_used = json!(val);
                    }
                }
            }

            if let Some(transaction_index) = obj.get_mut("transactionIndex") {
                if let Some(hex) = transaction_index.as_str() {
                    if let Ok(val) = i64::from_str_radix(&hex[2..], 16) {
                        *transaction_index = json!(val);
                    }
                }
            }

            // Convert log indices from hex to decimal
            if let Some(logs) = obj.get_mut("logs").and_then(Value::as_array_mut) {
                for log in logs {
                    if let Some(log_obj) = log.as_object_mut() {
                        if let Some(log_index) = log_obj.get_mut("logIndex") {
                            if let Some(hex) = log_index.as_str() {
                                if let Ok(val) = i64::from_str_radix(&hex[2..], 16) {
                                    *log_index = json!(val.to_string());
                                }
                            }
                        }
                        if let Some(block_number) = log_obj.get_mut("blockNumber") {
                            if let Some(hex) = block_number.as_str() {
                                if let Ok(val) = i64::from_str_radix(&hex[2..], 16) {
                                    *block_number = json!(val);
                                }
                            }
                        }
                        if let Some(tx_index) = log_obj.get_mut("transactionIndex") {
                            if let Some(hex) = tx_index.as_str() {
                                if let Ok(val) = i64::from_str_radix(&hex[2..], 16) {
                                    *tx_index = json!(val.to_string());
                                }
                            }
                        }
                    }
                }
            }

            // Handle the type field - make sure it's a plain string without 0x prefix
            if let Some(type_val) = obj.get_mut("type") {
                if let Some(hex) = type_val.as_str() {
                    if hex.starts_with("0x") {
                        *type_val = json!(hex[2..].to_string());
                    }
                }
            }
        }
    }

    Ok(Value::Array(receipts_array))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with updated timer configuration
    let _subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
        .pretty()
        .try_init()
        .expect("Failed to set tracing subscriber");

    info!("Starting indexer application");
    
    // Load .env from project root
    dotenv::from_path("../.env").ok();
    info!("Loaded environment variables");
    
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

    info!(
        start_block = start,
        block_count = count,
        clickhouse_url = %clickhouse_url,
        "Indexer configuration loaded"
    );

    let start_time = Instant::now();
    info!("Initializing indexer...");
    let indexer = Indexer::new(&clickhouse_url, print_output).await?;
    info!(
        elapsed_ms = start_time.elapsed().as_millis(),
        "Indexer initialized"
    );

    let process_time = Instant::now();
    info!("Starting block processing...");
    indexer.process_blocks(start, count).await?;
    info!(
        elapsed_sec = process_time.elapsed().as_secs(),
        "Block processing completed"
    );

    let cleanup_time = Instant::now();
    info!("Running cleanup...");
    indexer.cleanup().await?;
    info!(
        elapsed_ms = cleanup_time.elapsed().as_millis(),
        "Cleanup completed"
    );

    info!(
        total_elapsed_sec = start_time.elapsed().as_secs(),
        "Indexer completed successfully"
    );

    Ok(())
}
