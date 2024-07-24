use anyhow::Result;
use clap::builder::Str;
use reqwest::Client;
use serde_json::json;

pub(crate) async fn send_message(url: Option<String>, msg: &str) -> Result<()> {
    match url {
        None => {
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
