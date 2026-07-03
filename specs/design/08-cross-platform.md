# 跨平台与系统服务设计文档

> 本文档描述 Warp 的跨平台抽象层设计、四大平台（macOS / Linux / Windows / WASM）的具体实现，以及系统级服务 crate 的架构与职责。

---

## 1. 平台抽象层

### 1.1 warpui_core 平台 Trait

Warp 不依赖跨平台 GUI 框架（如 Qt/GTK）来屏蔽平台差异，而是在 `warpui_core` 中定义一组纯 Rust trait，由各平台提供具体实现：

| Trait | 职责 |
|-------|------|
| `PlatformDelegate` | 应用级生命周期：启动、退出、打开 URL、文件关联 |
| `PlatformWindow` | 单窗口操作：设置标题、全屏、最小化、关闭 |
| `PlatformWindowManager` | 多窗口管理：创建窗口、列出窗口、置焦 |
| `PlatformFontDB` | 字体枚举与加载（平台原生字体系统） |
| `PlatformMenu` | 菜单栏与上下文菜单构建 |
| `PlatformCursor` | 光标样式设置（箭头、文本、手型等） |
| `PlatformClipboard` | 系统剪贴板读写 |
| `PlatformScreen` | 屏幕信息：尺寸、缩放比、可用工作区 |

```rust
// 示意：各平台需实现的 trait
pub trait PlatformDelegate {
    fn name(&self) -> &'static str;
    fn initialize(&mut self, app: &mut AppContext);
    fn on_quit_request(&mut self, app: &mut AppContext);
}
```

### 1.2 warpui 平台实现分层

```
warpui/src/
├── platform.rs        — 平台抽象 trait 定义
├── mac/               — macOS 实现（Cocoa/AppKit）
│   ├── mod.rs
│   ├── app_delegate.rs
│   ├── window.rs
│   ├── menu.rs
│   └── fonts.rs
├── linux/             — Linux 实现（winit + wgpu）
│   ├── mod.rs
│   ├── window.rs
│   ├── fonts.rs       — fontconfig
│   └── dbus.rs        — DBus 集成
├── windows/           — Windows 实现（winit + wgpu）
│   ├── mod.rs
│   ├── window.rs
│   ├── fonts.rs       — DWrite
│   └── winrt.rs
└── wasm/              — WASM 实现（WebGL/Canvas）
    ├── mod.rs
    ├── renderer.rs
    └── bindings.rs
```

### 1.3 编译时选择

通过 `#[cfg(target_os = "...")]` 在编译时选择平台实现。WASM 目标使用 `cfg(target_arch = "wasm32")` 区分。

---

## 2. macOS 平台

### 2.1 架构

```
mac/
├── app_delegate.rs     — NSApplicationDelegate 实现
├── window.rs           — 窗口创建与管理（NSWindow）
├── menu.rs             — 菜单栏构建（MenuBarBuilder）
├── fonts.rs            — CoreText 字体枚举
├── dock.rs             — Dock 菜单集成
├── clipboard.rs        — NSPasteboard 封装
├── autorelease.rs      — NSAutoreleasePool 管理
└── helpers.rs          — 平台工具函数
```

### 2.2 NSApplication 集成

Warp 直接与 Cocoa/AppKit 交互（通过 `objc2` crate），而非经过 winit：

- `NSApplication` 启动与事件循环通过 `AppDelegate` 桥接到 Rust
- `AppDelegate` 响应 `applicationWillTerminate`、`openURL`、`openFile` 等系统事件
- `NSAutoreleasePool` 在每个事件循环迭代中 drain，通过 `autorelease.rs` 封装

### 2.3 菜单栏

菜单栏通过 `MenuBarBuilder` DSL 构建：

```rust
// 示意：MenuBarBuilder 使用模式
MenuBarBuilder::new()
    .app_menu(|m| m.about("Warp").quit())
    .file_menu(|m| m.new_tab().open().close_window())
    .edit_menu(|m| m.undo().redo().cut().copy().paste())
    .view_menu(|m| m.toggle_fullscreen().zoom_in().zoom_out())
    .window_menu(|m| m.minimize().bring_all_to_front())
    .help_menu(|m| m.search("Search").link("Documentation"))
    .build()
```

系统菜单事件通过 Action 系统分发到 View 树。

