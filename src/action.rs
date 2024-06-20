/// @Name ActionType
///
/// @Date 2024/5/12 上午2:44
///
/// @Author Matrix.Ye
///
/// @Description:
use mongodb::bson::doc;
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
            ActionType::TokenSwap => "swap",
            ActionType::AggTokenSwap => "agg_swap",
            ActionType::AddLiquidity => "add",
            ActionType::RemoveLiquidity => "remove",
            ActionType::Stake => "stake",
            ActionType::Unstake => "unstake",
            ActionType::WithdrawStake => "withdraw_stake",
            _ => "unknown",
        }
    }
    pub fn value_string(&self) -> String {
        self.value().to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ActionData {
    // 交易签名
    pub signature: String,
    // 区块时间戳
    pub block_time: i64,
    // 时间槽
    pub slot: u64,
    // 交易类型，TokenSwap，AddLiquidity，RemoveLiquidity
    pub action_type: String,
    // 签名者即交易者
    pub signer: String,
    // 池子账户地址
    pub pair: String,
    // 代币0账户地址
    pub token_a: String,
    // 代币a的数量
    pub amount_a: f64,
    // 代币a的精度
    pub decimals_a: u8,
    // 代币b账户地址
    pub token_b: String,
    // 代币b的数量
    pub amount_b: f64,
    // 代币b精度
    pub decimals_b: u8,
    // gas消耗
    pub fee: u64,
}

impl ActionData {
    pub fn default() -> ActionData {
        let data = ActionData {
            action_type: "Unknown".to_string(),
            ..Default::default()
        };
        data
    }
    // 检查
    pub fn check(&self) -> bool {
        !self.token_b.is_empty() && !self.token_a.is_empty() && self.action_type != "Unknown"
    }
}

pub struct Sign {}