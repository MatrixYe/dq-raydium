# Raydium 全量数据同步器

### 原理
订阅yellowston grpc消息并进行消息解析和存储


### 运行同步解析器
```bash
cargo run --bin sync --release -- --mg-url "你的mongodb数据库URL" \
--grpc-url "你的grpc连接" \
--grpc-x-token "你的grpc连接凭证" \
--retry-interval 30 \
--feishu "你的推送feishu URL"
```
参数说明：
- mg-url: mongodb 连接url
- grpc-url: yellowston grpc 连接url
- grpc-x-token: yellowston grpc x-token
- retry-interval: 可选，默认15s，重试时间间隔，单位秒
- feishu:可选，默认None，消息推送地址url，仅限飞书 hook

1. 常规模式
```shell
cargo run --bin sync --release -- --mg-url "mongodb://root:nopassword@127.0.0.1:5011/" \
--grpc-url "https://redoubled-skippet-wbhxrrgkog-dedicated-lb.helius-rpc.com:2053" \
--grpc-x-token "34df4c72-5d54-49ba-ae2c-2aa1d50679ce" \
--retry-interval 5 \
--feishu "https://open.feishu.cn/open-apis/bot/v2/hook/1e517e58-39dc-4b53-bb44-9b8914e09e8c" \
--restart
```
2. 安静模式
```shell
cargo run --bin sync --release -- --mg-url "mongodb://root:nopassword@127.0.0.1:5011/" \
--grpc-url "https://redoubled-skippet-wbhxrrgkog-dedicated-lb.helius-rpc.com:2053" \
--grpc-x-token "34df4c72-5d54-49ba-ae2c-2aa1d50679ce" \
--retry-interval 5
```


### 运行数据导出脚本
将mongo中的数据，action集合导出为csv文件，执行脚本时需要输入时间范围，比如`20240807`
```shell
chmod +x ./raydium-export.sh

./raydium-export
```
