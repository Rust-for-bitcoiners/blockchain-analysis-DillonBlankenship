use std::{env, time};
use bitcoincore_rpc::{json, jsonrpc::{self}, Auth, Client, RpcApi};
use chrono::Duration;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref RPC_CLIENT: Client = {
        dotenv::dotenv().ok();
        let rpc_url: String = env::var("BITCOIN_RPC_URL").expect("BITCOIN_RPC_URL must be set");
        let rpc_user: String = env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER must be set");
        let rpc_password: String =
            env::var("BITCOIN_RPC_PASSWORD").expect("BITCOIN_RPC_PASSWORD must be set");
        Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password)).unwrap()
    };
}

fn time_to_mine(block_height: u64) -> Result<Duration, Box<dyn std::error::Error>> {
    let rpc_client: &Client = &*RPC_CLIENT;
    let block_hash = rpc_client.get_block_hash(block_height)?;
    let block = rpc_client.get_block(&block_hash)?;
    let prev_block_hash = block.header.prev_blockhash;
    let prev_block = rpc_client.get_block(&prev_block_hash)?;
    
    let time_diff = block.header.time - prev_block.header.time;
    Ok(Duration::seconds(time_diff as i64))
}

fn number_of_transactions(block_height: u64) -> Result<u16, Box<dyn std::error::Error>> {
    let rpc_client: &Client = &*RPC_CLIENT;
    let block_hash = rpc_client.get_block_hash(block_height)?;
    let block = rpc_client.get_block(&block_hash)?;
    
    Ok(block.txdata.len() as u16)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const TIMEOUT_UTXO_SET_SCANS: time::Duration = time::Duration::from_secs(60 * 8); // 8 minutes
    dotenv::dotenv().ok();
    let rpc_url: String = env::var("BITCOIN_RPC_URL").expect("BITCOIN_RPC_URL must be set");
    let rpc_user: String = env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER must be set");
    let rpc_password: String =
        env::var("BITCOIN_RPC_PASSWORD").expect("BITCOIN_RPC_PASSWORD must be set");

    let custom_timeout_transport = jsonrpc::simple_http::Builder::new()
        .url(&rpc_url)
        .expect("invalid rpc url")
        .auth(rpc_user, Some(rpc_password))
        .timeout(TIMEOUT_UTXO_SET_SCANS)
        .build();
    let custom_timeout_rpc_client =
        jsonrpc::client::Client::with_transport(custom_timeout_transport);

    let rpc_client = Client::from_jsonrpc(custom_timeout_rpc_client);
    let res: json::GetTxOutSetInfoResult =
        rpc_client.get_tx_out_set_info(None, None, None)?;
    println!("{:?}", res);

    // Query the latest block height
    let latest_block_height = rpc_client.get_block_count()?;
    println!("Latest block height: {}", latest_block_height);

    // Example usage of the implemented functions with the latest block height
    let time_diff = time_to_mine(latest_block_height)?;
    println!("Time to mine block {}: {:?}", latest_block_height, time_diff);

    let num_tx = number_of_transactions(latest_block_height)?;
    println!("Number of transactions in block {}: {}", latest_block_height, num_tx);

    Ok(())
}
