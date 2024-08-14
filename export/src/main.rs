use std::process::Command;

use chrono::{Duration, Utc};
use clap::Parser;
use env_logger;
use log::{error, info};

use cli::Args;

mod cli;

fn main() {
    println!("Hello, world!");
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    info!("Application Start ... ... !");

    let args = Args::parse();
    info!("Mongo URL: {}", args.mg_url);
    info!("FeiShu hook url: {:?}", args.feishu);

    let now = Utc::now();
    let last_week = now - Duration::days(7);
    let now_str = now.format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
    let last_week_str = last_week.format("%Y-%m-%dT%H:%M:%S.000Z").to_string();

    // 格式化文件名中的日期
    let now_date_str = now.format("%Y%m%d").to_string();
    let last_week_date_str = last_week.format("%Y%m%d").to_string();
    let file_prefix = format!("raydium_actions_{}_{}", last_week_date_str, now_date_str);

    // 定义导出指令
    let mongoexport_cmd = format!(
        "mongoexport --uri=\"mongodb://root:nopassword@127.0.0.1:5011/\" \
        --authenticationDatabase admin \
        --db=raydium --collection=actions \
        --query='{{ \"date_time\": {{ \"$gte\": {{ \"$date\": \"{}\" }}, \"$lt\": {{ \"$date\": \"{}\" }} }} }}' \
        --type=csv --fields=\"date_time,metadata.pair,metadata.signer,signature,token_b,fee,decimals_a,action_type,token_a,slot,amount_b,timestamp,amount_a,decimals_b\" \
        --out={}.csv",
        last_week_str, now_str, file_prefix
    );
    info!("{}", mongoexport_cmd);

    // 执行导出指令
    // let export_status = Command::new("sh")
    //     .arg("-c")
    //     .arg(&mongoexport_cmd)
    //     .status()
    //     .expect("failed to execute mongoexport command");
    //
    // if !export_status.success() {
    //     eprintln!("Error during mongoexport");
    //     return;
    // }

    // 定义压缩指令
    let gzip_cmd = format!("pv {}.csv | gzip > {}.csv.gz", file_prefix, file_prefix);
    info!("{}", gzip_cmd);

    // let gzip_status = Command::new("sh")
    //     .arg("-c")
    //     .arg(&gzip_cmd)
    //     .status()
    //     .expect("failed to execute gzip command");
    //
    // if !gzip_status.success() {
    //     eprintln!("Error during gzip");
    //     return;
    // }

    // 定义移动文件指令
    let mv_cmd = format!("mv {}.csv.gz ~/.file-share-data", file_prefix);
    info!("{}", mv_cmd);
    //
    // let mv_status = Command::new("sh")
    //     .arg("-c")
    //     .arg(&mv_cmd)
    //     .status()
    //     .expect("failed to execute mv command");
    //
    // if !mv_status.success() {
    //     eprintln!("Error during mv");
    //     return;
    // }

    info!("Data export, compression, and move completed successfully.");
}
