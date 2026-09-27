# Serial Studio

一款专注串口通信的 macOS 桌面工具，使用 Tauri 2、Rust、SolidJS 构建。

## 功能

- 自动发现串口设备并监听设备插拔
- 单活动串口会话，支持常见波特率、数据位、停止位、校验位和流控制
- 发送支持 HEX、ASCII、GBK、UTF-8 切换；文本模式支持 LF、CR、CRLF 行尾
- RX / TX 实时终端可独立于发送格式切换 HEX、ASCII、GBK、UTF-8 显示，保留收发字节统计
- UTF-8 / GBK 接收支持跨读取的字符拼接；ASCII / GBK 发送遇到无法编码的字符会报错
- macOS 原生标题栏、浅色与深色外观

本项目不包含 BLE、网络、Modbus 或串口桥接功能。

## 开发

```bash
bun install
bun run tauri dev
```

## 验证

```bash
bun run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
```