### 2.4 Dock 菜单

- 通过 `NSApplication.shared.setDockMenu` 设置
- 支持快速新建窗口、打开最近文档
- 通过 Dock 右键菜单响应事件

---

## 3. Linux 平台

### 3.1 架构

```
linux/
├── window.rs           — winit 窗口封装
├── renderer.rs         — wgpu 渲染器（Vulkan backend）
├── fonts.rs            — fontconfig 字体枚举
├── dbus.rs             — DBus 服务集成
│   ├── secret_service  — Secret Service API（密钥链）
│   └── portal          — xdg-desktop-portal 集成
├── clipboard.rs        — wayland/clipboard 处理
├── platform_trait.rs   — LinuxPlatformDelegate
└── mod.rs
```

### 3.2 窗口与渲染

- 基于 **winit**（窗口创建与事件循环）+ **wgpu**（GPU 渲染，Vulkan backend）
- 支持 X11 与 Wayland 两种显示协议
- `force_x11` 选项：当 Wayland 兼容性不足时，可强制使用 X11 backend
- 窗口装饰使用 CSD（Client-Side Decoration）

### 3.3 字体系统

- 使用 **fontconfig**（`fc-*`）枚举系统字体、按 pattern 匹配
- 回退链：用户配置 → fontconfig fallback → 内置备用字体
- 支持的格式：TTF、OTF、WOFF2（通过 `ttf-parser` + `fontdb` 处理）

### 3.4 DBus 集成

通过 `zbus` crate 与桌面环境通信：

| 服务 | 用途 |
|------|------|
| **Secret Service**（`org.freedesktop.Secrets`） | 存储 API token、SSH 密钥密码 |
| **xdg-desktop-portal**（`org.freedesktop.portal.*`） | 文件选择器、打开 URL、截图权限 |
| **Notification**（`org.freedesktop.Notifications`） | 系统通知 |

### 3.5 Wayland 注意事项

- 剪贴板处理差异：Wayland 需要 `wl_data_device` 协议，不支持传统 X11 `CLIPBOARD` selection
- Drag-and-drop 使用 `wl_data_device` 拖放协议
- Wayland 下不支持全局坐标/窗口位置获取（安全限制）

---

## 4. Windows 平台

### 4.1 架构

```
windows/
├── window.rs           — winit 窗口封装
├── renderer.rs         — wgpu 渲染器（DirectX 12 backend）
├── fonts.rs            — DirectWrite 字体枚举
├── winrt.rs            — WinRT API 集成
├── pinvoke.rs          — P/Invoke 绑定
├── clipboard.rs        — Win32 clipboard API
└── mod.rs
```

### 4.2 窗口与渲染

- 基于 **winit** + **wgpu**（DirectX 12 backend）
- Win32 消息循环由 winit 管理，Warp 通过 winit 事件回调接入
- DPI 感知：通过 `SetProcessDPIAware` / `GetDpiForWindow` 处理缩放

### 4.3 字体系统

- 使用 **DirectWrite**（`dwrite` crate）枚举系统字体
- 支持字体回退链、可变字体、颜色字体（COLR/CPAL）
- 通过 `IDWriteFontCollection` 遍历系统字体集

### 4.4 WinRT/PInvoke

- WinRT：通过 `windows-rs` crate 调用现代 Windows API（通知、设置、账户）
- PInvoke：直接 FFI 调用 Win32 API（剪贴板、窗口管理、注册表）
- 使用场景：系统主题检测、深色模式切换、Windows Hello 认证

### 4.5 Windows 特有行为

- 窗口句柄（`HWND`）缓存用于 Win32 消息自定义处理
- 安装路径检测使用注册表（`HKEY_LOCAL_MACHINE`）
- 自动更新使用 Windows 原生更新机制

---

## 5. WASM 平台

### 5.1 架构

```
wasm/
├── renderer.rs         — WebGL / Canvas 渲染器
├── bindings.rs         — JS bridge / wasm-bindgen 绑定
├── paste.rs            — PasteListener 实现
├── network.rs          — 网络连接监听（online/offline）
├── theme.rs            — 系统主题监听（prefers-color-scheme）
├── viewport.rs         — Viewport resize 处理
└── mod.rs
```

### 5.2 渲染

