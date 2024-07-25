use std::collections::HashMap;
use std::fmt;
use std::fmt::Error;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::Local;
use chrono::Utc;
use clap::{arg, Parser};
use env_logger;
use futures::{future::TryFutureExt, Sink, sink::SinkExt, Stream, stream::StreamExt};
use futures::channel::mpsc::SendError;
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
use yellowstone_grpc_proto::geyser::SubscribeUpdate;
use yellowstone_grpc_proto::prelude::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts,
    SubscribeRequestFilterBlocks, SubscribeRequestFilterBlocksMeta, SubscribeRequestFilterEntry,
    SubscribeRequestFilterSlots, SubscribeRequestFilterTransactions, SubscribeRequestPing,
    SubscribeUpdateAccount, SubscribeUpdateTransaction, SubscribeUpdateTransactionStatus,
};
use yellowstone_grpc_proto::tonic::Status;

use crate::action::{ActionData, ActionType, Metadata, NewPairData};
use crate::cli::Args;
use crate::common::ProgramID::{RaydiumAuthorityV4, RaydiumLiquidityPoolV4};
use crate::pretty::{AccountPretty, TransactionPretty, TransactionStatusPretty, VersionInfo};

mod action;
mod common;
mod pretty;
mod utils;
mod cli;

// mongo时
const MG_DB_NAME_RAYDIUM: &str = "raydium";
const MG_COLL_ACTIONS: &str = "actions";
const MG_COLL_NEW_PAIRS: &str = "newpairs";

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
            let _fi = iv.index;
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
                    let pre = pre_balance.ui_token_amount.ui_amount.unwrap_or(0.0);
                    let post = post_balance.ui_token_amount.ui_amount.unwrap_or(0.0);
                    let change = post - pre;
                    let decimals = post_balance.ui_token_amount.decimals;

                    token_balance_change.push((token, change, decimals, pre, post));
                } else {
                    // 如果 pre_balance 不存在，表示这是一个新的 token，则认为 pre_balance 为 0
                    let token = post_balance.mint.clone();
                    let pre = 0.0;
                    let post = post_balance.ui_token_amount.ui_amount.unwrap_or(0.0);
                    let change = post - pre;
                    let decimals = post_balance.ui_token_amount.decimals;
                    token_balance_change.push((token, change, decimals, pre, post));
                }
            }
        }
    }
    if token_balance_change.len() != 2 {
        return Err(anyhow!("{:?} token_balance_change.len() != 2", signature));
    }

    let a = token_balance_change.get(0).unwrap().clone();
    let b = token_balance_change.get(1).unwrap().clone();

    let a_token = a.0;
    let a_change = a.1;
    let a_decimals = a.2;
    let a_pre = a.3;
    let a_post = a.4;

    let b_token = b.0;
    let b_change = b.1;
    let b_decimals = b.2;
    let b_pre = b.3;
    let b_post = b.4;

    // 忽略无关交易
    if a_change == 0.0 || b_change == 0.0 {
        return Err(anyhow!(
            "{:?} action_data.amount_a == 0.0 || action_data.amount_b == 0.0",
            signature
        ));
    }

    // 填充数据信息
    action_data.token_a = a_token;
    action_data.amount_a = -1.0 * a_change;
    action_data.decimals_a = a_decimals;
    action_data.token_b = b_token;
    action_data.amount_b = -1.0 * b_change;
    action_data.decimals_b = b_decimals;

    // SWAP B=>A
    if a_change > 0.0 && b_change < 0.0 {
        action_data.action_type = ActionType::TokenSwap.value_string();
    }
    // SWAP A=>B
    if a_change < 0.0 && b_change > 0.0 {
        action_data.action_type = ActionType::TokenSwap.value_string();
    }

    // ADD or Create
    if a_change > 0.0 && b_change > 0.0 {
        if a_pre == 0.0 && b_pre == 0.0 {
            action_data.action_type = ActionType::Create.value_string();
        } else {
            // 普通ADD
            action_data.action_type = ActionType::AddLiquidity.value_string();
        }
    }
    // REMOVE
    if a_change < 0.0 && b_change < 0.0 {
        action_data.action_type = ActionType::RemoveLiquidity.value_string();
    }

    if !action_data.check() {
        return Err(anyhow!("{:?} unkonwn action data", signature));
    }
    Ok(action_data)
}

