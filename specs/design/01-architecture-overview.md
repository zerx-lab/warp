# Warp 全局架构设计

> 本文档描述 Warp 项目的整体架构设计、分层体系、核心抽象和跨模块协作机制。

## 1. 项目定位

Warp 是一个以 Rust 为主的 **Agentic 终端 / 开发环境**，在自研 UI 框架（WarpUI）之上集成了：

- 终端模拟（PTY、ANSI/VT 解析、Grid 渲染、Shell 集成）
- AI Agent（多模型支持、MCP 工具调用、上下文管理）
- 云同步（Drive：对象级跨设备同步）
- 代码评审（Git Diff 查看、内联评论）
- 代码补全（Shell 命令补全引擎）
- Notebook / Workflow（可同步的富文本文档和自动化流程）
- 设置系统（TOML 持久化、热重载、云同步）

## 2. 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│                   app/ (主二进制)                              │
│  UI 视图根 | 平台粘合 | 持久化迁移 | 产品功能装配               │
│  ┌─────────┬──────────┬────────┬─────────┬──────────────┐  │
│  │Terminal │    AI    │  Code  │  Drive  │   Settings   │  │
│  │  View   │  Agent   │ Review │  Panel  │    Views     │  │
│  ├─────────┼──────────┼────────┼─────────┼──────────────┤  │
│  │ PaneGroup / Tab / Workspace / Menu / Modal 等 UI 容器 │  │
│  └─────────┴──────────┴────────┴─────────┴──────────────┘  │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────┼──────────────────────────────────────┐
│        产品域 Crate  │                                       │
│  ┌────────┬──────────┼──────┬────────┬──────────────┐      │
│  │crates/ │ crates/  │crates│crates/ │ crates/      │      │
│  │  ai    │computer_ │ /lsp │ comple │ languages    │      │
│  │        │   use    │      │  ter   │              │      │
│  ├────────┴──────────┴──────┴────────┴──────────────┤      │
│  │ crates/onboarding / crates/vim / crates/command  │      │
│  └───────────────────────────────────────────────────┘      │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────┼──────────────────────────────────────┐
│       框架 Crate     │                                       │
│  ┌────────┬──────────┼──────┬────────┬─────────────┐       │
│  │warpui_ │ warpui   │warpui│editor  │ sum_tree    │       │
│  │  core  │          │extras│(warp_  │             │       │
│  │        │          │      │editor) │             │       │
│  ├────────┴──────────┴──────┴────────┴─────────────┤       │
│  │        ui_components / syntax_tree              │       │
│  └─────────────────────────────────────────────────┘       │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────┼──────────────────────────────────────┐
│     基础设施 Crate   │                                       │
│  ┌────────┬──────────┼──────┬────────┬─────────────┐       │
│  │warp_   │ warp_    │http_ │persist │ websocket   │       │
│  │  core  │  util    │client│ ence   │             │       │
│  ├────────┴──────────┴──────┴────────┴─────────────┤       │
│  │  virtual_fs / watcher / asset_cache / ipc /     │       │
│  │  jsonrpc / managed_secrets / repo_metadata      │       │
│  └─────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

### 2.1 四层职责

