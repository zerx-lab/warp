# WarpUI 框架设计文档

> 本文档描述 WarpUI——Warp 终端自研声明式 UI 框架的整体架构、核心抽象与关键实现模式。

---

## 1. 架构总览

WarpUI 是一个 **自研声明式 UI 框架**，受 Flutter 启发但为 Rust CLI/桌面环境量身定制。它分为三个层次：

```
┌─────────────────────────────────────────────────────┐
│                    ui_components                     │
│        (Button / Switch / Dialog / Tooltip 等)       │
├─────────────────────────────────────────────────────┤
│              warpui_extras (可选扩展件)               │
│           ┌──────────────────┐                       │
│           │     warpui       │  Element 树、布局、     │
│           │ 渲染管线、Scene    │  绘制后端抽象            │
│           └──────┬───────────┘                       │
│                  │ 依赖                               │
│           ┌──────┴───────────┐                       │
│           │  warpui_core     │  App / Entity /        │
│           │  ViewHandle      │  AppContext 等基础    │
│           └──────────────────┘                       │
├─────────────────────────────────────────────────────┤
│  warpui_core ⊗ warpui → MIT 许可证                     │
│  其余 → AGPL-3.0                                      │
└─────────────────────────────────────────────────────┘
```

### 1.1 warpui_core（核心）

**许可证：MIT**

功能：
- `App`——应用根对象，持有所有 Entity 的全局存储
- `Entity`——所有 view/model 实体的基抽象
- `ViewHandle<T>` / `ModelHandle<T>`——类型安全的引用句柄
- `AppContext`——框架在视图/模型中注入的上下文
- `EntityId`——全局唯一标识符
- `SingletonEntity`——单例 entity 特化

### 1.2 warpui（渲染管线）

**许可证：MIT**

功能：
- 声明式 **Element 树** 系统——48+ 内置 Element 类型
- 布局引擎——`Layout` / `after_layout` 两阶段布局
- 绘制管线——`Scene` -> R-Tree -> 原语平面 -> 后端 API
- 事件分发——`dispatch_event` 沿 Element 树传播
- 键盘映射系统——`Keystroke` + `Context` + `Matcher` + `Trigger`

### 1.3 warpui_extras（可选）

**许可证：MIT**

功能：
- warpui 的可选扩展组件集
- 默认不启用全部 features，按需引入

### 1.4 ui_components（跨视图高层组件）

**许可证：AGPL-3.0**

功能：
- `Component` trait——持久状态 + Params + Options 三件套
- 具体组件：Button、Switch、Dialog、Tooltip、输入框、列表、模态框等

### 1.5 依赖关系

```
ui_components
    ↓ 依赖
warpui_extras (可选)
    ↓ 依赖
warpui
    ↓ 依赖
warpui_core
    ↓ 依赖
warp_util, warp_core 等基础设施 crate
```

---

## 2. Entity-Handle 架构

Entity-Handle 是 WarpUI 最核心的架构模式：**所有 view/model 实体统一由 `App` 全局拥有**，View 之间通过类型安全句柄引用，而不是直接拥有对方。

### 2.1 核心概念

```
┌────────────────────────────────────────────────────────────┐
│                          App                                │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Entity 存储 (SlotMap 或等价结构)                    │    │
│  │                                                     │    │
│  │  EntityId(1) → TerminalModel                       │    │
│  │  EntityId(2) → EditorModel                         │    │
│  │  EntityId(3) → TerminalView                        │    │
│  │  EntityId(4) → EditorView                          │    │
│  │  ...                                                │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
│  外部通过句柄访问:                                          │
│  ┌──────────────┐   ┌──────────────┐                       │
│  │ ViewHandle<T> │──→│  EntityId    │  (PhantomData + Arc)  │
│  │ ModelHandle<T>│──→│  + 类型标记   │                       │
│  └──────────────┘   └──────────────┘                       │
└────────────────────────────────────────────────────────────┘
```

### 2.2 EntityId

```rust
// 简化示意
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(usize);

impl EntityId {
    pub fn new() -> Self {
        // 使用 AtomicUsize 递增生成全局唯一 ID
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        EntityId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
```

- 全局唯一，原子递增生成
- `Copy + Clone + PartialEq + Eq + Hash`——轻量可复制
- 本身不携带类型信息（运行时类型擦除）

### 2.3 ViewHandle<T> / ModelHandle<T>

```rust
// 简化示意
pub struct ViewHandle<T: 'static> {
    entity_id: EntityId,
    _phantom: PhantomData<*const T>,
    ref_counts: Arc<RefCounts>,
}

pub struct ModelHandle<T: 'static> {
    entity_id: EntityId,
    _phantom: PhantomData<*const T>,
    ref_counts: Arc<RefCounts>,
}
```

**关键设计：**

| 特性 | 说明 |
|------|------|
| `PhantomData<*const T>` | 所有权标记，不实际持有 T |
| `Arc<RefCounts>` | 共享引用计数，追踪实体是否仍然存活 |
| `EntityId` | 指向 App 中实体的键 |
| `Send + Sync` | 句柄本身可跨线程传递 |

