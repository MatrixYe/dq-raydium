use anyhow::Result;
use reqwest::Client;
use serde_json::json;

pub(crate) async fn send_message(url: &str, msg: &str) -> Result<()> {
    let client = Client::new();
    let text = json!({
        "msg_type": "text",
        "content": {
            "text": msg
        }
    });
    let res = client.post(url).json(&text).send().await?;

    Ok(())
}
