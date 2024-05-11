/// @Name ActionType
///
/// @Date 2024/5/12 上午2:44
///
/// @Author Matrix.Ye
///
/// @Description:

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

pub fn insert_action() {}

pub fn delete_action() {}

pub fn update_action() {}

pub fn query_action() {}