**访问模式：**

```rust
// 读访问 —— 不可变引用，可在任意上下文中调用
let model: ReadRef<'_, MyModel> = model_handle.read(cx);

// 可变访问 —— 需在 update 上下文中调用
model_handle.update(cx, |model, cx| {
    model.counter += 1;
});

// 降级为通用 EntityId
let id: EntityId = handle.entity_id();
```

- `handle.read(cx)` —— 返回 `ReadRef<'_, T>`，不可变借用，可在任意上下文中使用
- `handle.update(cx, |model, cx| { ... })` —— 可变借用，只能在 App::update 作用域内调用
- `handle.as_ref()` —— 返回 `&EntityId`

### 2.4 生命周期与安全性

```
实体创建:
  let handle: ViewHandle<MyView> = cx.add_view(MyView::new());
  let model_handle: ModelHandle<MyModel> = cx.add_model(MyModel::new());

实体删除:
  // View 销毁时自动触发
  // Model 通过 cx.remove_model(handle) 显式移除

防悬垂引用:
  尝试 read/update 已销毁实体 → 返回 None 或 panic（调试模式）
```

### 2.5 SingletonEntity

单例实体——在整个应用中只存在一份的实体（如 `Workspace`、`Appearance`）。

```rust
// 简化示意
pub struct SingletonEntity<T: 'static> {
    entity_id: EntityId,
    _phantom: PhantomData<*const T>,
}
```

- 在 `App::new()` 阶段注册
- 通过 `app.singleton::<T>()` 全局访问
- 不可销毁（应用存活期间始终存在）

### 2.6 窗口管理

每个 OS 窗口对应一组 Entity：

```
Window 1
  ├── Workspace (Singleton)
  │   ├── PaneGroup
  │   ├── TerminalView (→ TerminalModel)
  │   └── EditorView (→ EditorModel)
  └── ...

Window 2
  ├── Workspace (Singleton)
  │   ├── PaneGroup
  │   ├── TerminalView (→ TerminalModel)
  │   └── ...
  └── ...
```

多窗口协调通过 `voltron`（多窗口/多 workspace 协调模块）实现。

---

## 3. View 系统

View 是 WarpUI 中**负责渲染和交互的实体**。每个 View 对应 UI 树中的一个节点。

### 3.1 View Trait

```rust
// 简化示意
pub trait View: 'static {
    type State: Default;

    fn ui_name(&self) -> Option<&str> { None }

    fn render(&mut self, state: &mut Self::State, cx: &mut ViewContext<Self>) -> ImplAnyElement;

    fn keymap_context(&self) -> Option<Vec<Context>> { None }

    // 生命周期回调
    fn on_focus_in(&mut self, state: &mut Self::State, cx: &mut ViewContext<Self>) {}
    fn on_focus_out(&mut self, state: &mut Self::State, cx: &mut ViewContext<Self>) {}
    fn on_frame(&mut self, state: &mut Self::State, cx: &mut ViewContext<Self>) {}
    fn on_finish_quit(&mut self, state: &mut Self::State, cx: &mut ViewContext<Self>) {}
}
```

| 方法 | 说明 |
|------|------|
| `render()` | **核心方法**——返回当前帧的 Element 树 |
| `ui_name()` | 调试/工具用的人类可读名称 |
| `keymap_context()` | 该 View 活跃时的键盘映射上下文 |
| `on_focus_in/out` | 焦点进入/离开时回调 |
| `on_frame` | 每帧回调（动画/定时任务） |
| `on_finish_quit` | 应用退出前回调 |

### 3.2 TypedActionView

类型安全 Action 分派——View 的增强 trait。

```rust
// 简化示意
pub trait TypedActionView: View {
    type Action: 'static;

    fn dispatch_action(
        &mut self,
        state: &mut Self::State,
        cx: &mut ViewContext<Self>,
        action: &Self::Action,
    );
}
```

- `Action` 是关联类型，编译期强制类型检查
- 分派时无需字符串匹配，直接匹配枚举变体
- View 注册自己关心的 Action 类型

### 3.3 ViewContext

ViewContext 是框架注入给 **View 生命周期方法**的上下文对象，提供：

```rust
// 简化示意
pub struct ViewContext<'a, V: View + 'static> {
    app: &'a mut App,
    view_handle: ViewHandle<V>,
    entity_id: EntityId,
    // 内部状态
}
```

**核心能力：**

| 方法 | 说明 |
|------|------|
| `cx.add_model(model)` | 创建新 Model 实体，返回 `ModelHandle<T>` |
| `cx.add_view(view)` | 创建子 View 实体，返回 `ViewHandle<T>` |
| `cx.focus(handle)` | 将焦点设到指定 View |
| `cx.focused()` | 获取当前焦点 View 的句柄 |
| `cx.subscribe(handle, listener)` | 订阅 Model 变化通知 |
| `cx.spawn(future)` | 在异步运行时中启动任务 |
| `cx.notify(handle)` | 通知框架某 Model 已变化，触发重渲染 |
| `cx.remove_model(handle)` | 移除 Model 实体 |
| `cx.window_bounds()` | 获取窗口边界 |
| `cx.on_next_frame(callback)` | 注册下一帧回调 |

