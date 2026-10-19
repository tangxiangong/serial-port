# Windows

直接打开 `SerialStudio.sln`，选择 Debug/Release 与 x64/ARM64，运行 SerialStudio。
要求 Windows 10 2004+、支持 .NET 10 的 Visual Studio、Windows SDK、WinUI 工作负载、
MSVC C++ 构建工具与 Rust MSVC 工具链。

```powershell
rustup target add x86_64-pc-windows-msvc
# ARM64 构建还需 VS ARM64 C++ 工具和 aarch64-pc-windows-msvc target。
./windows/scripts/build.ps1 -Architecture x64
```

`SerialStudio/Build/Rust.targets` 在正常构建时编译 Rust DLL、运行 Interoptopus 生成器并纳入生成的 C#。
IntelliSense 的 design-time build 不启动 Cargo，干净检出不依赖预生成的 `Interop.cs`。
发布包携带 .NET 与 Windows App SDK 运行时。

Velopack installer hooks 先于 WinUI 与 Rust 初始化运行。
安装版且配置 `WINDOWS_UPDATE_PUBLIC_KEY` 后启用更新。
`scripts/publish.ps1` 生成 Velopack 安装器与完整更新包，根目录 `xtask` 生成签名 feed。
此工程需在 Windows 上实际构建和验收安装升级。
