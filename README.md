# netutil

一个用 Rust 编写的网络协议工具，支持 HTTP、WebSocket、TCP、UDP、ICMP 等协议。

## 安装

### 从源码编译

```bash
# 需要 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

git clone <repo-url>
cd netutil
cargo build --release

# 安装到 PATH
cp target/release/netutil ~/.local/bin/
```

### 交叉编译

```bash
# 编译 Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# 编译 Linux aarch64
cargo build --release --target aarch64-unknown-linux-gnu

# 编译 macOS
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# 编译 Windows
cargo build --release --target x86_64-pc-windows-gnu
```

## 使用方式

```
netutil [OPTIONS] <TARGET>
```

### HTTP / HTTPS

```bash
# GET 请求
netutil "https://ping.beamof.com"
netutil "https://ping.beamof.com" -t "GET"

# POST 请求
netutil "https://httpbin.org/post" -t "POST" -d '{"hello":"world"}' -H "Content-Type: application/json"

# PUT / DELETE / PATCH
netutil "https://httpbin.org/put" -t "PUT" -d '{"key":"value"}'
netutil "https://httpbin.org/delete" -t "DELETE"

# 显示响应头
netutil "https://example.com" --show-headers

# 自定义超时
netutil "https://example.com" --timeout 30
```

### WebSocket

```bash
# 连接 WebSocket 并发送消息
netutil "wss://echo.websocket.org" --ws -m "hello"

# 自定义超时
netutil "wss://echo.websocket.org" --ws -m "hello" --timeout 5
```

### TCP

```bash
# TCP 连接并发送原始数据
netutil "tcp://example.com:80" -d "GET / HTTP/1.0\r\nHost: example.com\r\n\r\n"

# 连接 Redis
netutil "tcp://127.0.0.1:6379" -d "PING\r\n"
```

### UDP

```bash
# 发送 UDP 数据报
netutil "udp://8.8.8.8:53" -d "test"

# 显示响应
netutil "udp://8.8.8.8:53" -d "test" -v
```

### ICMP (Ping)

```bash
# Ping（默认 4 次）
netutil "8.8.8.8" --icmp

# 指定次数
netutil "8.8.8.8" --icmp -c 10

# 简写（纯 IP 地址自动识别为 ICMP）
netutil "8.8.8.8" -c 4
```

## 参数说明

| 参数 | 缩写 | 说明 | 默认值 |
|------|------|------|--------|
| `--type` | `-t` | HTTP 方法 (GET/POST/PUT/DELETE/HEAD/OPTIONS/PATCH) | `GET` |
| `--data` | `-d` | 请求体 / 发送数据 | - |
| `--header` | `-H` | 自定义请求头（可重复） | - |
| `--count` | `-c` | ICMP ping 次数 | `4` |
| `--message` | `-m` | WebSocket 发送消息 | `hello` |
| `--ws` | - | 强制使用 WebSocket 协议 | - |
| `--icmp` | - | 强制使用 ICMP 协议 | - |
| `--timeout` | - | 超时时间（秒） | `10` |
| `--show-headers` | - | 显示 HTTP 响应头 | - |
| `--verbose` | `-v` | 详细输出 | - |

## 协议自动识别

根据目标地址前缀自动选择协议：

| 前缀 | 协议 |
|------|------|
| `https://` / `http://` | HTTP |
| `wss://` / `ws://` | WebSocket |
| `tcp://` | TCP |
| `udp://` | UDP |
| `ping://` / `icmp://` | ICMP |
| 纯 IP 或域名（无端口） | ICMP |
| `host:port` 格式 | TCP |

## 输出示例

```
$ netutil "https://ping.beamof.com" -t "GET"
> GET https://ping.beamof.com
< 200 OK
<
{"ip": "1.2.3.4", "loc": "Shenzhen / Guangdong / CN"}
```

```
$ netutil "https://httpbin.org/post" -t "POST" -d '{"hello":"world"}' -H "Content-Type: application/json" --show-headers
> POST https://httpbin.org/post
> Content-Length: 17
< 200 OK
< content-type: application/json
< server: gunicorn/19.9.0
<
{"json": {"hello": "world"}, "origin": "1.2.3.4"}
```

```
$ netutil "8.8.8.8" --icmp -c 2
PING 8.8.8.8 (8.8.8.8) 56(84) bytes of data.
64 bytes from 8.8.8.8: icmp_seq=1 ttl=113 time=38.3 ms
64 bytes from 8.8.8.8: icmp_seq=2 ttl=113 time=38.5 ms
--- 8.8.8.8 ping statistics ---
2 packets transmitted, 2 received, 0% packet loss
rtt min/avg/max/mdev = 38.269/38.370/38.472/0.101 ms
```

## 技术栈

- [clap](https://crates.io/crates/clap) — CLI 参数解析
- [reqwest](https://crates.io/crates/reqwest) — HTTP 客户端
- [tokio](https://crates.io/crates/tokio) — 异步运行时
- [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite) — WebSocket 客户端
- [futures-util](https://crates.io/crates/futures-util) — 异步工具
- [colored](https://crates.io/crates/colored) — 终端彩色输出
- [anyhow](https://crates.io/crates/anyhow) — 错误处理

## License

MIT