---

## 4. Element 树系统

Element 是 WarpUI 的**声明式 UI 描述单元**，使用 Builder 模式构造。

### 4.1 声明式 Builder 模式

```rust
// 典型用法
Element::new()
    .flex(1)
    .flex_direction(FlexDirection::Column)
    .children(
        vec![
            Element::new()
                .text("Hello")
                .font_size(16.0)
                .into(),
            Element::new()
                .button()
                .on_click(cx.listener(|_, _, _| println!("Clicked")))
                .child(Element::new().text("Click me").into())
                .into(),
        ]
    )
```

- 每个方法返回 `impl ElementBuilder` 或 `Element`
- 链式调用构造完整的 UI 树
- 编译期检查：错误属性组合在编译时捕获

### 4.2 Element Trait

```rust
// 简化示意
pub trait Element: 'static {
    fn layout(&mut self, constraint: &SizeConstraint, ctx: &mut LayoutContext) -> Size;

    fn after_layout(&mut self, ctx: &mut AfterLayoutContext);

    fn paint(&mut self, scene: &mut Scene, ctx: &mut PaintContext);

    fn dispatch_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventDispatchResult;
}
```

| 方法 | 阶段 | 说明 |
|------|------|------|
| `layout()` | 布局 | 接收父约束，返回自身尺寸 |
| `after_layout()` | 布局后 | 子节点布局完成后回调（如滚动容器调整滚动范围） |
| `paint()` | 绘制 | 将绘制指令写入 Scene |
| `dispatch_event()` | 事件分发 | 处理输入/鼠标事件，返回处理结果 |

### 4.3 Element 类型一览

WarpUI 内置 **48+** Element 类型，按功能分组：

| 类别 | Element 类型 | 说明 |
|------|-------------|------|
| **布局** | `Flex`, `Column`, `Row`, `Stack`, `Expanded`, `Flexible`, `Spacer`, `Divider`, `ConstrainedBox`, `SizedBox`, `Padding`, `Center`, `Align`, `AspectRatio`, `FractionallySized` | 基础布局 |
| **滚动** | `Scroll`, `List`, `VerticalList`, `HorizontalList`, `VirtualList` | 滚动与虚拟化 |
| **文本** | `Text`, `Paragraph`, `Label`, `CodeBlock`, `Markdown` | 文本渲染 |
| **输入** | `TextInput`, `TextArea`, `SearchInput`, `PasswordInput` | 文本输入 |
| **图形** | `Image`, `Svg`, `Canvas`, `Shader`, `Gradient`, `Border`, `Shadow`, `RoundedRect` | 绘制原语 |
| **交互** | `Button`, `Clickable`, `Hoverable`, `Draggable`, `GestureDetector` | 交互响应 |
| **容器** | `Container`, `Clip`, `ClipRRect`, `Opacity`, `Transform`, `RepaintBoundary`, `Offstage` | 容器/变换 |
| **装饰** | `DecoratedBox`, `Background`, `Foreground`, `Overlay`, `Tooltip` | 装饰 |
| **特殊** | `Scrollbar`, `Selection`, `Cursor`, `FocusScope`, `KeyboardListener`, `HitTestBlocker` | 系统功能 |

### 4.4 树的组合

```
render() 返回的 Element 树:
  Column
    ├── Flex(1) → Scroll
    │   └── List
    │       ├── Text("item 1")
    │       ├── Text("item 2")
    │       └── ...
    ├── Divider
    └── Row
        ├── Button("Cancel")
        └── Button("Save")
```

---

## 5. Element 生命周期

每一帧的 Element 树经历 **四个阶段**：

```
┌──────────────────────────────────────────────────────────────────┐
│                        每一帧                                     │
│                                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────────┐    ┌─────────┐ │
│  │ render() │───→│  Layout  │───→│ after_layout │───→│  Paint  │ │
│  │          │    │          │    │              │    │         │ │
│  │ View 重  │    │ 测量+排布 │    │ 子节点布局后   │    │ 写 Scene │ │
│  │ 建树     │    │ SizeConstraints│ 调整          │    │ R-Tree  │ │
│  └──────────┘    └──────────┘    └──────────────┘    └────┬────┘ │
│                                                            │     │
│                                                   dispatch_event │
│                                                   (事件驱动，非帧) │
│                                                            │     │
└────────────────────────────────────────────────────────────────────┘
```

### 5.1 Stage 1: render()

