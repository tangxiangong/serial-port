# 分平台更新公告

与 BibCiTeX 使用相同格式：每个版本提供 `zh-Hans.md` 和 `en.md`。
未标记的段落属于全部平台，旧公告保持兼容。

```markdown
# Serial Studio 0.1.0

- 公共改进。

<!-- platform:macos -->
## macOS
- SwiftUI 与 Sparkle 更新。
<!-- /platform -->

<!-- platform:windows -->
## Windows
- WinUI 与 Velopack 更新。
<!-- /platform -->
```

标记独占一行，支持 `macos`、`windows`、`linux`、`all` 和逗号分隔的平台列表。
`all` 必须单独使用。区块不能嵌套；未知平台、格式错误和未配对标记都会导致生成失败，
包括被排除的区块和代码围栏内的标记。平台专属标题应写在对应区块内。

```sh
cargo run --locked -p xtask -- release-notes release-notes/0.1.0 target/release-notes/0.1.0
```

两种语言全部校验成功后才输出：

- `notes-macos-{locale}.md`：Sparkle appcast 引用的 macOS 公告。
- `notes-windows-{locale}.md`：Windows 单语言公告。
- `notes-windows.md`：带 `locale:zh-Hans/en` 标记的双语公告，传给 Velopack 的 `--releaseNotes`。
- `release-body.md`：GitHub 使用的全部平台、双语公告，保留可见的平台标题。
- `notes-{locale}.md` 与 Linux 过滤结果：与 BibCiTeX 的生成器兼容；不代表本项目提供 Linux 客户端。

客户端不从 GitHub Release 正文猜测平台。发布前先过滤平台，再由客户端选择语言。
