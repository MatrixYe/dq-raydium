/// @Name ActionType
///
/// @Date 2024/5/12 上午2:44
///
/// @Author Matrix.Ye
///
/// @Description:
use mongodb::{bson::doc, sync::Client};
use serde::{Deserialize, Serialize};

pub enum ActionType {
    // 掉期
    TokenSwap,
    // 切分掉期
    AggTokenSwap,
    // 添加流动性
    AddLiquidity,
    // 移除流动性
    RemoveLiquidity,
    //
    Stake,
    //
    Unstake,
    //
    WithdrawStake,
}

impl ActionType {
    pub fn value(&self) -> &'static str {
        match self {
            ActionType::TokenSwap => "TokenSwap",
            ActionType::AggTokenSwap => "AggTokenSwap",
            ActionType::AddLiquidity => "AddLiquidity",
            ActionType::RemoveLiquidity => "RemoveLiquidity",
            ActionType::Stake => "Stake",
            ActionType::Unstake => "Unstake",
            ActionType::WithdrawStake => "WithdrawStake",
            _ => "Unknown",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionData {
    pub block_time: i64,
    pub slot: u64,
    pub signer: String,
    pub pair: String,
    pub token0: String,
    pub token1: String,
    pub amount0: f64,
    pub amount1: f64,
    pub fee: u64,
    pub signature: String,
}

impl ActionData {
    pub fn default() -> ActionData {
        let data = ActionData {
            signature: "".to_string(),
            block_time: 0,
            slot: 0,
            signer: "".to_string(),
            pair: "".to_string(),
            token0: "".to_string(),
            token1: "".to_string(),
            amount0: 0.0,
            amount1: 0.0,
            fee: 0,
        };
        data
    }
}

pub fn insert_action(client: Client, action: ActionData) -> mongodb::error::Result<()> {
    // let uri = "<connection string>";
    // let client = Client::with_uri_str(uri)?;


    let res = client
        .database("raydium")
        .collection("actions").insert_one(action, None)?;
    println!("Inserted a document with _id: {}", res.inserted_id);
    Ok(())
}

pub fn delete_action() {}

pub fn update_action() {}

pub fn query_action() {}