```
触发条件：
  - Model 变化（通过 cx.notify(model_handle)）
  - View 自身状态变化
  - 外部事件（窗口 resize、定时器等）

过程：
  View::render() 被调用 → 生成新的 Element 树
  ↓
  框架将新树与旧树进行 diff（按 Element 类型+key 匹配）
  ↓
  对匹配节点复用 Element 实例（状态保持）
  ↓
  对新增/删除节点执行挂载/卸载回调
```

### 5.2 Stage 2: Layout

```
输入：SizeConstraint (min_width, max_width, min_height, max_height)

过程（深度优先，自顶向下）：
  父节点接收约束
    ↓
  父节点向子节点传递调整后的约束
    ↓
  子节点计算自己的尺寸（产生 Size）
    ↓
  父节点根据子节点尺寸确定子节点位置
    ↓
  父节点返回自己的总 Size

SizeConstraint:
  - loose(min, max)——子节点在范围内自由选择
  - tight(size)——强制子节点等于指定尺寸
  - unbounded()——子节点大小无约束（仅用于内部计算）
```

### 5.3 Stage 3: after_layout

```
时机：所有节点 layout() 完成后

用途：
  - Scroll 容器计算可滚动范围
  - VirtualList 计算可见窗口
  - 需要知道子节点最终位置的布局后处理

特点：
  - 可读不可写（不能改变尺寸/位置）
  - 深度优先遍历
```

### 5.4 Stage 4: Paint

```
输入：Scene 对象

过程（自顶向下）：
  每个 Element 的 paint() 将绘制指令追加到 Scene
  ↓
  原语类型：
    - FillRect(x, y, w, h, color)
    - StrokeRect(x, y, w, h, color, width)
    - DrawText(x, y, text, font, size, color)
    - DrawImage(x, y, w, h, image)
    - ClipRect(x, y, w, h)
    - PushLayer(alpha, mask)
    - PopLayer
  ↓
  Scene 包含：
    - Layer 栈（透明度/裁剪层）
    - R-Tree（空间索引，用于鼠标命中检测）
    - 原语平面列表（按 z-order 排列）
```

### 5.5 事件分发（dispatch_event）

```
触发：鼠标点击/移动/键盘输入

过程：
  App 从根 Element 树开始分发
    ↓
  沿 R-Tree 做空间命中测试（鼠标事件）
    ↓
  命中路径上 dispatch_event 依次调用
    ↓
  任何节点返回 EventDispatchResult::Consumed → 传播停止
    ↓
  未被消费的事件→ 回到 App 级默认处理

EventDispatchResult:
  - Consumed——事件已处理，停止传播
  - Propagate——继续传播给父节点
```

---

## 6. Action 事件系统

Action 是 WarpUI 的**高级事件抽象**——将输入（键盘/菜单/命令）映射到特定语义操作。

### 6.1 传统 Action vs TypedAction

```
传统 Action（字符串键控）:
  "close_tab" → 查找响应者 → 执行

TypedAction（类型安全）:
  CloseTab → 编译期类型匹配 → 分派给 View
```

| 特性 | 传统 Action | TypedAction |
|------|------------|-------------|
| 标识方式 | 字符串常量 | Rust 类型 |
| 类型安全 | 否（运行时拼写错误） | 是（编译期检查） |
| 调试友好 | 否 | 是 |
| 泛用性 | 全局可分发 | 需 View 声明支持 |

### 6.2 响应者链

```
触发 Action
    ↓
焦点 View → 检查是否处理该 Action
  ├── 是 → 调用 dispatch_action / handle_action
  └── 否 → 传递给父 View（PaneGroup）
            ├── 是 → 调用
            └── 否 → 传递给 Workspace
                      ├── 是 → 调用
                      └── 否 → App 级默认处理
                                ├── StandardAction::Quit → 退出
                                ├── StandardAction::Hide → 隐藏窗口
                                └── ...
```

### 6.3 全局 Action

跨 View 边界的 Action（如菜单项、全局快捷键）：

```rust
// 简化示意
pub struct GlobalAction {
    pub action: ActionId,        // 字符串标识
    pub source: ActionSource,    // Menu | KeyEquivalent | CommandPalette
    pub payload: Option<Box<dyn Any>>,
}

impl App {
    pub fn dispatch_global_action(&mut self, action: GlobalAction) {
        // 1. 先尝试焦点 View
        // 2. 再尝试 Workspace
        // 3. 最后 App 级默认
    }
}
```

### 6.4 StandardAction

操作系统菜单映射为 StandardAction：

| Action | 触发时机 | 默认行为 |
|--------|---------|----------|
| `StandardAction::Close` | Cmd+W / 关闭按钮 | 关闭当前 tab |
| `StandardAction::Quit` | Cmd+Q | 退出应用 |
| `StandardAction::Hide` | Cmd+H | 隐藏窗口 |
| `StandardAction::Minimize` | Cmd+M | 最小化窗口 |
| `StandardAction::Zoom` | 绿色按钮 | 窗口缩放 |
| `StandardAction::EnterFullScreen` | Cmd+Ctrl+F | 全屏 |
| `StandardAction::Cut` | Cmd+X | 剪切 |
| `StandardAction::Copy` | Cmd+C | 复制 |
| `StandardAction::Paste` | Cmd+V | 粘贴 |
| `StandardAction::SelectAll` | Cmd+A | 全选 |
| `StandardAction::Undo` | Cmd+Z | 撤销 |
| `StandardAction::Redo` | Cmd+Shift+Z | 重做 |

