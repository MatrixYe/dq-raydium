use std::collections::HashMap;
use std::fmt;
use std::fmt::Error;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::Local;
use chrono::Utc;
use env_logger;
use futures::{future::TryFutureExt, sink::SinkExt, stream::StreamExt};
use log::{error, info, warn};
use mongodb::{Client, IndexModel};
use mongodb::bson::{DateTime, doc, Document};
use mongodb::options::{
    ClientOptions, CreateCollectionOptions, IndexOptions, ServerApi, ServerApiVersion,
    TimeseriesGranularity, TimeseriesOptions,
};
use serde_json;
use solana_sdk::{bs58, pubkey::Pubkey, signature::Signature, transaction::TransactionError};
use solana_transaction_status::{
    EncodedTransaction, EncodedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction,
    UiMessage, UiParsedInstruction, UiParsedMessage, UiTransactionEncoding,
    UiTransactionStatusMeta,
};
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::parse_accounts::ParsedAccount;
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;
use yellowstone_grpc_proto::prelude::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts,
    SubscribeRequestFilterBlocks, SubscribeRequestFilterBlocksMeta, SubscribeRequestFilterEntry,
    SubscribeRequestFilterSlots, SubscribeRequestFilterTransactions, SubscribeRequestPing,
    SubscribeUpdateAccount, SubscribeUpdateTransaction, SubscribeUpdateTransactionStatus,
};

use crate::action::{ActionData, ActionType, Metadata};
use crate::common::ProgramID::{RaydiumAuthorityV4, RaydiumLiquidityPoolV4};
use crate::pretty::{AccountPretty, TransactionPretty, TransactionStatusPretty, VersionInfo};

mod action;
mod common;
mod pretty;

const MG_DB_NAME_RAYDIUM: &str = "raydium";
const MG_COLL_NAME_ACTIONS: &str = "actions";

type SlotsFilterMap = HashMap<String, SubscribeRequestFilterSlots>;
type AccountFilterMap = HashMap<String, SubscribeRequestFilterAccounts>;
type TransactionsFilterMap = HashMap<String, SubscribeRequestFilterTransactions>;
type TransactionsStatusFilterMap = HashMap<String, SubscribeRequestFilterTransactions>;
type EntryFilterMap = HashMap<String, SubscribeRequestFilterEntry>;
type BlocksFilterMap = HashMap<String, SubscribeRequestFilterBlocks>;
type BlocksMetaFilterMap = HashMap<String, SubscribeRequestFilterBlocksMeta>;

