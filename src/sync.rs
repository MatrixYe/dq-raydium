/// @Name sync
///
/// @Date 2024/5/8 下午7:55
///
/// @Author Matrix.Ye
///
/// @Description:
use std::error::Error;
use std::str::FromStr;
use std::time::Instant;

use solana_client::client_error::Result as Result2;
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiMessage, UiTransactionEncoding};

use crate::action::ActionData;
use crate::common::ProgramID::{RaydiumAuthorityV4, RaydiumLiquidityPoolV4};

const RPC_URL: &str = "http://gastroscopic-agars-gGx9fjhXm7.helius-rpc.com";
// const RPC_URL: &str = "https://devnet.helius-rpc.com/?api-key=2c7e26a1-607c-46e4-9907-9e6ea71dff6a";

fn get_signatures(client: &RpcClient, pb: &Pubkey) {}


// use solana_client::
fn core() -> Result<(), Box<dyn Error>> {
    // 初始化 RPC 客户端
    let client = RpcClient::new(RPC_URL);

    // Raydium 合约的公钥地址
    let raydium_pubkey = RaydiumAuthorityV4.pubkey();
    let last_time = 1715172732;

    let mut before: Option<Signature> = None;
    loop {
        let config = GetConfirmedSignaturesForAddress2Config {
            before,
            until: None,
            limit: Some(1000),
            commitment: None,
        };
        let signatures = client.get_signatures_for_address_with_config(&raydium_pubkey, config)?;
        // 遍历签名，获取每个交易的详细信息
        for signature in &signatures {
            let signature_str = &signature.signature;
            let block_time = signature.block_time.unwrap();
            println!(" block_time: {:?} signature: {:?}", block_time, signature_str);
            if block_time < last_time {
                println!("{:?}", "同步历史完成");
                return Ok(());
            }
            // config.before = Some(Signature::from_str(signature_str).unwrap());
            before = Some(Signature::from_str(signature_str).unwrap());
        }
        // last_signature = Some(signatures.last().unwrap().signature.to_string());
        // sleep(Duration::new(1, 0))
    }
}

/// 获取交易信息by签名
fn get_trancation(client: &RpcClient, sign: &str) -> Result2<EncodedConfirmedTransactionWithStatusMeta> {
    let signature = Signature::from_str(sign).expect("error sign");
    let mut config = RpcTransactionConfig::default();
    config.max_supported_transaction_version = Some(0);
    config.commitment = Some(CommitmentConfig::confirmed());
    config.encoding = Some(UiTransactionEncoding::JsonParsed);

    let transaction = client.get_transaction_with_config(
        &signature,
        config,
    );
    return transaction;
}


/// 解析交易
fn parse_tx(encode_tx: EncodedConfirmedTransactionWithStatusMeta) -> Option<()> {
    println!("{:?}", encode_tx);
    let mut data = ActionData::default();
    // 区块时间
    let block_time = encode_tx.block_time?;
    data.block_time = block_time as i64;
    // 区块slot
    let slot = encode_tx.slot;
    data.slot = slot as u64;
    // meta数据
    if let Some(meta) = encode_tx.transaction.meta {
        // gas费
        let fee = meta.fee;
        data.fee = fee;
    }
    if let EncodedTransaction::Json(tx) = encode_tx.transaction.transaction {
        // 解析签名
        let signature = tx.signatures.get(0)?;
        data.signature = signature.clone();
        if let UiMessage::Parsed(msg) = tx.message {
            let accounts = msg.account_keys;
            let singer = accounts.get(0)?;
            data.signer = singer.pubkey.to_string();
        }
    }

    Some(())
}


fn debug() {
    let st = Instant::now();
    let signature = "34XdvmWNyS3LTZC4UEDGAAsdpnpAeB6ipa2ahHPFy3Qti3vNvR5w6ED7kYzbZh3WZqioLq3w2CKgLBiFD4icbT7d";
    let client = RpcClient::new(RPC_URL);

    let raydium_pubkey = RaydiumLiquidityPoolV4.pubkey();
    println!("{:?}", "开始解析第一个");
    let tx_result = get_trancation(&client, signature);
    // let tx_result = get_trancation(&client, "4RNYDXphf3S44bEKjpginAx7am3SatineSStEpFZBh8imAEQN4hMEhVi7NDTQYDhWaSU1VZJ4uWB6MQsiWrfk87y");
    match tx_result {
        Ok(v) => {


// 创建文件名，基于签名的前6位字符
//             let filename = format!("{}.json", &signature[0..6]);
//             let mut file = File::create(filename).unwrap();
//             writeln!(file, "{}", s)?;
//             println!("Transaction data has been written to the file.");
            // parse_tx(v);
        }
        Err(err) => {
            println!("ERRORx:{:?}", err);
        }
    }
    let d = st.elapsed();
    println!("elapsed:{:?}", d.as_millis());
}

pub fn start() -> Result<(), Box<dyn Error>> {
    debug();
    Ok(())
}