---

## 7. 状态管理

### 7.1 read / update 分离

WarpUI 的状态访问严格分离为两种上下文：

```
read（不可变访问）
  ─ 可在任意上下文中调用（render、paint、layout、事件处理）
  ─ 返回 ReadRef<'_, T>（借用检查器保证的生命周期安全借用）
  ─ 多个 read 可并发

update（可变访问）
  ─ 只能在 App::update 或 cx.update 回调中调用
  ─ 借出 &mut T
  ─ 同一时刻只能有一个 update（内部借用检查）
  ─ 调用 notify 触发重渲染
```

### 7.2 效果队列（flush_effects）

```
Model::update() 内：
  model.field = new_value;
  cx.notify();  // 标记状态变更
                  ↓
延迟到 update 回调结束 → flush_effects：
  ├── 遍历所有标记为 dirty 的 View
  ├── 对每个 dirty View 调用 render() 重建 Element 树
  ├── 执行 diff → layout → after_layout → paint
  └── 渲染到屏幕
```

- 同步效果队列——同一帧内完成所有重绘
- 批量合并——同一 Model 在 update 期间多次 notify 只触发一次重渲染

### 7.3 订阅 vs 观察

```
订阅 (subscribe):
  cx.subscribe(model_handle, |view, model, cx| {
      // 当 model 变化时，View 收到通知
      // View 可选择是否重渲染
  });
  ─ 一对一关系：一个 View 订阅一个 Model
  ─ 自动清理：View 销毁时自动取消订阅

观察 (observe):
  cx.observe(model_handle, |model, cx| {
      // 通用回调，不绑定特定 View
  });
  ─ 一对多关系：多个观察者监听同一 Model
  ─ 手动清理
```

### 7.4 状态生命周期

```
Model 状态：
  创建：cx.add_model(MyModel::new()) → 返回 ModelHandle<T>
  使用：handle.read(cx) / handle.update(cx, |m, cx| { ... })
  销毁：cx.remove_model(handle) → 释放内存 + 取消所有订阅

View 状态（View::State）：
  创建：View 首次 render() 前调用 State::default()
  保持：View 在 Element 树中存活期间 State 一直存在
  重置：View 被移出 Element 树（但 Entity 未销毁）后，重新插入时 State 恢复
  销毁：View Entity 被销毁时释放
```

---

## 8. 渲染管线

### 8.1 Scene 结构

```
Scene
  ├── Layer 栈
  │   ├── Layer { alpha: 0.5, mask: ClipRect(...) }
  │   ├── Layer { alpha: 1.0, mask: None }
  │   └── Layer { alpha: 1.0, mask: ClipRect(...) }
  ├── R-Tree（空间索引，命中测试用）
  │   ├── Node { bounds: Rect, children: [...] }
  │   ├── Node { bounds: Rect, children: [...] }
  │   └── Leaf { bounds: Rect, element_id }
  └── 原语平面列表（有序，从后往前绘制）
      ├── { z: 0, FillRect(0, 0, 800, 600, #1E1E1E) }
      ├── { z: 1, DrawText(20, 20, "Hello", ...) }
      ├── { z: 2, FillRect(100, 100, 200, 40, #333333) }
      └── { z: 3, DrawText(110, 110, "Button", ...) }
```

### 8.2 渲染流水线

```
Element::paint() 写入 Scene
    ↓
Scene 排序（按 z-order / layer 嵌套）
    ↓
R-Tree 重建（更新 hit-test 空间索引）
    ↓
Layer 展开（alpha 融合、裁剪计算）
    ↓
原语平面→平台 API 调用
    ↓
    ├── macOS: Metal → CAMetalLayer
    ├── Linux: wgpu → Vulkan/OpenGL/ANGLE
    ├── Windows: wgpu → DirectX
    └── WASM: WebGL → Canvas
```

### 8.3 缩放/变焦

```
窗口缩放因子（device pixel ratio）:
  - macOS: 从 NSWindow::backingScaleFactor 获取
  - Linux: 从 wl_output::scale / xrdb Xft.dpi 获取
  - Windows: 从 GetDpiForWindow 获取
  - WASM: 从 window.devicePixelRatio 获取

逻辑坐标 → 物理像素：
   物理宽度 = 逻辑宽度 × scale
   物理高度 = 逻辑高度 × scale
```

### 8.4 绘制后端

| 后端 | 目标平台 | 状态 |
|------|---------|------|
| Metal | macOS 原生 | 主力后端 |
| wgpu | Linux / Windows / 通用 | 跨平台 Vulkan/DX12/Metal |
| WebGL | WASM | Web 兼容后端 |

