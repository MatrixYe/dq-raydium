/// @Name ProgramID
///
/// @Date 2024/5/10 下午2:33
///
/// @Author Matrix.Ye
///
/// @Description:

const RAYDIUM_AUTHORITY_V4: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";
const RAYDIUM_LIQUIDITY_POOL_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const JUPITER_AGGREGATOR_V6: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
const WSOL: &str = "So11111111111111111111111111111111111111112";

pub enum Action {
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

impl Action {
    fn value(&self) -> &'static str {
        match self {
            Action::TokenSwap => "TokenSwap",
            Action::AggTokenSwap => "AggTokenSwap",
            Action::AddLiquidity => "AddLiquidity",
            Action::RemoveLiquidity => "RemoveLiquidity",
            Action::Stake => "Stake",
            Action::Unstake => "Unstake",
            Action::WithdrawStake => "WithdrawStake",
            _ => "Unknown",
        }
    }
}
