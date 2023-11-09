use std::env;
use dotenv::dotenv;
use std::time::{SystemTime, UNIX_EPOCH, SystemTimeError};
use serde_json::json;
use arweave_rs::network::NetworkInfoClient;
use reqwest::{Url};

use crate::domain::clients::{uploader::UploaderClient, store::StoreClient};
use crate::domain::core::json::{Message, Process, SortedMessages};
use crate::domain::core::builder::{Builder, BuildResult};


async fn build(input: Vec<u8>) -> Result<BuildResult, String> {
    dotenv().ok();
    let wallet_path = env::var("SU_WALLET_PATH").expect("SU_WALLET_PATH must be set");
    let builder = Builder::new("https://node2.irys.xyz", &wallet_path)?;
    let build_result = builder.build(input).await?;
    Ok(build_result)
}

async fn upload(build_result: Vec<u8>) -> Result<String, String> {
    let uploader = UploaderClient::new("https://node2.irys.xyz")?;
    let uploaded_tx = uploader.upload(build_result).await?;
    let result = match serde_json::to_string(&uploaded_tx) {
        Ok(r) => r,
        Err(e) => return Err(format!("{:?}", e))
    };
    Ok(result)
}

pub async fn write_message(input: Vec<u8>) -> Result<String, String> {
    let mut data_store = match StoreClient::connect() {
        Ok(d) => d,
        Err(e) => return Err(format!("{:?}", e))
    };
    let build_result = build(input).await?;
    let message = Message::from_bundle(&build_result.bundle)?;
    data_store.save_message(&message)?;
    let r = upload(build_result.binary.to_vec()).await?;
    Ok(r)
}

pub async fn write_process(input: Vec<u8>) -> Result<String, String> {
    let mut data_store = match StoreClient::connect() {
        Ok(d) => d,
        Err(e) => return Err(format!("{:?}", e))
    };
    let build_result = build(input).await?;
    let process = Process::from_bundle(&build_result.bundle)?;
    data_store.save_process(&process)?;
    let r = upload(build_result.binary.to_vec()).await?;
    Ok(r)
}

pub async fn read_messages(
    process_id: String, 
    from: Option<String>, 
    to: Option<String>
) -> Result<String, String> {
    let mut data_store = match StoreClient::connect() {
        Ok(d) => d,
        Err(e) => return Err(format!("{:?}", e))
    };
    let messages = data_store.get_messages(&process_id)?;
    let sorted_messages = SortedMessages::from_messages(messages, from, to)?;
    let result = match serde_json::to_string(&sorted_messages) {
        Ok(r) => r,
        Err(e) => return Err(format!("{:?}", e))
    };
    Ok(result)
}

pub async fn read_message(
    message_id: String
) -> Result<String, String> {
    let mut data_store = match StoreClient::connect() {
        Ok(d) => d,
        Err(e) => return Err(format!("{:?}", e))
    };
    let message = data_store.get_message(&message_id)?;
    let result = match serde_json::to_string(&message) {
        Ok(r) => r,
        Err(e) => return Err(format!("{:?}", e))
    };
    Ok(result)
}

pub async fn read_process(
    process_id: String
) -> Result<String, String> {
    let mut data_store = match StoreClient::connect() {
        Ok(d) => d,
        Err(e) => return Err(format!("{:?}", e))
    };
    let process = data_store.get_process(&process_id)?;
    let result = match serde_json::to_string(&process) {
        Ok(r) => r,
        Err(e) => return Err(format!("{:?}", e))
    };
    Ok(result)
}


fn system_time() -> Result<String, SystemTimeError> {
    let start_time = SystemTime::now();
    let duration = start_time.duration_since(UNIX_EPOCH)?;
    let millis = duration.as_secs() * 1000 + u64::from(duration.subsec_millis());
    let millis_string = millis.to_string();
    Ok(millis_string)
}

pub async fn timestamp() -> Result<String, String>{
    match system_time() {
        Ok(timestamp) => {
            let gateway_url = "https://arweave.net".to_string();
            let url = match Url::parse(&gateway_url) {
                Ok(u) => u,
                Err(e) => return Err(format!("url error {:?}", e))
            };

            let network_client = NetworkInfoClient::new(url);
            let network_info = network_client.network_info().await;
            match network_info {
                Ok(info) => {
                    let height = info.height.clone();
                    let height_string = format!("{:0>12}", height);
                    let response_json = json!({ "timestamp": timestamp, "block_height": height_string });
                    Ok(response_json.to_string())
                },
                Err(e) => {
                    Err(format!("{:?}", e))
                }
            }
            
        }
        Err(e) => Err(format!("{:?}", e))
    }
}