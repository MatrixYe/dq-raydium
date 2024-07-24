# Raydium 全量数据同步器

### 原理
订阅Rar消息


### 运行
```bash
cargo run -- --mg-url "mongodb://example.com" \
--grpc-url "grpc://example.com" \
--grpc-x-token "my_token" \
--retry-interval 30 \
--feishu "http://your_feishu_hook_url.com"
```
参数说明：
- mg-url: mongodb 连接url
- grpc-url: yellowston grpc 连接url
- grpc-x-token: yellowston grpc x-token
- retry-interval: 重试时间间隔，单位秒
- feishu:消息推送地址url，仅限飞书 hook

cargo run -- --mg-url "mongodb://root:nopassword@127.0.0.1:5011/" \
--grpc-url "https://redoubled-skippet-wbhxrrgkog-dedicated-lb.helius-rpc.com:2053" \
--grpc-x-token "34df4c72-5d54-49ba-ae2c-2aa1d50679ce" \
--retry-interval 15 \
--feishu "https://open.feishu.cn/open-apis/bot/v2/hook/1e1c87ee-191d-4513-9daf-6ec739f1b78d"
