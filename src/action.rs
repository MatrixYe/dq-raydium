use mongodb::bson::{DateTime, doc};
use serde::{Deserialize, Serialize};

pub enum ActionType {
    // 创建新交易对
    #[allow(unused_variables)]
    Create,
    // 掉期
    TokenSwap,
    // 切分掉期
    #[allow(unused_variables)]
    AggTokenSwap,
    // 添加流动性
    AddLiquidity,
    // 移除流动性
    RemoveLiquidity,
    //
    #[allow(unused_variables)]
    Stake,
    //
    #[allow(unused_variables)]
    Unstake,
    //
    #[allow(unused_variables)]
    WithdrawStake,
    //
    #[allow(unused_variables)]
    Unknown,
}

impl ActionType {
    pub fn value(&self) -> &'static str {
        match self {
            ActionType::Create => "create",
            ActionType::TokenSwap => "swap",
            ActionType::AggTokenSwap => "agg_swap",
            ActionType::AddLiquidity => "add",
            ActionType::RemoveLiquidity => "remove",
            ActionType::Stake => "stake",
            ActionType::Unstake => "unstake",
            ActionType::WithdrawStake => "withdraw_stake",
            ActionType::Unknown => "unknown",
        }
    }
    pub fn value_string(&self) -> String {
        self.value().to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionData {
    // 时间序列字段
    pub date_time: Option<DateTime>,
    // 元数据
    pub metadata: Metadata,
    // 交易时间戳
    pub timestamp: i64,
    // 交易签名
    pub signature: String,
    // 时间槽
    pub slot: u64,
    // 交易类型，TokenSwap，AddLiquidity，RemoveLiquidity
    pub action_type: String,
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

// 元数据字段
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    // 签名者
    pub signer: String,
    // pair地址，即ammid
    pub pair: String,
}

impl ActionData {
    pub fn default() -> ActionData {
        let data = ActionData {
            date_time: None,
            timestamp: 0,
            signature: "".to_string(),
            slot: 0,
            action_type: "".to_string(),
            token_a: "".to_string(),
            amount_a: 0.0,
            decimals_a: 0,
            token_b: "".to_string(),
            amount_b: 0.0,
            decimals_b: 0,
            fee: 0,
            metadata: Metadata {
                signer: "".to_string(),
                pair: "".to_string(),
            },
        };
        data
    }
    // 检查
    pub fn check(&self) -> bool {
        !self.token_b.is_empty()
            && !self.token_a.is_empty()
            && !self.metadata.pair.is_empty()
            && !self.metadata.signer.is_empty()
            && !self.action_type.is_empty()
    }
}