// 插入数据
async fn insert_action_data(client: &Client, action_data: ActionData) -> Result<String> {
    let db = client.database(MG_DB_NAME_RAYDIUM);
    let sig = &action_data.signature.clone();
    let action_type = &action_data.action_type.clone();
    // 如果是创建新池事件，单独插入集合new pairs
    if action_type.eq(ActionType::Create.value()) {
        info!("new pair is created");
        let new_pair = NewPairData::from_action(action_data.clone());
        db.collection::<NewPairData>(MG_COLL_NEW_PAIRS)
            .insert_one(new_pair, None)
            .await
            .context("insert MG_COLL_NEW_PAIRS fail")?;
    }
    db.collection::<ActionData>(MG_COLL_ACTIONS)
        .insert_one(action_data, None)
        .await
        .context("insert MG_COLL_ACTIONS fail")?;
    Ok(sig.to_string())
}

// 连接mongodb数据库
async fn init_mongo_client(endpoint: &str) -> Result<Client> {
    info!("connect mongodb by url: {:?}", endpoint);
    let client_options = ClientOptions::parse_async(endpoint).await?;
    let mg = Client::with_options(client_options)?;
    let database = mg.database(MG_DB_NAME_RAYDIUM);
    let collection_names = database.list_collection_names(None).await?;
    // 给actions集合创建时间序列
    if !collection_names.contains(&MG_COLL_ACTIONS.to_string()) {
        // 创建时间序列集合
        let mut options = CreateCollectionOptions::default();
        let timeseries_options = TimeseriesOptions::builder()
            .time_field("date_time".to_string())
            .meta_field(Some("metadata".to_string()))
            .granularity(Some(TimeseriesGranularity::Seconds))
            .build();
        options.timeseries = Some(timeseries_options);
        database.create_collection(MG_COLL_ACTIONS, options).await?;
    }
    // 给newpairs集合创建索引
    let collection = database.collection::<NewPairData>(MG_COLL_NEW_PAIRS);

    let date_time_index = IndexModel::builder()
        .keys(doc! { "date_time": -1 })
        .options(
            IndexOptions::builder()
                .name(Some("date_time_index".to_string()))
                .build(),
        )
        .build();

    let signature_index = IndexModel::builder()
        .keys(doc! { "signature": 1 })
        .options(
            IndexOptions::builder()
                .unique(true)
                .name(Some("signature_index".to_string()))
                .build(),
        )
        .build();

    let pair_index = IndexModel::builder()
        .keys(doc! { "pair": 1 })
        .options(
            IndexOptions::builder()
                .unique(true)
                .name(Some("pair_index".to_string()))
                .build(),
        )
        .build();

    collection
        .create_indexes(vec![date_time_index, signature_index, pair_index], None)
        .await?;

    Ok(mg)
}

// 创建grpc 订阅请求参数
fn create_request() -> SubscribeRequest {
    info!("构建grpc请求参数");
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

    let subscribe_request = SubscribeRequest {
        transactions: txmap,
        commitment: Some(CommitmentLevel::Confirmed as i32),
        ..Default::default()
    };
    subscribe_request
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    info!("Application Start ... ... !");

    let args = Args::parse();
    info!("Mongo URL: {}", args.mg_url);
    info!("gRPC URL: {}", args.grpc_url);
    info!("gRPC X-Token: {}", args.grpc_x_token);
    info!("Retry Interval: {:?}", args.retry_interval);
    info!("FeiShu hook url: {:?}", args.feishu);

    // 初始化mongodb 客户端
    let mg_connect_str = args.mg_url.as_str();
    let mg = init_mongo_client(mg_connect_str).await?;
    let interval = args.retry_interval;
    let feishu = args.feishu;

    // 初始化solana geyser grpc 客户端
    let endpoint = args.grpc_url;
    let x_token = args.grpc_x_token;
    info!("solana endpoint: {:?}", endpoint);
    info!("solana x-token: {:?}", x_token);
    // 连接solana 节点 by YellowSton Grpc
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

    // 获取节点信息
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

    loop {
        // 构建订阅请求
        let request = create_request();
        // 消息流
        let (mut sub, mut stream) = client.subscribe_with_request(Some(request)).await?;
        // 死循环接收消息并处理
        while let Some(message) = stream.next().await {
            if message.is_err() {
                break;
            }
            let msg = message.unwrap();
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
                    // todo 处理发送ping 异常的问题
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
        let nitify_msg = "dq_raydium:yellowston-grpc:strem error => 自动重启!";
        utils::send_message(feishu.clone(), nitify_msg).await?;
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}
