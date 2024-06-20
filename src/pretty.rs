use std::fmt;

use serde::Deserialize;
use solana_sdk::bs58;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::TransactionError;
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionEncoding};
use yellowstone_grpc_proto::geyser::{SubscribeUpdateAccount, SubscribeUpdateTransaction, SubscribeUpdateTransactionStatus};

/// @Name pretty
///
/// @Date 2024/6/14 下午4:22
///
/// @Author Matrix.Ye
///
/// @Description:
#[derive(Debug)]
#[allow(dead_code)]
pub struct AccountPretty {
    pub is_startup: bool,
    pub slot: u64,
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: String,
    pub write_version: u64,
    pub txn_signature: String,
}

impl From<SubscribeUpdateAccount> for AccountPretty {
    fn from(
        SubscribeUpdateAccount {
            is_startup,
            slot,
            account,
        }: SubscribeUpdateAccount,
    ) -> Self {
        let account = account.expect("should be defined");
        Self {
            is_startup,
            slot,
            pubkey: Pubkey::try_from(account.pubkey).expect("valid pubkey"),
            lamports: account.lamports,
            owner: Pubkey::try_from(account.owner).expect("valid pubkey"),
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            data: hex::encode(account.data),
            write_version: account.write_version,
            txn_signature: bs58::encode(account.txn_signature.unwrap_or_default()).into_string(),
        }
    }
}

#[allow(dead_code)]
pub struct TransactionPretty {
    pub slot: u64,
    pub signature: Signature,
    pub is_vote: bool,
    pub tx: EncodedTransactionWithStatusMeta,
}

impl fmt::Debug for TransactionPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct TxWrap<'a>(&'a EncodedTransactionWithStatusMeta);
        impl<'a> fmt::Debug for TxWrap<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let serialized = serde_json::to_string(self.0).expect("failed to serialize");
                fmt::Display::fmt(&serialized, f)
            }
        }

        f.debug_struct("TransactionPretty")
            .field("slot", &self.slot)
            .field("signature", &self.signature)
            .field("is_vote", &self.is_vote)
            .field("tx", &TxWrap(&self.tx))
            .finish()
    }
}

impl From<SubscribeUpdateTransaction> for TransactionPretty {
    fn from(SubscribeUpdateTransaction { transaction, slot }: SubscribeUpdateTransaction) -> Self {
        let tx = transaction.expect("should be defined");
        Self {
            slot,
            signature: Signature::try_from(tx.signature.as_slice()).expect("valid signature"),
            is_vote: tx.is_vote,
            tx: yellowstone_grpc_proto::convert_from::create_tx_with_meta(tx)
                .expect("valid tx with meta")
                .encode(UiTransactionEncoding::JsonParsed, Some(u8::MAX), true)
                .expect("failed to encode"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TransactionStatusPretty {
    pub slot: u64,
    pub signature: Signature,
    pub is_vote: bool,
    pub index: u64,
    pub err: Option<TransactionError>,
}

impl From<SubscribeUpdateTransactionStatus> for TransactionStatusPretty {
    fn from(status: SubscribeUpdateTransactionStatus) -> Self {
        Self {
            slot: status.slot,
            signature: Signature::try_from(status.signature.as_slice()).expect("valid signature"),
            is_vote: status.is_vote,
            index: status.index,
            err: yellowstone_grpc_proto::convert_from::create_tx_error(status.err.as_ref())
                .expect("valid tx err"),
        }
    }
}


///
/// ## Solana version info
/// ```json
/// "{\"version\":{\"package\":\"yellowstone-grpc-geyser\",\"version\":\"1.14.1+solana.1.17.31\",\"proto\":\"1.14.0+solana.1.17.31\",\"solana\":\"1.17.31\",\"git\":\"46ecf8c\",\"rustc\":\"1.73.0\",\"buildts\":\"2024-05-30T03:39:31.649324344Z\"},\"extra\":{\"hostname\":\"m3-dimsum-ewr-9\"}}"
/// ```
#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub version: Version,
    pub extra: Extra,
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub package: String,
    pub version: String,
    pub proto: String,
    pub solana: String,
    pub git: String,
    pub rustc: String,
    pub buildts: String,
}

#[derive(Debug, Deserialize)]
pub struct Extra {
    pub hostname: String,
}