| 层 | 包含 | 职责 |
|----|------|------|
| **app/** | 主二进制 + 60+ 域目录 | 装配所有子系统、UI 视图、平台粘合、数据库迁移 |
| **产品域 Crate** | ai, computer_use, lsp, completer, languages, vim... | 面向产品的功能逻辑 |
| **框架 Crate** | warpui_core, warpui, editor, sum_tree, syntax_tree, ui_components | 跨功能复用的 UI 与数据结构框架 |
| **基础设施 Crate** | warp_core, persistence, http_client, websocket... | 通用工具与平台抽象 |

### 2.2 关键架构约束

- **无跨层倒挂**：下层不能依赖上层。框架层不依赖产品域。
- **Entity-Handle 系统**：跨层通信通过类型化句柄（`ViewHandle<T>` / `ModelHandle<T>`），而非直接引用。
- **Feature Flag 优先**：运行时 `FeatureFlag::is_enabled()` 优先于 `#[cfg]`，仅在编译必需时使用条件编译。

## 3. Entity-Handle 架构

### 3.1 核心概念

WarpUI 的核心抽象围绕两个实体类型：

```
Entity (trait: 'static + Event type)
 ├── Model (纯数据，存储在 AppContext 的 HashMap<EntityId, Box<dyn AnyModel>>)
 └── View (可交互 UI 组件，存储在 Window 的 HashMap<EntityId, Box<dyn AnyView>>)
      └── TypedActionView (声明处理特定 Action 类型)
```

访问模式：

```
┌──────────────┐     handle.read(&app, |model, ctx| → &T)
│  ViewHandle  │──── handle.update(&mut app, |model, ctx| → &mut T)
│  或          │──── handle.as_ref(&app)  → &T (临时引用)
│  ModelHandle │
└──────────────┘
      │
      ├── PhantomData<T> (类型安全)
      ├── EntityId (全局唯一 AtomicUsize)
      └── Arc<Mutex<RefCounts>> (引用计数，自动清理)
```

### 3.2 单例模型

`SingletonEntity` trait 标记全局唯一的 Model，通过 `TypeId` 存储在 `FxHashMap<TypeId, AnyModelHandle>` 中：

```rust
pub trait SingletonEntity: Entity + Sized {
    fn handle<T: GetSingletonModelHandle>(ctx: &T) -> ModelHandle<Self>;
    fn as_ref(ctx: &crate::AppContext) -> &Self;
}
```

内置单例包括：字体缓存、窗口管理器、资源缓存、图像缓存。

### 3.3 窗口管理

`AppContext` 持有 `HashMap<WindowId, Window>`，每个 Window 包含：

- 根视图（`AnyViewHandle`）
- 该窗口的所有视图（`HashMap<EntityId, Box<dyn AnyView>>`）
- `view_to_window` 映射（支持视图在窗口间转移）

## 4. Element 树系统

### 4.1 声明式 UI

View 的 `render()` 方法返回一个 Element 树（类似 React 组件产生 Virtual DOM）：

```
View.render(&self, app: &AppContext) → Box<dyn Element>
```

### 4.2 Element 生命周期

```
render() 产生 Element 树
    │
    ▼
Layout ←── SizeConstraint (min/max) 向下传递，尺寸向上返回
    │
    ▼
after_layout ←── 读取子元素布局结果
    │
    ▼
Paint ←── 将 Element 树转换为 Scene（层的平面列表 + R-Tree 命中测试）
    │
    ▼
dispatch_event ←── 命中测试 → Action → 响应者链向上传播
```

### 4.3 内置 Element 类别

| 类别 | Element |
|------|---------|
| 布局 | Flex(行/列), Container(填充/边距/背景/边框), Align, ConstrainedBox, Padding, MinSize |
| 堆叠 | Stack(子元素层叠和定位) |
| 滚动 | Scrollable, ClippedScrollable, List, UniformList, ViewportedList |
| 文本 | Text, FormattedTextElement |
| 图形 | Rect(填充/渐变/描边/圆角), Image, Icon |
| 交互 | Hoverable, EventHandler, SelectableArea, DragResize, Dismiss |
| 容器 | ChildView(嵌入另一个 View), Empty, Clipped |
| 表格 | Table(标题行、列宽、排序状态) |

### 4.4 响应者链

Action 事件沿视图祖先链向上传播，每个级别可以选择处理并停止传播：

```
Child View → Parent View → ... → Root View
```

## 5. 状态管理与数据流

### 5.1 不可变/可变分离

App 通过 `Rc<RefCell<AppContext>>` 提供安全访问：

```rust
app.read(|ctx| { /* &AppContext */ });    // 只读
app.update(|ctx| { /* &mut AppContext */ });  // 可变，退出时自动 flush_effects()
```

### 5.2 订阅与观察

| 机制 | 用途 |
|------|------|
| `subscribe_to_model()` | 接收实体发出的带数据事件 |
| `observe()` | 监听无效化（模型已修改需重绘），不接收事件数据 |

### 5.3 完整数据流

```
用户输入 → Event → Element.dispatch_event()
    → EventContext.dispatch_action(name, arg)
    → AppContext 沿响应者链查找处理程序
    → 调用 View 的 handle_action → 更新 Model
    → Model.notify() → 观察者触发重绘
    → Presenter.invalidate() → 重新 render() → 布局 → 绘制
```

## 6. 平台抽象层

定义在 `warpui_core/src/platform/`，实现在 `warpui/src/platform/{mac,linux,windows,wasm}/`：

| 抽象 | 用途 |
|------|------|
| `platform::Delegate` | 应用生命周期（打开 URL、休眠、终止） |
| `platform::Window` | 原生窗口句柄、尺寸、全屏、IME |
| `platform::WindowManager` | 窗口创建和管理 |
| `platform::FontDB` | 字体枚举和加载 |
| `platform::Menu` | 原生菜单栏 |
| `platform::Cursor` | 光标形状 |
| `platform::Clipboard` | 系统剪贴板 |

渲染后端：
- **macOS**: 原生 Metal
- **Windows**: wgpu(DirectX) + DWrite
- **Linux**: wgpu(Vulkan) + fontconfig
- **WASM**: WebGL/Canvas

## 7. 并发模型

```
┌──────────────────────┐     ┌──────────────────────┐
│    UI 线程 (主线程)   │     │  PTY Event Loop 线程  │
│                      │     │                      │
│  WarpUI Presenter    │     │  mio 事件循环         │
│  Element 树管理      │     │  PTY 读写             │
│  Action 分发         │     │  ANSI 解析            │
│                      │     │  TerminalModel 写入   │
└────────┬─────────────┘     └──────────┬───────────┘
         │                              │
         │    Arc<FairMutex<             │
         │     TerminalModel>>           │
         └──────────────────────────────┘
```

- PTY 事件循环是单线程的，通过 `mio` 驱动
- `TerminalModel` 通过 `Arc<FairMutex<T>>` 在 UI 和 PTY 线程间共享
- `FairMutex` 防止写入者饥饿
- SQLite 使用单写入器 + 多读取器模式（SyncSender channel）

## 8. 关键设计模式

| 模式 | Warp 中的实现 |
|------|---------------|
| **Entity-Handle** | 引用计数句柄，类型安全，自动清理 |
| **声明式 UI** | View.render() → Element 树，与 Flutter/React 类似 |
| **Flutter 式布局** | 向下传约束，向上返尺寸；Flex、Container、Stack |
| **单向数据流** | 事件 → Action → Model 更新 → 通知 → 重绘 |
| **响应者链** | Action 沿视图祖先链向上传播 |
| **效果队列** | 可变状态变更排队，作用域结束时批量刷新 |
| **基于层的绘制** | Scene 中的层叠层，R-Tree 命中测试 |
| **SumTree** | 持久化平衡 B 树，O(log n) 聚合查询 |
| **Handler Trait** | ANSI 解析通过虚方法分派与模型状态分离 |
| **写时复制** | FlatStorage 的间隔映射样式存储 |

## 9. Cargo 工作区配置

- `resolver = "2"`
- `default-members` 收敛到常用编译子集
- `serve-wasm` 与 `integration` 不在 default-members 中
- 依赖集中在 `[workspace.dependencies]`

## 10. 许可证

- `crates/warpui` + `crates/warpui_core` → **MIT**
- 其余 → **AGPL-3.0-only**