- 两个渲染路径：
  - **WebGL**：通过 `wgpu` WASM backend → WebGL2（默认路径，性能优先）
  - **Canvas 2D**：回退路径（浏览器不支持 WebGL2 时）
- 渲染帧通过 `requestAnimationFrame` 驱动
- 字体从 Web 字体 URL 加载，或使用 `localStorage` 缓存

### 5.3 浏览器集成

| 功能 | 实现 |
|------|------|
| **剪贴板粘贴**（PasteListener） | `navigator.clipboard.readText()` + `paste` 事件监听 |
| **网络状态** | `navigator.onLine` + `online`/`offline` 事件 |
| **系统主题** | `window.matchMedia("(prefers-color-scheme: dark)")` 监听 |
| **Viewport** | `resize` 事件 + `ResizeObserver` |

### 5.4 限制与适配

- 无原生文件系统访问：通过虚拟 FS + IndexedDB 持久化
- 无原生子进程：WASI / Web Worker 替代
- 无原生 GPU 管线：wgpu 限制为 WebGL2 特性子集

---

## 6. 进程派生（crates/command）

### 6.1 设计动机

直接使用 `std::process::Command` 存在以下问题：

- **Windows 弹窗问题**：子进程崩溃或需要 GUI 时弹出不可预期的窗口
- **跨平台一致性**：各平台创建进程的标志位、环境变量处理方式不同
- **安全约束**：某些上下文需要禁止子进程创建新窗口

### 6.2 统一接口

```rust
// 示意：CommandBuilder 接口
pub struct CommandBuilder {
    program: PathBuf,
    args: Vec<String>,
    env: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    no_window: bool,          // Windows 关键标志
    create_new_console: bool,
}
```

关键方法：

| 方法 | 说明 |
|------|------|
| `new(program)` | 创建 builder |
| `arg(arg) / args(args)` | 添加参数 |
| `env(key, value)` | 设置环境变量 |
| `working_dir(path)` | 设置工作目录 |
| `no_window(true)` | **Windows 特有**：设置 `CREATE_NO_WINDOW` 标志 |
| `spawn()` | 派生子进程，返回 `CommandHandle` |
| `output()` | 阻塞等待输出 |
| `kill()` | 终止进程 |

### 6.3 平台差异化

| 行为 | Windows | macOS / Linux |
|------|---------|---------------|
| `no_window` | `CREATE_NO_WINDOW` flag | 无操作（忽略） |
| 进程组 | 使用 Job Object | 使用 `setsid` / `pgid` |
| 路径查找 | `CreateProcess` + 环境 PATH | `execvp` 语义 |

### 6.4 使用约束

- 新派生子进程一律走 `crates/command`，禁止直接使用 `std::process::Command`
- `NoWindow` 标志在使用 GUI 工具（编辑器、浏览器）派生时应设置为 `false`

---

## 7. 秘密管理（crates/managed_secrets）

### 7.1 抽象层

提供统一的密钥/凭据存储接口，屏蔽各平台原生密钥链差异：

```rust
// 示意：SecretStore trait
pub trait SecretStore {
    fn set(&self, service: &str, key: &str, value: &[u8]) -> Result<()>;
    fn get(&self, service: &str, key: &str) -> Result<Option<Vec<u8>>>;
    fn delete(&self, service: &str, key: &str) -> Result<()>;
}
```

### 7.2 平台实现

| 平台 | 后端 | 说明 |
|------|------|------|
| **macOS** | Keychain（`Security.framework`） | 通过 `security` crate 调用 `SecItemAdd`/`SecItemCopyMatching` |
| **Windows** | DPAPI（`CryptProtectData`） | 通过 Win32 API 加密数据到当前用户上下文中 |
| **Linux** | 多种后端自动选择 | Secret Service（DBus）→ Keyring → 加密文件 fallback |
| **WASM** | 代理（`managed_secrets_wasm`） | 通过 JS bridge 调用浏览器 `navigator.credentials` 或 IndexedDB |

### 7.3 Linux 后端选择策略

1. **Secret Service**（通过 DBus `org.freedesktop.Secrets`）— 最优先
2. **Linux Keyring**（`keyctl` syscall）— 后备
3. **加密存储文件**（AES-GCM 加密写入 `~/.local/share/warp/secrets`）— 最终 fallback

### 7.4 WASM 代理

