## Dq-raydium

**基于 Yellowstone Geyser gRPC 流的 Raydium（Solana）AMM 实时数据索引器。**

订阅 Raydium V4 程序的链上活动，把每一笔 swap / 添加流动性 / 移除流动性 / 建池
解析成规范化的 action 记录，并以 MongoDB 原生 time-series 集合存储。

轮询 Raydium 的 AMM 程序意味着每天几百万条签名，并且每条交易一次往返请求——
RPC 账单和延迟都随链上活跃度线性增长。

Geyser 插件流会推送**已完成解码的交易**，更重要的是它允许**由服务端应用过滤条件**：

```
account_include = [675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8]   // Raydium V4 AMM
vote   = false     // 丢弃验证者投票交易
failed = false     // 丢弃失败回滚的交易
commitment = Confirmed
```

只有命中的交易会真正过网络。一条长连接订阅替代了整个轮询循环。
### 运行

```bash
cargo build --release

cargo run --bin sync --release -- \
  --mg-url          "mongodb://user:pass@host:27017/" \
  --grpc-url        "https://your-geyser-endpoint" \
  --grpc-x-token    "your-geyser-token" \
  --retry-interval  15 \
  --restart
```

| 参数 | 必填 | 默认 | 含义 |
|---|---|---|---|
| `--mg-url` | 是 | — | MongoDB 连接串 |
| `--grpc-url` | 是 | — | Yellowstone Geyser gRPC 端点 |
| `--grpc-x-token` | 是 | — | gRPC `x-token` 鉴权头 |
| `--retry-interval` | 否 | `15` | 重连前等待秒数 |
| `--feishu` | 否 | 无 | 飞书 webhook，用于生命周期通知 |
| `--restart` | 否 | `false` | 流出错时重连。不加则进程退出。 |

建表是自动的：首次运行创建 `actions` time-series 集合，每次启动都会确保 `newpairs` 的索引存在。

#### 导出 CSV

```bash
chmod +x ./raydium-export.sh
./raydium-export.sh          # 会提示输入起止日期（yyyyMMdd）和 Mongo 主机
```

输出 `raydium_actions_<起>_<止>.csv.gz` 到 `~/.file-share-data`。

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.