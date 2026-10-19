# Serial Studio

原生串口工具：macOS 使用 SwiftUI，Windows 使用 WinUI 3，共享 Rust 核心。
工程结构、桥接与更新框架与 BibCiTeX / FlashDesk 对齐。

- 自动发现串口与热插拔；单活动串口会话。
- 内置 `usb-device.json`，按 VID/PID 补全系统缺失的 USB 产品名与厂商名，macOS / Windows 共用。
- 波特率、5–8 数据位、1/2 停止位、奇/偶校验、XON/XOFF、RTS/CTS。
- HEX、ASCII、GBK、UTF-8 收发，发送与显示编码独立切换；LF、CR、CRLF 行尾。
- GBK / UTF-8 跨读取解码，非法 ASCII / GBK 发送明确报错。
- RX / TX 计数；终端保留最多 2000 条且限制内存，清空显示保留统计。
- 原生外观、菜单与键盘快捷键。
- Sparkle / Velopack 检查、下载、安装；stable / beta / alpha 通道与分平台、双语公告。
- 活动串口期间延后安装，退出关闭串口。

## 原生工程

- **Xcode**：[macos/SerialStudio.xcodeproj](macos/SerialStudio.xcodeproj)，运行 SerialStudio scheme。
- **Visual Studio**：[windows/SerialStudio.sln](windows/SerialStudio.sln)，选择 x64 或 ARM64。

两个工程在编译前自动构建 Rust 并生成桥接，干净检出无需预生成绑定。
macOS 14+；Windows 10 2004+。环境详见 [macOS](macos/README.md) / [Windows](windows/README.md)。

```sh
./script/build_and_run.sh
```

```powershell
./windows/scripts/build.ps1 -Architecture x64
```

## 结构

```text
crates/
  serial-studio-core/       串口、缓冲、编码与类型
  serial-studio-service/    共享桌面服务与运行时
  serial-studio-ffi/        Swift UniFFI 导出与远程类型
  serial-studio-csharp/     Windows Interoptopus 导出
bindings/
  generator/               官方 UniFFI 生成器
  csharp-generator/        Interoptopus C# 生成器
macos/
  SerialStudio.xcodeproj/   Xcode 工程与共享 scheme
  SerialStudio/Sources/     SwiftUI 应用
  build-bindings.sh         Xcode 桥接构建步骤
  build.sh
windows/
  SerialStudio.sln          VS 解决方案
  SerialStudio/            WinUI 工程与更新器
  scripts/
ci/                        原生平台发布脚本
release-notes/             双语、分平台更新公告
xtask/                     公告生成与签名更新 feed
```

macOS 通过 **UniFFI**，Windows 通过 **Interoptopus** 调用共享服务。
生成器产物位于被忽略的 `bindings/generated/`，不手工修改。

## 验证

默认进行本机逻辑与静态检查，不添加 GUI 自动化测试：

```sh
cargo test --locked -p serial-studio-core -p serial-studio-csharp -p xtask --lib
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

客户端分别在对应宿主构建。真机串口收发、插拔、更新安装与重启须单独验收。

## 更新与发布

- macOS：Sparkle 2，Ed25519 签名归档、原生更新 UI、架构/通道/语言 appcast。
- Windows：Velopack，P-256 签名 feed、包大小与 SHA-256 校验、下载并重启安装。
- 优先使用 `https://app-release-1302963684.cos.ap-guangzhou.myqcloud.com/serial-port/update-feed`，
  失败后回退到本仓库 GitHub 的 `update-feed` Release。镜像目录由维护者配置发布，GitHub 回退可独立工作。
- 缺少验证公钥的开发构建明确停用更新，不复用其他产品密钥。
- 发布前过滤平台，再按系统语言选择公告，详见 [公告格式](release-notes/README.md)。

配置本项目独立的 `SPARKLE_PUBLIC_ED_KEY` / `SPARKLE_PRIVATE_ED_KEY`、
`WINDOWS_UPDATE_PUBLIC_KEY` / `WINDOWS_UPDATE_PRIVATE_KEY`。
Windows 公钥为 Base64 DER SPKI、私钥为 Base64 DER PKCS#8 的 P-256 密钥；
Sparkle 密钥使用官方工具生成。私钥只交给发布环境，不加入源码。开发构建不需要这些密钥。

支持 `vX.Y.Z`、`vX.Y.Z-beta.N`、`vX.Y.Z-alpha.N` tag，
每版提供 `release-notes/<version>/{zh-Hans,en}.md`。
发布脚本和签名工具位于 `ci/`、`windows/scripts/` 和 `xtask/`。
稳定版可晋升到 beta/alpha，通道按语义版本防止回退。

迁移建立本地工程与发布链路，不会自动发布现有构建。
实际公钥、更新源、签名/公证和安装升级需在发布前配置并验收。
Sparkle 更新签名与 Apple Developer ID 签名/公证分别配置，ad hoc 签名不代表通过 Gatekeeper 分发验收。

## 许可证

MIT OR Apache-2.0。
