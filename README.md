# Serial Studio

一款专注串口通信的 macOS 桌面工具，使用 Tauri 2、Rust、SolidJS 构建。

## 功能

- 自动发现串口设备并监听设备插拔
- 单活动串口会话，支持常见波特率、数据位、停止位、校验位和流控制
- 文本或十六进制发送，支持 LF、CR、CRLF 行尾
- RX / TX 实时终端、文本与 HEX 组合视图、收发字节统计
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
