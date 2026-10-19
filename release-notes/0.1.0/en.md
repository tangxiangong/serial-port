# Serial Studio 0.1.0

- Preserve serial discovery, hotplug, configuration, byte counters, and the terminal.
- Support HEX, ASCII, GBK, and UTF-8, including multibyte receive boundaries.
- Share serial behavior between both native clients through the Rust core.

<!-- platform:macos -->
## macOS

- Migrate to native SwiftUI and include an Xcode project.
- Use UniFFI for Rust bindings and Sparkle to check, download, and install updates.
<!-- /platform -->

<!-- platform:windows -->
## Windows

- Add a native WinUI 3 client and a Visual Studio solution.
- Use Interoptopus for Rust bindings and Velopack to check, download, and install updates.
<!-- /platform -->
