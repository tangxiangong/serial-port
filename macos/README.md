# macOS

直接打开 `SerialStudio.xcodeproj`，运行 `SerialStudio` scheme。
要求 macOS 14+、Xcode 与 Rust。首次构建安装对应 Rust target：

```sh
rustup target add aarch64-apple-darwin # Intel 使用 x86_64-apple-darwin
./macos/build.sh
```

依赖顺序为 `RustBindings → SerialStudioCore.framework → SerialStudio.app`。
`RustBindings` 自动调用 Cargo 与官方 UniFFI 生成器，干净检出无需先手动生成绑定。
Sparkle 由 Xcode 的 Swift Package Manager 解析并嵌入。

`project.yml` 是工程维护源，修改后运行 `xcodegen generate --spec macos/project.yml`，
并提交生成的工程与共享 scheme。正常打开 Xcode 不需要 XcodeGen。

```sh
./script/build_and_run.sh
./script/build_and_run.sh --logs
ARCH=x86_64 CONFIGURATION=Release ./macos/build.sh
```

发布构建读取 `SPARKLE_PUBLIC_ED_KEY`、`APP_VERSION`、`BUILD_NUMBER`。
没有公钥的开发构建明确显示更新不可用，不复用其他产品密钥。
公钥与更新地址在打包副本写入后重新签名；Xcode 直接运行开发构建不要求公钥。