`managed_secrets_wasm` 作为独立 WASM 编译目标（`crates/managed_secrets_wasm`），通过 `wasm-bindgen` 导出 JavaScript API，再由 Rust 主 WASM 模块 import。

---

## 8. 文件系统监视（crates/watcher）

### 8.1 设计

基于 `notify` crate 的文件系统事件监听封装：

```rust
// 示意：FileWatcher API
pub struct FileWatcher {
    inner: RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(paths: &[PathBuf], recursive: bool) -> Result<Self>;
    pub fn events(&self) -> Receiver<FileChangeEvent>;
}

pub enum FileChangeEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Removed(PathBuf),
    Renamed(PathBuf, PathBuf),  // from → to
}
```

### 8.2 平台差异

| 平台 | 后端 | 限制 |
|------|------|------|
| macOS | FSEvents | 高容量事件流，递归监视效率高 |
| Linux | inotify | 需手动处理目录递归（`watcher` crate 封装递归注册） |
| Windows | ReadDirectoryChangesW | 需处理长路径和卷挂载点 |
| WASM | 不支持（返回空事件流） | 通过轮询模拟 |

### 8.3 防抖动

内部 debounce 机制（200ms 窗口）聚合相同文件的事件，避免频繁触发重新索引。

---

## 9. 虚拟文件系统（crates/virtual_fs）

### 9.1 统一接口

```rust
// 示意：VirtualFs trait
pub trait VirtualFs: Send + Sync {
    fn read(&self, path: &Path) -> Result<Vec<u8>>;
    fn write(&self, path: &Path, data: &[u8]) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn list(&self, dir: &Path) -> Result<Vec<PathBuf>>;
    fn metadata(&self, path: &Path) -> Result<FileMetadata>;
}
```

### 9.2 实现

| 实现 | 用途 |
|------|------|
| `RealFs` | 生产环境：封装 `std::fs` + 平台特定路径处理 |
| `MockFs` | 测试：内存中文件树，不触及磁盘 |
| `ReadOnlyFs` | 包装器：拒绝 `write` 操作 |
| `PrefixFs` | 包装器：为路径加上固定前缀（用于 chroot 风格隔离） |

### 9.3 测试优势

测试使用 `MockFs` 完全在内存中操作，不依赖磁盘环境，可实现快速、确定性的文件操作测试。

---

## 10. 休眠抑制（crates/prevent_sleep）

### 10.1 动机

AI Agent 执行长时间任务（代码生成、批量测试运行）期间，系统休眠会中断工作流。需要在任务运行时阻止系统进入睡眠状态。

### 10.2 接口

```rust
// 示意：SleepInhibitor
pub struct SleepInhibitor {
    #[cfg(target_os = "macos")]
    assertion_id: IOPMAssertionID,
}

impl SleepInhibitor {
    pub fn new(reason: &str) -> Result<Self>;
}

impl Drop for SleepInhibitor {
    fn drop(&mut self) { /* 释放抑制 */ }
}
```

### 10.3 平台实现

| 平台 | API |
|------|-----|
| **macOS** | `IOPMAssertionCreateWithName(kIOPMAssertionTypeNoDisplaySleep)` |
| **Linux** | `systemd-inhibit` 或 DBus `org.freedesktop.login1.Manager.Inhibit()` |
| **Windows** | `SetThreadExecutionState(ES_CONTINUOUS \| ES_SYSTEM_REQUIRED)` |
| **WASM** | 不可用（浏览器自动处理休眠） |

### 10.4 使用模式

RAII 模式：`SleepInhibitor` 在作用域内持有抑制锁，离开作用域自动释放。

---

## 11. 隔离平台（crates/isolation_platform）

### 11.1 动机

Warp 在以下环境运行时需要特殊兼容处理：

- **Docker 容器**：无 GPU、无 DRI、无 PulseAudio
- **GitHub Actions / CI**：无交互式终端、无 DISPLAY
- **SSH 会话**：无本地图形环境
- **Flatpak / Snap**：沙箱化文件系统

### 11.2 检测

```rust
// 示意：环境检测
pub enum ExecutionEnvironment {
    Native,        // 原生桌面环境
    Docker,        // 容器内
    CI,            // CI 系统
    SSH,           // SSH 远程会话
    Sandboxed,     // Flatpak/Snap
    Wasm,          // 浏览器
}
```