---

## 9. 键盘映射

键盘映射系统将物理按键组合转换为语义 Action。

### 9.1 核心概念

```
Keystroke          → 物理按键组合（Cmd+K, Cmd+T）
Context            → 键盘上下文（"terminal", "editor", "search"）
ContextPredicate   → 上下文匹配条件（View::keymap_context() 返回）
Matcher            → 匹配规则（某键 + 某上下文 → 某 Action）
Trigger            → 匹配结果触发的行为
```

### 9.2 优先级链

```
事件从 app 分发至 View, 匹配规则按以下优先级：
  1. 焦点 View 精确匹配（Keystroke + context = action）
  2. 焦点 View 部分匹配（Keystroke only）
  3. 父 View 匹配（PaneGroup 等）
  4. Workspace 级别匹配
  5. 全局默认匹配
```

### 9.3 Matcher 定义

```rust
// 简化示意
struct KeyBinding {
    keystroke: Keystroke,  // 如 Keystroke::new().cmd().key("K")
    context: Context,       // 如 Context::Terminal
    action: ActionId,       // 如 "terminal:clear"
}

struct KeymapTable {
    bindings: Vec<KeyBinding>,
}
```

### 9.4 运行流程

```
用户按键
    ↓
App 接收按键事件
    ↓
Keystroke 标准化（修饰键归一化、键码映射）
    ↓
获取当前焦点 View 的 keymap_context()
    ↓
在 KeymapTable 中查找匹配的 (Keystroke, Context)
  ├── 匹配成功 → 触发对应 Action
  └── 匹配失败 → 沿焦点链向上查找
```

---

## 10. ui_components

`ui_components` 是跨视图复用的高层组件库，基于 `Component` trait 构建。

### 10.1 Component Trait

```rust
// 简化示意
pub trait Component: 'static + Default {
    type State: Default + 'static;
    type Params: Default + 'static;
    type Options: Default + 'static;

    fn ui(state: &mut Self::State, params: &Self::Params, options: &Self::Options) -> StatefulElement;
}
```

| 关联类型 | 说明 |
|---------|------|
| `State` | 组件内部可变状态（如开关的 on/off） |
| `Params` | 组件参数（如按钮文本、图标） |
| `Options` | 可选配置（如颜色、尺寸变体） |

### 10.2 内置组件

| 组件 | State | Params | Options |
|------|-------|--------|---------|
| `Button` | `()`, `bool(disabled)` | `label: String`, `icon: Option<Icon>`, `on_click: Callback` | `size`, `variant`, `color` |
| `Switch` | `bool(on)` | `label: String`, `on_toggle: Callback<bool>` | `color` |
| `Dialog` | `Option<DialogResult>` | `title`, `content: Element`, `buttons` | `modal: bool` |
| `Tooltip` | `Option<Point>` | `text`, `target: ViewHandle` | `delay`, `position` |
| `Dropdown` | `Option<usize>(selected)` | `items: Vec<String>` | `width` |
| `Checkbox` | `bool(checked)` | `label: String` | `indeterminate` |
| `Slider` | `f64(value)` | `min`, `max`, `step` | `show_label` |
| `ProgressBar` | `f64(progress)` | `()`, | `color`, `size` |
| `TabBar` | `usize(active)` | `tabs: Vec<Tab>` | `style` |
| `TreeList` | `TreeState` | `items: Vec<TreeNode>` | `indent`, `show_lines` |

### 10.3 使用范例

```rust
// Button 在 render() 中的使用
fn render(&mut self, state: &mut Self::State, cx: &mut ViewContext<Self>) -> ImplAnyElement {
    Button::ui(
        Button::state(),
        &Button::params()
            .label("Submit")
            .icon(Icon::Check)
            .on_click(cx.listener(|_, _, _| { /* handle click */ })),
        &Button::options()
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Medium),
    ).into()
}
```

---

## 11. sum_tree——持久化平衡 B 树

`sum_tree` 是 WarpUI 的核心数据结构 crate，为编辑器缓冲区、Notebook 内容、大列表等提供高效持久化数据支持。

### 11.1 核心 trait

`sum_tree` 以三个 trait 为核心：

```
Item trait                 KeyedItem trait                Dimension trait
  ├── initially()           ├── key()                      ├── add(a, b) → D
  ├── measure() → D         └── (Item + 有键支持)          ├── subtract(a, b) → D
  └── (元素本身的值)                                       └── (可加可减的度量)
```

| trait | 说明 | 示例 |
|-------|------|------|
| `Item` | 树中的元素，可测量自身大小 | `char` (测量为宽度) |
| `KeyedItem` | 带键的元素（支持 key 查找） | `Line { key: usize, text: String }` |
| `Dimension` | 可聚合的度量值 | `usize` (行数)、`f32` (宽度)、`Summary` (多字段) |

### 11.2 数据结构

