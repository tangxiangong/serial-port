# Native release pipeline

`native.yml` 在各自宿主编译 SwiftUI 与 WinUI；不运行 GUI 自动化。
`release.yml` 在推送版本 tag 时分别生成 macOS arm64 / x86_64 与 Windows x64 / ARM64 包。

配置四个本项目独立 secrets：`SPARKLE_PUBLIC_ED_KEY`、`SPARKLE_PRIVATE_ED_KEY`、
`WINDOWS_UPDATE_PUBLIC_KEY`、`WINDOWS_UPDATE_PRIVATE_KEY`。
版本 tag、workspace/平台工程版本与对应 `release-notes/<version>/` 应同时更新。

发布前先生成双语公告，验证两个平台的包身份、架构、版本、签名、大小与哈希。
`publish.py` 才会上传版本 Release，再上传 Velopack payload，最后推进 `update-feed` 元数据。
版本资产不可变，重试只跳过字节相同的资产；通道文件可以推进，但不会语义降级。
流水线串行发布，保留已有 channel 历史。所有网络写操作只发生在实际执行发布流水线时。

GitHub 更新源可以独立提供更新。COS 镜像需要维护者另行同步已验证的 `update-feed` 内容，
客户端在镜像不可用时回退到 GitHub。此流水线不修改已有 BibCiTeX / FlashDesk 更新源。
macOS 产物默认 ad hoc 签名；配置 Developer ID 和公证是独立的发布验收步骤。