检测方式：环境变量（`CI`、`CONTAINER`、`SSH_CONNECTION`）、文件存在性（`/.dockerenv`）、`cgroup` 检查。

### 11.3 兼容策略

| 环境 | 渲染 | 音频 | 剪贴板 |
|------|------|------|--------|
| Native | wgpu GPU | 系统音频 | 系统剪贴板 |
| Docker | 软渲染（CPU） | 无 / ALSA null | 无 |
| CI | 无渲染（headless） | 无 | 无 |
| SSH | 无渲染 | 无 | 无 |
| Flatpak | wgpu（ portal 委派） | PipeWire | portal 剪贴板 |

---

## 12. Node 运行时管理（crates/node_runtime）

### 12.1 动机

Warp 的某些功能（AI Agent 的部分工具、LSP 辅助进程）需要 Node.js 运行时。为保证开箱即用，Warp 自动安装/管理 Node.js。

### 12.2 接口

```rust
// 示意：NodeRuntime API
pub struct NodeRuntime;

impl NodeRuntime {
    pub fn ensure_installed() -> Result<NodePath>;
    pub fn version() -> Result<SemVer>;
    pub fn npm_install(package: &str, version: &str) -> Result<()>;
    pub fn run_script(script: &Path) -> Result<ChildHandle>;
}
```

### 12.3 多平台多架构

| 平台 | 架构 | 获取方式 |
|------|------|----------|
| macOS | x64, arm64 | 预编译二进制下载 |
| Linux | x64, arm64 | 预编译二进制下载 |
| Windows | x64, arm64 | 预编译二进制下载 |
| WASM | — | 不可用（跳过） |

Node.js 二进制缓存到 `~/.local/share/warp/node/`（macOS/Linux）或 `%LOCALAPPDATA%/warp/node/`（Windows）。

---

## 13. 关键设计模式

### 13.1 `#[cfg]` 平台隔离

```rust
// 不同平台的相同接口，编译时选择
#[cfg(target_os = "macos")]
mod platform_impl;

#[cfg(target_os = "linux")]
mod platform_impl;

#[cfg(target_os = "windows")]
mod platform_impl;
```

原则：产品代码中尽量少用 `#[cfg]`，将平台差异封装到 `platform_impl` 模块或 crate 的 trait 背后。

### 13.2 Trait 抽象统一接口

每个系统级服务定义一个 trait，各平台提供实现。消费方通过 trait 方法调用，不感知平台细节：

```
PlatformDelegate (trait)
  ├── MacPlatformDelegate
  ├── LinuxPlatformDelegate
  ├── WindowsPlatformDelegate
  └── WasmPlatformDelegate

SecretStore (trait)
  ├── MacKeychain
  ├── WindowsDPAPI
  └── LinuxSecretService

VirtualFs (trait)
  ├── RealFs
  └── MockFs
```

### 13.3 平台特性差异局部化

将平台差异限定在最少量的文件中：

- 平台特定 crate（`managed_secrets`、`prevent_sleep`）内部按 `#[cfg]` 分文件
- 主二进制 `app/` 的 `platform/` 目录仅存放调用平台 trait 的胶水代码
- 产品代码不直接 import 平台特定符号，统一通过 trait 或 `PlatformDelegate` 句柄访问

### 13.4 回退链设计

对于有多个可选平台后端的服务（如秘密存储、字体枚举），使用回退链：

```
第一优先 → 第二优先 → ... → 兜底实现（纯 Rust 实现）
```

确保在任意平台上都能正常工作，而非仅在"完美"环境中。

---

## 14. 总结

Warp 的跨平台策略遵循以下核心原则：

1. **trait 抽象在前，平台实现在后**：在 `warpui_core` 定义接口，平台 crate 提供具体实现
2. **最小化 `#[cfg]` 渗透**：平台差异封装在模块内部，不暴露给消费方
3. **各平台原生集成**：macOS 用 Cocoa，Linux 用 DBus/winit，Windows 用 WinRT/Win32，WASM 用 Web API
4. **系统服务 RAII 化**：休眠抑制、密钥链访问等系统资源使用 RAII 模式管理生命周期
5. **回退链保障鲁棒性**：每个系统服务都有从原生 API 到纯 Rust 实现的完整回退路径