```
B-树节点（每个节点 M 个子节点）:
  ┌─────────────────────────────────┐
  │  Internal Node                    │
  │                                   │
  │  Subtree 1  Subtree 2  Subtree 3  │
  │  [0..100)   [100..250) [250..400) │
  │  sum: 100   sum: 150   sum: 150   │
  │                                   │
  │  Children: [Node, Node, Node]     │
  └─────────────────────────────────┘

每个子树缓存该子树中所有元素的 Dimension 之和（sum）。
通过 sum 实现 O(log n) 的按偏移量查找。
```

### 11.3 Cursor API

```rust
// 简化示意
pub struct Cursor<'a, T: Item> {
    tree: &'a SumTree<T>,
    position: usize,     // 逻辑位置（按 Dimension 度量）
    path: Vec<usize>,    // 树中的物理路径
}

impl<T: Item> Cursor<'_, T> {
    // 导航
    pub fn seek(&mut self, position: &T::Dimension);  // 定位到某度量位置
    pub fn slice(&mut self, len: usize) -> &[T];       // 读取后续元素
    pub fn suffix(&mut self) -> Cursor<'_, T>;          // 当前位置之后的光标副本
    pub fn prev(&mut self) -> Option<&T>;               // 前一个元素
    pub fn next(&mut self) -> Option<&T>;               // 后一个元素

    // 信息
    pub fn start(&self) -> T::Dimension;                // 光标处累计度量
    pub fn end(&self) -> T::Dimension;                  // 光标处 + 当前元素度量
    pub fn item(&self) -> Option<&T>;                   // 当前元素
    pub fn at_end(&self) -> bool;                       // 是否到末尾
}
```

### 11.4 批量编辑

```rust
// 简化示意
impl<T: Item> SumTree<T> {
    pub fn edit<R>(&mut self, cursor: &Cursor<T>, f: impl FnOnce(&mut Editor<'_, T>) -> R) -> R;
}

pub struct Editor<'a, T: Item> {
    tree: &'a mut SumTree<T>,
    position: usize,
    // 内部操作日志
}

impl<T: Item> Editor<'_, T> {
    pub fn insert(&mut self, item: T);
    pub fn remove(&mut self);
    pub fn split(&mut self) -> Cursor<'_, T>;
}
```

- `edit()` 方法接收一个回调，回调内可执行插入、删除、拆分操作
- 编辑器在回调结束后一次性合并变更（批量重平衡）
- 支持嵌套编辑

### 11.5 使用场景

| 场景 | Item | Dimension | 说明 |
|------|------|-----------|------|
| 编辑器缓冲区 | 行 `Line { text, ... }` | `usize` (行数) | 按行号快速跳转 |
| 宽字符文本 | 字符 | `f32` (x偏移) | 按像素偏移量定位光标 |
| 大列表 | 列表项 | `usize` (项数) | 虚拟滚动 |

---

## 12. 编辑器 Core

编辑器 core 是建立在 sum_tree 之上的文本编辑基石。

### 12.1 CoreEditorModel Trait

```rust
// 简化示意
pub trait CoreEditorModel {
    type Content: Item + Default;

    fn buffer(&self) -> &Buffer<Self::Content>;
    fn buffer_mut(&mut self) -> &mut Buffer<Self::Content>;

    fn selection(&self) -> &SelectionModel;
    fn selection_mut(&mut self) -> &mut SelectionModel;

    fn undo_stack(&self) -> &UndoStack;
    fn undo_stack_mut(&mut self) -> &mut UndoStack;

    fn render_state(&self) -> &RenderState;
    fn render_state_mut(&mut self) -> &mut RenderState;
}
```

### 12.2 Buffer (SumTree 存储)

```
Buffer:
  ┌──────────────────────────────────────┐
  │  content: SumTree<Line>              │   → 当前内容
  │  dirty_lines: HashSet<usize>         │   → 需要重画的行
  │  line_wraps: SumTree<LineWrap>       │   → 折行信息
  │  total_rows: usize                   │   → 总显示行数
  │  max_line_width: f32                 │   → 最宽行宽度
  └──────────────────────────────────────┘

操作:
  - insert(line_idx, text)   → 通过 SumTree::edit 插入行
  - remove(line_idx)         → 通过 SumTree::edit 删除行
  - edit_line(line_idx, f)   → 修改某行的内容
  - get_line(line_idx)       → 通过 Cursor::seek 读取行
```

### 12.3 SelectionModel

```
SelectionModel:
  ┌──────────────────────────────────────┐
  │  selections: Vec<Selection>          │   → 支持多光标
  │  pending: Option<Selection>          │   → 进行中的选择
  │  clipboard: Option<(String, ...)>    │   → 剪切板
  │  display: SelectionDisplay           │   → 显示配置
  └──────────────────────────────────────┘

Selection:
  ┌──────────────────────────────────────┐
  │  start: Position (row, column)       │
  │  end: Position (row, column)         │
  │  reversed: bool                      │   → 是否反向拖动
  │  goal_column: Option<usize>          │   → 纵向移动的目标列
  └──────────────────────────────────────┘
```