//noinspection ALL,RsMainFunctionNotFound
async fn handle_transaction_update(transaction: TransactionPretty) -> Result<ActionData> {
    let mut action_data = ActionData::default();

    let data = transaction.tx;
    let signature = transaction.signature.to_string();
    // info!("signature: {}",signature);
    // 交易签名
    action_data.signature = signature.clone();
    // 时间槽
    action_data.slot = transaction.slot;
    let now_ts = Utc::now().timestamp_millis();
    action_data.timestamp = now_ts;
    action_data.date_time = Some(DateTime::from_millis(now_ts));

    if data.meta.is_none() {
        return Err(anyhow!("data.meta.is_none"));
    }

    let mut accounts: Vec<ParsedAccount> = vec![];
    if let EncodedTransaction::Json(tx) = data.transaction {
        if let UiMessage::Parsed(msg) = tx.message {
            accounts = msg.account_keys;
            //     解析外部指令
            for (i, instruction) in msg.instructions.iter().enumerate() {
                match instruction {
                    UiInstruction::Compiled(_) => {
                        // info!("外部指令#{:?} UiInstruction::Compiled",i+1);
                        // info!("{:?}",x);
                    }
                    UiInstruction::Parsed(ui_ins_parsed) => {
                        match ui_ins_parsed {
                            UiParsedInstruction::Parsed(_) => {}
                            UiParsedInstruction::PartiallyDecoded(x) => {
                                // info!("外部指令#{:?} UiParsedInstruction::PartiallyDecoded",i+1);
                                // info!("{:?}",x);
                                if x.program_id == RaydiumLiquidityPoolV4.value_str() {
                                    let ammid = &x.accounts[1];
                                    // info!("amm: {:?}",ammid);
                                    action_data.metadata.pair = ammid.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // 解析签名者
    action_data.metadata.signer = accounts.first().unwrap().pubkey.to_string();
    let meta = data.meta.unwrap();
    action_data.fee = meta.fee;

    // 解析内部交易
    if let OptionSerializer::Some(ui_inner_ins) = meta.inner_instructions {
        for iv in ui_inner_ins {
            let fi = iv.index;
            for (i, instruction) in iv.instructions.iter().enumerate() {
                match instruction {
                    UiInstruction::Compiled(_) => {}
                    UiInstruction::Parsed(x) => {
                        match x {
                            UiParsedInstruction::Parsed(_) => {}
                            UiParsedInstruction::PartiallyDecoded(v) => {
                                // info!("内部指令#{:?}.{:?} UiParsedInstruction::PartiallyDecoded",fi+1,i+1);
                                // info!("{:?}",v);
                                if v.program_id == RaydiumLiquidityPoolV4.value_str() {
                                    let ammid = &v.accounts[1];
                                    // info!("amm: {:?}",ammid);
                                    action_data.metadata.pair = ammid.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let pre_balances = match meta.pre_token_balances {
        OptionSerializer::Some(balances) => balances,
        _ => vec![],
    };

    let post_balances = match meta.post_token_balances {
        OptionSerializer::Some(balances) => balances,
        _ => vec![],
    };

    let mut token_balance_change = Vec::new();

    for post_balance in post_balances {
        if let OptionSerializer::Some(owner_value) = post_balance.owner.clone() {
            if owner_value == RaydiumAuthorityV4.value_str() {
                if let Some(pre_balance) = pre_balances
                    .iter()
                    .find(|pre| pre.account_index == post_balance.account_index)
                {
                    let token = post_balance.mint.clone();
                    let change = post_balance.ui_token_amount.ui_amount.unwrap_or(0.0)
                        - pre_balance.ui_token_amount.ui_amount.unwrap();
                    let decimals = post_balance.ui_token_amount.decimals;

                    token_balance_change.push((token, change, decimals));
                } else {
                    // 如果 pre_balance 不存在，表示这是一个新的 token，则认为 pre_balance 为 0
                    let token = post_balance.mint.clone();
                    let change = post_balance.ui_token_amount.ui_amount.unwrap_or(0.0);
                    let decimals = post_balance.ui_token_amount.decimals;
                    token_balance_change.push((token, change, decimals));
                }
            }
        }
    }
    if token_balance_change.len() != 2 {
        return Err(anyhow!("{:?} token_balance_change.len() != 2", signature));
    }

    let c0 = token_balance_change.get(0).unwrap().clone();
    let c1 = token_balance_change.get(1).unwrap().clone();
    action_data.token_a = c0.0;
    action_data.amount_a = c0.1 * -1.0;
    action_data.decimals_a = c0.2;

    action_data.token_b = c1.0;
    action_data.amount_b = c1.1 * -1.0;
    action_data.decimals_b = c1.2;

    if action_data.amount_a == 0.0 || action_data.amount_b == 0.0 {
        return Err(anyhow!(
            "{:?} action_data.amount_a == 0.0 || action_data.amount_b == 0.0",
            signature
        ));
    }

    if action_data.amount_a > 0.0 && action_data.amount_b < 0.0
        || action_data.amount_a < 0.0 && action_data.amount_b > 0.0
    {
        action_data.action_type = ActionType::TokenSwap.value_string();
    }
    if action_data.amount_a > 0.0 && action_data.amount_b > 0.0 {
        action_data.action_type = ActionType::RemoveLiquidity.value_string();
    }
    if action_data.amount_a < 0.0 && action_data.amount_b < 0.0 {
        action_data.action_type = ActionType::AddLiquidity.value_string();
    }

    if !action_data.check() {
        return Err(anyhow!("{:?} unkonwn action data", signature));
    }
    Ok(action_data)
}

// 插入数据
async fn insert_action_data(client: &Client, action_data: ActionData) -> Result<String> {
    let db = client.database(MG_DB_NAME_RAYDIUM);
    let collection = db.collection::<ActionData>(MG_COLL_NAME_ACTIONS);
    let s = &action_data.signature.clone();

    let result = collection.insert_one(action_data, None).await;
    match result {
        Ok(_) => Ok(s.to_string()),
        Err(e) => Err(anyhow!(e.to_string())),
    }
}

// 连接mongodb数据库
async fn init_mongo_client(endpoint: &str) -> Result<Client> {
    info!("connect mongodb by url: {:?}", endpoint);
    let client_options = ClientOptions::parse_async(endpoint).await?;
    let mg = Client::with_options(client_options)?;
    let database = mg.database(MG_DB_NAME_RAYDIUM);
    let collection_names = database.list_collection_names(None).await?;
    if !collection_names.contains(&MG_COLL_NAME_ACTIONS.to_string()) {
        // 创建时间序列集合
        let mut options = CreateCollectionOptions::default();
        let timeseries_options = TimeseriesOptions::builder()
            .time_field("date_time".to_string())
            .meta_field(Some("metadata".to_string()))
            .granularity(Some(TimeseriesGranularity::Seconds))
            .build();
        options.timeseries = Some(timeseries_options);
        database
            .create_collection(MG_COLL_NAME_ACTIONS, options)
            .await?;
    }
    Ok(mg)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    info!("this is application start ... ... !");

    // 初始化mongodb 客户端
    let mg_connect_str = "mongodb://root:nopassword@127.0.0.1:5011/";
    let mg = init_mongo_client(mg_connect_str).await?;

    // 初始化solana geysergrpc 客户端
    let endpoint = "https://redoubled-skippet-wbhxrrgkog-dedicated-lb.helius-rpc.com:2053";
    let x_token = "34df4c72-5d54-49ba-ae2c-2aa1d50679ce";
    info!("solana endpoint: {:?}", endpoint);
    info!("solana x-token: {:?}", x_token);

    let mut client = GeyserGrpcClient::build_from_shared(endpoint)
        .expect("connect solana fail,error endpoint")
        .x_token(Some(x_token))
        .expect("connect solana fail,error x-token")
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .expect("connect solana fail,server error");
    info!("connect solana validator success.");
    // 获取节点版本号
    let version_response = client.get_version().await?;
    let version_info: VersionInfo =
        serde_json::from_str(version_response.version.as_str()).unwrap();
    let check_response = client.health_check().await?;

    info!("version:{:?}", version_info.version.version);
    info!("solana:{:?}", version_info.version.solana);
    info!("rustc:{:?}", version_info.version.rustc);
    info!("proto:{:?}", version_info.version.proto);
    info!("hostname:{:?}", version_info.extra.hostname);
    info!("status: {:?}", check_response.status());
    // 订阅指定账户下的实时交易
    let mut txmap: TransactionsFilterMap = HashMap::new();
    // 构建交易过滤器，指定目标账户、屏蔽投票交易、屏蔽失败交易
    let filter = SubscribeRequestFilterTransactions {
        vote: Some(false),
        failed: Some(false),
        signature: None,
        account_include: vec![RaydiumLiquidityPoolV4.value_str()],
        account_exclude: vec![],
        account_required: vec![],
    };
    txmap.insert("raydium".to_owned(), filter);

    let subscriberequest = SubscribeRequest {
        transactions: txmap,
        commitment: Some(CommitmentLevel::Confirmed as i32),
        ..Default::default()
    };

    let (mut sub, mut stream) = client
        .subscribe_with_request(Some(subscriberequest))
        .await?;
    // 接收消息并处理
    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                match msg.update_oneof {
                    Some(UpdateOneof::Transaction(tx)) => {
                        let transaction: TransactionPretty = tx.into();
                        info!("new transaction update: filters {:?}", msg.filters);
                        match handle_transaction_update(transaction).await {
                            Ok(action) => {
                                // info!("action parse success:\n{:?}", action);
                                match insert_action_data(&mg, action).await {
                                    Ok(s) => {
                                        info!("Success: {:?}", s);
                                    }
                                    Err(e) => {
                                        error!("Error inserting: {:?}", e.to_string());
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("parse fail{:?}", e.to_string())
                            }
                        }
                        continue;
                    }
                    Some(UpdateOneof::Ping(_)) => {
                        sub.send(SubscribeRequest {
                            ping: Some(SubscribeRequestPing { id: 1 }),
                            ..Default::default()
                        })
                            .await?;
                        continue;
                    }
                    _ => {
                        continue;
                    }
                }
            }
            Err(error) => {
                error!("error: {error:?}");
            }
        }
    }
    Ok(())
}
