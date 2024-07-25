/// @Name cli
///
/// @Date 2024/7/25 下午11:03
///
/// @Author Matrix.Ye
///
/// @Description:
///
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub mg_url: String,

    #[arg(long)]
    pub feishu: Option<String>,
}
