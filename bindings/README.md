# Native bindings

与 BibCiTeX 相同，应用只调用生成的类型与函数，不维护手写 C ABI 或 JSON RPC。

- `generator/`：官方 UniFFI CLI，从 `crates/serial-studio-ffi` 生成 Swift、C 头与 modulemap。
- `csharp-generator/`：从 `crates/serial-studio-csharp` 的 inventory 生成 Interoptopus C# 绑定。
- `generated/`：本地生成产物，不入库。

Xcode 的 `RustBindings` target 和 VS 的 `Build/Rust.targets` 在编译前自动生成。
macOS 支持单架构与通用静态库；Windows 生成器在宿主架构运行，应用 DLL 按选中的 x64/ARM64 构建。
