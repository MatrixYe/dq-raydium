use anyhow::Result;
use log::{error, info, warn};
use reqwest::Client;
use serde_json::json;

pub(crate) async fn send_message(url: Option<String>, msg: &str) -> Result<()> {
    match url {
        None => {
            warn!("feishu url is empty,send cancle");
            Ok(())
        }
        Some(u) => {
            let client = Client::new();
            let text = json!({"msg_type": "text","content": {"text": msg}});
            let res = client.post(u.as_str()).json(&text).send().await?;
            Ok(())
        }
    }
}