### 12.4 UndoStack

```
UndoStack:
  ┌──────────────────────────────────────┐
  │  groups: Vec<UndoGroup>             │   → 撤销分组栈
  │  position: usize                    │   → 当前撤销位置
  │  saved_position: usize              │   → 未保存位置
  │  coalescing: bool                   │   → 是否合并中
  └──────────────────────────────────────┘

UndoGroup:
  ├── transactions: Vec<Transaction>
  │   ├── Transaction { changes: Vec<Change>, timestamp, label }
  │   └── Change { old_text, new_text, position }
  │
  Coalesce 规则：
    同一位置连续输入 → 合并为一个 Transaction
    光标移动 → 断开合并
    超过超时时间 → 断开合并
```

### 12.5 RenderState

```
RenderState:
  ┌──────────────────────────────────────┐
  │  scroll: ScrollPosition              │   → 滚动位置 (row, col)
  │  visible_range: Range<usize>         │   → 可见行范围
  │  cursor_blink: bool                  │   → 光标闪烁状态
  │  highlight_ranges: Vec<Highlight>    │   → 高亮范围（搜索匹配等）
  │  line_cache: Vec<LineRenderInfo>     │   → 缓存的行布局信息
  └──────────────────────────────────────┘
```

---

## 13. 关键模式总结

| # | 模式 | 说明 | 主要涉及模块 |
|---|------|------|-------------|
| 1 | **Entity-Handle** | App 全局持有所有实体，通过类型安全句柄引用 | `warpui_core` |
| 2 | **View + render()** | 每帧重建声明式 Element 树 | `warpui_core` |
| 3 | **Element Builder** | 链式 API 构建 UI 树，编译期安全检查 | `warpui` |
| 4 | **Layout 两阶段** | layout() 测量 + after_layout() 后处理 | `warpui` |
| 5 | **Scene 分层绘制** | Layer 栈 + R-Tree + 原语平面 | `warpui` |
| 6 | **TypedAction** | 类型安全的 Action 分派，编译期验证 | `warpui_core` |
| 7 | **read/update 分离** | 不可变读 vs 可变写，借用检查器保障 | `warpui_core` |
| 8 | **效果队列 (flush_effects)** | 同步批量重渲染，避免帧内多次绘制 | `warpui_core` |
| 9 | **sum_tree B 树** | 持久化平衡树，Cursor 高效定位 | `sum_tree` |
| 10 | **Component** | 持久 State + Params + Options 三件套 | `ui_components` |
| 11 | **订阅/观察** | Model→View 单向数据流通知 | `warpui_core` |

### 13.1 Entity-Handle 模式详解

```
优点：
  - 解耦——View 之间不直接持有彼此引用
  - 安全——类型检查 + 生命周期跟踪
  - 灵活——可运行时创建/销毁实体
  - 可测——通过 Mock App 注入实体

权衡：
  - 间接访问（通过句柄 + 查找），比直接引用略慢
  - 需要小心持有锁期间的死锁（见 TerminalModel::lock() 警告）

适用场景：
  - 所有跨 View/跨 Model 的引用
  - 需要运行时动态创建/销毁的组件
  - 需要序列化/反序列化的实体
```

### 13.2 声明式渲染循环

```
每帧循环：
  1. 处理输入事件（键盘、鼠标、菜单）
     ↓
  2. 执行 Action → 更新 Model
     ↓
  3. 效果队列收集脏 View
     ↓
  4. 脏 View 调用 render() → Element 树重建
     ↓
  5. Layout 两阶段布局
     ↓
  6. Paint → Scene 构建
     ↓
  7. Scene → 平台后端 → 屏幕绘制
```

### 13.3 与常见框架对比

| 维度 | WarpUI | Flutter | React | SwiftUI |
|------|--------|---------|-------|---------|
| 语言 | Rust | Dart | JavaScript | Swift |
| 渲染 | Metal/wgpu/WebGL | Skia | DOM/Virtual DOM | Metal/CoreAnimation |
| 树 | Element 树（手写 Builder） | Widget 树 | Virtual DOM | View 结构体 |
| 状态 | Entity-Handle + read/update | State + InheritedWidget | State + Context | @State + @Binding |
| 布局 | layout() 两阶段 | RenderBox 布局 | CSS/Flexbox | Layout 协议 |
| 事件 | TypedAction + 响应者链 | Callback | Event Handler | @Action |

---

> **文档历史**
> - v1: 2026-07 — 初始版本，基于 AGENTS.md、WARP.md 与 crates 源码归纳
>
> **相关文档**
> - AGENTS.md (§3.1 warpui_core/warpui/ui_components, §5 工程纪律)
> - WARP.md（风格约定、测试流程）
> - `crates/warpui_core/src/lib.rs`——框架核心源码入口
> - `crates/warpui/src/lib.rs`——渲染管线与 Element 树源码入口
> - `crates/sum_tree/src/lib.rs`——B 树实现
