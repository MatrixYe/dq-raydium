/// @Name cli
///
/// @Date 2024/7/24 下午5:26
///
/// @Author Matrix.Ye
///
/// @Description:
use clap::Parser;

/// 命令行参数结构体
#[derive(Parser, Debug)]
#[command(name = "Crate:Sync", about = "An example CLI for argument parsing")]
pub struct Args {
    /// Mongo数据库连接URL
    #[arg(long)]
    pub mg_url: String,

    /// gRPC连接URL
    #[arg(long)]
    pub grpc_url: String,

    /// gRPC连接的Token
    #[arg(long)]
    pub grpc_x_token: String,

    /// 重试时间间隔（可选），默认值为15
    #[arg(long, default_value_t = 15)]
    pub retry_interval: u64,

    // /// 飞书消息推送url
    #[arg(long)]
    pub feishu: Option<String>,

    #[arg(long)]
    pub restart: bool,
}
