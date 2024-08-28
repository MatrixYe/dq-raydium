use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;

/// @Name ProgramID
///
/// @Date 2024/5/10 下午2:33
///
/// @Author Matrix.Ye
///
/// @Description:

/// 相关的固定字段
pub enum ProgramID {
    #[allow(unused_variables)]
    RaydiumLiquidityPoolV4, // Raydium V4 核心合约地址
    #[allow(unused_variables)]
    RaydiumAuthorityV4,
    #[allow(unused_variables)]
    JupiterAggregatorV6, // Jupiter的地址
}

impl ProgramID {
    pub fn value(&self) -> &'static str {
        match self {
            ProgramID::RaydiumLiquidityPoolV4 => "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
            ProgramID::RaydiumAuthorityV4 => "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
            ProgramID::JupiterAggregatorV6 => "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
        }
    }
    pub fn value_str(&self) -> String {
        let v = String::from_str(self.value()).unwrap();
        v
    }
    pub fn pubkey(&self) -> Pubkey {
        return Pubkey::from_str(self.value()).unwrap();
    }
}

/// 相关Token的mint地址
#[allow(unused_variables)]
pub enum TokenID {
    WSOL,
}

impl TokenID {
    pub fn value(&self) -> &'static str {
        match self {
            TokenID::WSOL => "So11111111111111111111111111111111111111112",
        }
    }
    pub fn pubkey(&self) -> Pubkey {
        return Pubkey::from_str(self.value()).unwrap();
    }
}
