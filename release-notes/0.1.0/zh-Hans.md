# Serial Studio 0.1.0

- 保留串口自动发现、热插拔、通信参数、收发计数和数据终端。
- 支持 HEX、ASCII、GBK、UTF-8；GBK 和 UTF-8 接收可跨读取拼接字符。
- 共享 Rust 核心为两个原生客户端提供一致的串口行为。

<!-- platform:macos -->
## macOS

- 迁移为 SwiftUI 原生应用，提供 Xcode 工程。
- 通过 UniFFI 接入 Rust，通过 Sparkle 检查、下载并安装更新。
<!-- /platform -->

<!-- platform:windows -->
## Windows

- 新增 WinUI 3 原生应用与 Visual Studio 解决方案。
- 通过 Interoptopus 接入 Rust，通过 Velopack 检查、下载并安装更新。
<!-- /platform -->
