use std::collections::HashMap;
use std::time::Duration;

use {yellowstone_grpc_client::GeyserGrpcClient,
     yellowstone_grpc_proto::prelude::{
         CommitmentLevel,
         SubscribeRequestFilterAccounts,
         SubscribeRequestFilterBlocks,
         SubscribeRequestFilterBlocksMeta,
         SubscribeRequestFilterEntry,
         SubscribeRequestFilterSlots,
         SubscribeRequestFilterTransactions,
     }};

type SlotsFilterMap = HashMap<String, SubscribeRequestFilterSlots>;
type AccountFilterMap = HashMap<String, SubscribeRequestFilterAccounts>;
type TransactionsFilterMap = HashMap<String, SubscribeRequestFilterTransactions>;
type TransactionsStatusFilterMap = HashMap<String, SubscribeRequestFilterTransactions>;
type EntryFilterMap = HashMap<String, SubscribeRequestFilterEntry>;
type BlocksFilterMap = HashMap<String, SubscribeRequestFilterBlocks>;
type BlocksMetaFilterMap = HashMap<String, SubscribeRequestFilterBlocksMeta>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = "https://redoubled-skippet-wbhxrrgkog-dedicated-lb.helius-rpc.com:2053";
    let token = "34df4c72-5d54-49ba-ae2c-2aa1d50679ce";
    let mut client = GeyserGrpcClient::build_from_shared(endpoint)?
        .x_token(Some(token))?
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .connect()
        .await?;

    println!("{:?}", "end connect");
    let version_response = client.get_version().await?;
    println!("Version:{:?}", version_response.version);

    let slot_response = client.get_slot(Some(CommitmentLevel::Finalized)).await?;
    println!("Slot:{:?}", slot_response.slot);

    let block_height_response = client.get_block_height(Some(CommitmentLevel::Finalized)).await?;
    println!("Block_height:{:?}", block_height_response.block_height);

    let latest_blockhash_response = client.get_latest_blockhash(Some(CommitmentLevel::Finalized)).await?;
    println!("latest_blockhash_response:{:?}", latest_blockhash_response);

    let check_response = client.health_check().await?;
    println!("Check:{:?}", check_response.status());

    // // 订阅账户地址的交易数据
    // let mut accounts: AccountFilterMap = HashMap::new();
    //
    // let mut stream = client.geyser.subscribe(())
    //     .await?
    //     .into_inner();
    //
    // println!("{:?}", "start subscribing");
    //
    // while let Some(message) = stream.message().await? {
    //     println!("Received transaction: {:?}", message);
    // }

    Ok(())
}
