# dq-raydium

**基于 Yellowstone Geyser gRPC 流的 Raydium（Solana）AMM 实时数据索引器。**

订阅 Raydium V4 程序的链上活动，把每一笔 swap / 添加流动性 / 移除流动性 / 建池
解析成规范化的 action 记录，并以 MongoDB 原生 time-series 集合存储。

![language](https://img.shields.io/badge/Rust-2021-dea584) ![stream](https://img.shields.io/badge/Yellowstone-grpc-1.14-blue)

---

## 为什么用 gRPC 而不是轮询 `getSignaturesForAddress`

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

---

## 架构

```
        Solana 验证节点 / 启用 Yellowstone 的 RPC
                        │
                        │  Geyser gRPC（SubscribeRequest，Confirmed 承诺级别）
                        ▼
        ┌───────────────────────────────────────────────────────────┐
        │  GeyserGrpcClient                                         │
        │  启动握手：get_version() + health_check()                  │
        │  记录 solana / rustc / proto 版本与 hostname               │
        └───────────────────────────┬───────────────────────────────┘
                                    │  SubscribeUpdate 流
                                    ▼
        ┌───────────────────────────────────────────────────────────┐
        │  core() —— 流循环                                          │
        │    UpdateOneof::Transaction → handle_transaction_update()  │
        │    UpdateOneof::Ping        → 回 Pong（保活）              │
        │    UpdateOneof::Pong        → 忽略                        │
        │    流错误                    → 飞书通知，然后重连           │
        └───────────────────────────┬───────────────────────────────┘
                                    │
                                    ▼
        ┌───────────────────────────────────────────────────────────┐
        │  解码与归类                                                │
        │  1. pair id  ← 外层指令 accounts[1]，要求                  │
        │                program_id == Raydium V4                   │
        │                （兜底：内层指令 inner instructions）        │
        │  2. signer   ← account_keys[0]                            │
        │  3. 差额      ← post_token_balances − pre_token_balances   │
        │                仅取 owner == Raydium Authority 的账户      │
        │  4. action   ← 两个差额的正负号组合                        │
        └───────────────────────────┬───────────────────────────────┘
                                    │
                     合法？ ────────┴──────── 丢弃（记录原因）
                                    │
                                    ▼
        ┌───────────────────────────────────────────────────────────┐
        │  MongoDB  db = "raydium"                                  │
        │   actions   ← time-series 集合                            │
        │               timeField = date_time                       │
        │               metaField = metadata {signer, pair}         │
        │               granularity = seconds                       │
        │   newpairs  ← 每次建池一条文档                             │
        │               唯一索引：signature、pair                    │
        └───────────────────────────┬───────────────────────────────┘
                                    │
                                    ▼
        ┌───────────────────────────────────────────────────────────┐
        │  导出工具                                                  │
        │   raydium-export.sh → mongoexport → CSV → gzip → ~/.file-share-data
        └───────────────────────────────────────────────────────────┘
```

---

## 行为归类

这里不去解析 Raydium 的指令布局（该布局在程序版本之间会变），而是读取
**AMM authority 账户的 token 余额差额**——池子只能动用 authority 持有的资产，
所以差额的正负号组合**就是**行为本身：

| A 差额 | B 差额 | 归类 | 附加条件 |
|---|---|---|---|
| `+` | `−` | `swap` | |
| `−` | `+` | `swap` | |
| `+` | `+` | `add` | 若 `pre_a == 0 && pre_b == 0` 则改判为 `create` |
| `−` | `−` | `remove` | |

差额按 `post − pre` 计算，再取负号，表示**从池子视角**的资产变动。
一笔交易若牵涉的 mint 数量不等于 2，直接丢弃——这样就把聚合器路由和部分失败的交易
排除出数据集。

### 解析器固定的程序 ID

| 常量 | 地址 |
|---|---|
| `RaydiumLiquidityPoolV4` | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` |
| `RaydiumAuthorityV4` | `5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1` |
| `JupiterAggregatorV6` | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` |

---

## 关键工程决策

| 决策 | 理由 |
|---|---|
| **`actions` 用 MongoDB 原生 time-series 集合** | 这个工作负载是只追加、按时间有序、按范围查询。time-series 的桶机制自带压缩，并通过 `metaField` 免费建立二级维度索引；用普通集合加逐文档索引撑不住这个写入速率。 |
| **`metadata` 里只放两个查询维度** | time-series 的 `metaField` 是唯一的带索引二级维度。把 `signer` 和 `pair` 放进去，"查某个池子的全部活动"就是一次廉价查询。 |
| **用 authority 余额差额归类，而不是指令布局** | 对 Raydium 程序升级免疫；一条代码路径，而不是一张需要版本化的 ABI 分发表。 |
| **外层和内层指令都解析以取 pair id** | CPI 路由的 swap（聚合器、机器人）会把 Raydium 调用放在内层指令里。 |
| **收到 `Ping` 必须回 `Pong`** | Geyser 服务端会关掉长期不响应的流。忽略这一点曾被观察到导致订阅掉线。 |
| **`--restart` 选择是否重连** | 长期生产应该带退避重连；一次性调试应该大声失败并退出。做成开关，两种行为都保留。 |
| **生命周期事件推飞书 webhook** | 一个静默死了六小时的索引器，比一个吵闹的索引器更糟。启动、流出错、重连都会推送消息。 |
| **启动时做版本/健康握手** | `yellowstone-grpc-proto` 版本不匹配或端点写错，之前会在收到第一条消息时才失败，而不是在连接阶段。 |
| **全部配置走 `clap`** | 不用配置文件，避免环境变量漂移；启动时把生效配置全量打印出来。 |

---

## 数据模型

`actions`（time-series）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `date_time` | BSON Date | time-series 集合的时间字段 |
| `metadata.signer` | string | 交易签名者（meta 字段） |
| `metadata.pair` | string | AMM id（meta 字段） |
| `signature` | string | 交易签名 |
| `slot` | u64 | slot |
| `timestamp` | i64 | 入库时的墙钟毫秒 |
| `action_type` | string | `create` / `swap` / `add` / `remove` |
| `token_a` / `token_b` | string | mint 地址 |
| `amount_a` / `amount_b` | f64 | 带符号的代币数量（池子视角） |
| `decimals_a` / `decimals_b` | u8 | mint 精度 |
| `fee` | u64 | 交易手续费（lamports） |

`newpairs`（普通集合，`signature` 与 `pair` 上有唯一索引）：`actions` 中
`action_type == "create"` 的子集，重整为 `creator` + `init_amount_a/b`。

---

## 运行

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

### 导出 CSV

```bash
chmod +x ./raydium-export.sh
./raydium-export.sh          # 会提示输入起止日期（yyyyMMdd）和 Mongo 主机
```

输出 `raydium_actions_<起>_<止>.csv.gz` 到 `~/.file-share-data`。

`pump-export.sh` 对另一套 `pump` 数据集（不同的 Mongo 实例）做同样的事。

### 单笔交易排查

`get_tx.py` 通过 RPC 服务商拉取并格式化打印单笔交易——是调试辅助工具，不属于数据管道。
**端点请通过环境变量传入，不要把 API key 硬编码在这个文件里。**

```bash
export HELIUS_RPC_URL="https://your-endpoint?api-key=..."
python get_tx.py
```

---

## 技术栈

```
语言       Rust 2021（tokio 异步运行时）
链         yellowstone-grpc-client / -proto 1.14，solana-sdk 1.17，solana-transaction-status 1.17
存储       mongodb 2.8（tokio-sync）——time-series + 复合/唯一索引
命令行配置  clap 4.5（derive）
可观测性    log + env_logger，chrono，reqwest（飞书 webhook）
```

---

## 局限与已知缺口

写出来而不是藏起来——每一条都是面试里很可能被问到的问题：

- **只有实时，没有历史。** 仓库里没有历史回补，它从当前 slot 开始。重建历史需要走
  `getBlock` / `getTransaction`，`get_tx.py` 只是把这条路演示了一小段。
- **没有重组（reorg）处理。** `commitment = Confirmed` 意味着重组可能让已写入的记录失效。
  生产级索引需要改成 `Finalized`，或者加 slot/确认深度字段并实现回滚。
- **`actions` 集合没有去重。** 文档使用自动生成的 `ObjectId`，所以重连后重放流可能重复写入。
  `newpairs` 有唯一索引保护，`actions` 没有。给 `signature` 加唯一索引
  （或以它为主键做 upsert）就是显然的修法。
- **没有吞吐量和端到端延迟的基准测试。** "实时"是架构上的说法，没有实测数据。
- **单订阅、单进程。** 不按池子分片，也没有消费组；这个进程是单点。
- **解析器有碎片化风险。** 靠硬编码程序 ID 匹配，意味着程序升级就要改代码；
  运行时不会校验这个 ID 是否仍然是 AMM 程序。
- **没有测试和 CI。** 归类逻辑是解码后交易的纯函数，是最值得加测试的部分。
- **没有 LICENSE 文件。**

## 后续计划

1. 给 `signature` 加唯一索引 + 改为 upsert 写入 → 重放幂等。
2. 使用 `Finalized` 承诺级别，或为 `Confirmed` 补一条回滚路径。
3. 增加一个历史回补 bin，与实时订阅共用解码逻辑。
4. 为 `handle_transaction_update` 补单元测试，用捕获的 fixture
   （仓库里已有的 `tx_demo.json` 就可以直接当第一个）。
