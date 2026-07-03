# 终端模块设计文档

> 本文档描述 Warp 终端模块的整体架构、核心子系统及关键设计模式。
> 对应代码位置：`crates/warp_terminal/`（构建块库）与 `app/src/terminal/`（解析/PTY/渲染集成）。

---

## 1. 架构总览

终端模块采用**两层架构**：

```
┌─────────────────────────────────────────────────────┐
│                  app/src/terminal/                    │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────┐  │
│  │ PTY I/O  │  │ Processor│  │ GridRenderer      │  │
│  │ EventLoop│  │ + Handler│  │ (像素端渲染)      │  │
│  └────┬─────┘  └────┬─────┘  └────────┬──────────┘  │
│       │              │                 │             │
│  ┌────▼──────────────▼─────────────────▼──────────┐  │
│  │           TerminalModel (状态聚合)             │  │
│  └────────────────────────────────────────────────┘  │
└──────────────────────┬──────────────────────────────┘
                       │ 依赖
┌──────────────────────▼──────────────────────────────┐
│                crates/warp_terminal/                  │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────┐  │
│  │ Grid     │  │ Block    │  │ Shell Integration │  │
│  │ Storage  │  │ List     │  │ DCS Protocol      │  │
│  └──────────┘  └──────────┘  └───────────────────┘  │
└─────────────────────────────────────────────────────┘
```

- **crates/warp_terminal**：纯数据结构和算法，无平台依赖。包含网格存储（GridStorage / FlatStorage）、块列表（BlockList）、ANSI 解析器（Processor + Handler trait）、Shell 集成协议（DCS 钩子）、输入分类器。
- **app/src/terminal**：平台相关胶水层。包含 PTY 生命周期管理（EventLoop、PtySpawner）、平台信号桥接（SIGCHLD、SIGWINCH）、像素渲染管线（GridRenderer、CellGlyphCache）、TerminalModel（统一状态聚合，持有网格、块列表、会话状态）。

关键数据流：

```
键盘输入 → InputClassifier → PTY master fd → (子进程 shell)
子进程输出 → PTY master fd → Processor(vte::Parser) → Handler → Grid 更新
                                                            → Block 边界检测
                                                            → GPU 渲染
```

---

## 2. PTY 管理

### 2.1 EventLoop 架构

PTY 读写由 `EventLoop` 驱动，基于 **mio** 事件循环：

```
┌───────────────────────────────────────┐
│            EventLoop                   │
│  ┌──────────┐       ┌──────────────┐  │
│  │ mio::Poll │──────▶│ 事件分派     │  │
│  └──────────┘       └──────┬───────┘  │
│                            │          │
│       ┌────────────────────┼──────┐   │
│       ▼                    ▼     ▼   │
│  on_readable()      on_signal()   ...│
│  (PTY 输出 →        (SIGCHLD/       │
│   Processor)         SIGWINCH)      │
└──────────────────────────────────────┘
```

- `mio::Poll` 注册 PTY master fd 的可读事件和信号 fd。
- `on_readable()` 从 master fd 读取原始字节，送入 `Processor`。
- 信号处理通过 `signalfd`（Linux）或 kqueue `EVFILT_SIGNAL`（macOS）桥接。
- Windows 上使用 `WaitForMultipleObjects` / IOCP 替代。

### 2.2 PTY 生命周期

```
PtySpawner::spawn() ──→ ShellStarter::start() ──→ PtyPair { master, slave }
                                                          │
                    ┌─────────────────────────────────────┤
                    ▼                                     ▼
            EventLoop::run()                     子进程(shell)
            (持有 master fd)                     (持有 slave fd)
```

- `PtySpawner`：工厂，负责创建 PTY 对。平台特化：
  - Unix：`openpty()` + `login_tty()` 或 `posix_openpt()` 系列。
  - Windows：`CreateConPty()` / `InitializeProcThreadAttributeList` 创建 ConPTY。
- `ShellStarter`：在 fork 后（Unix）或创建进程时（Windows）设置会话、环境变量、工作目录，然后 exec shell。

### 2.3 PTY I/O 数据流

```
[Shell stdout] → slave fd → [内核 PTY 缓冲区] → master fd
                                                    │
                              ┌─────────────────────┘
                              ▼
              EventLoop::on_readable()
                              │
                              ▼
                    [原始字节 Vec<u8>]
                              │
                              ▼
                   Processor::advance()
```

写入方向（键盘输入 → shell）：

```
InputClassifier → [原始字节] → master fd → [内核 PTY 缓冲区] → slave fd → shell stdin
```

---

## 3. ANSI/VT 解析

### 3.1 Processor → Handler 模式

解析架构采用**两阶段设计**，将标准兼容的解析与领域语义分离：

```
原始字节
    │
    ▼
Processor (vte::Parser)      ← 通用 VT 解析器（crate vte）
    │  按 Escape / CSI / OSC / DCS / APC 分类
    ▼
Performer (vte::Perform trait)
    │  hook / unhook / put / osc_dispatch / execute / print / ...
    ▼
Handler trait                 ← Warp 定义，解析结果到网格命令的桥接
    │  input_byte() / text() / cursor_up() / scroll_up() / ...
    ▼
GridHandler                   ← 具体实现，操作 GridStorage
```

关键接口：

```rust
// Handler trait：每个 VT 序列解析完成后调用对应方法
pub trait Handler {
    fn input_byte(&mut self, _byte: u8) {}
    fn text(&mut self, _text: &str) {}
    fn cursor_up(&mut self, _rows: usize) {}
    fn cursor_down(&mut self, _rows: usize) {}
    fn cursor_forward(&mut self, _cols: usize) {}
    fn cursor_back(&mut self, _cols: usize) {}
    fn cursor_next_line(&mut self, _rows: usize) {}
    fn cursor_preceding_line(&mut self, _rows: usize) {}
    fn carriage_return(&mut self) {}
    fn linefeed(&mut self) {}
    fn bell(&mut self) {}
    fn scroll_up(&mut self, _rows: usize) {}
    fn scroll_down(&mut self, _rows: usize) {}
    fn insert_blank(&mut self, _chars: usize) {}
    fn delete_characters(&mut self, _chars: usize) {}
    fn insert_blank_lines(&mut self, _rows: usize) {}
    fn delete_lines(&mut self, _rows: usize) {}
    fn erase_display(&mut self, _mode: EraseDisplayMode) {}
    fn erase_chars(&mut self, _chars: usize) {}
    fn erase_line(&mut self, _mode: EraseLineMode) {}
    fn set_cursor_style(&mut self, _style: CursorStyle) {}
    fn set_mode(&mut self, _mode: SetMode) {}
    fn reset_mode(&mut self, _mode: ResetMode) {}
    fn set_scroll_region(&mut self, _top: usize, _bottom: usize) {}
    fn save_cursor_position(&mut self) {}
    fn restore_cursor_position(&mut self) {}
    fn window_title(&mut self, _title: Option<String>) {}
    fn push_title(&mut self, _title: Option<String>) {}
    fn pop_title(&mut self) {}
    fn set_color_scheme(&mut self, _scheme: ColorScheme) {}
    fn reset_color_scheme(&mut self) {}
    fn set_dcs_hook(&mut self, _hook: Option<DCSPassthrough>) {}
    fn put_dcs_data(&mut self, _data: &[u8]) {}
    fn unhook_dcs(&mut self) {}
    fn begin_osc(&mut self) {}
    fn osc_end(&mut self) {}
    fn put_osc_data(&mut self, _data: &[u8]) {}
    fn dispatch_osc(&mut self) {}
    // ... 及其他
}
```

### 3.2 同步输出模式

支持 **DECSET 2006**（同步输出 / Synchronized Output）：

```
DECSET 2006 开启 → 渲染暂停，只写 Grid 内部状态
DECRST 2006 关闭 → 触发一次完整渲染，刷新到屏幕
```

用于避免输出过程中屏幕出现中间态闪烁。协议对称：shell 输出开始时发 `\e[?2006h`，结束时发 `\e[?2006l`。

### 3.3 Tmux 控制模式

当检测到 tmux 控制模式（`\ePtmux;...\e\`）时，DCS 解析路径会特殊处理，将 tmux 包裹的转义序列解包后重新注入 Processor。

---

## 4. 网格 / 滚回架构

### 4.1 两层存储

网格存储采用**写时复制（Copy-on-Write）两层设计**：

```
┌────────────────────────────────────────────┐
│              TerminalGrid                    │
│                                              │
│  ┌──────────────────────┐                   │
│  │    GridStorage       │ ← 活跃网格（可见区域）│
│  │  (可写, 固定尺寸)    │                    │
│  │  rows × cols        │                    │
│  └──────────┬───────────┘                   │
│             │ 滚动时行移出                    │
│             ▼                               │
│  ┌──────────────────────┐                   │
│  │   FlatStorage        │ ← 滚回缓冲区       │
│  │  (写时复制, 不可变)   │   （写时复制）     │
│  │  Arc<[Row]>          │                    │
│  └──────────────────────┘                   │
└────────────────────────────────────────────┘
```

- **GridStorage**：固定尺寸的可写网格（rows × cols），存储当前可见区域的所有 Cell。
- **FlatStorage**：写时复制的只读 Rollback 缓冲区。当行从 GridStorage 顶部滚出时，该行被克隆为 `Arc<Row>` 追加到 FlatStorage。Arc 引用计数保证多线程读安全，且只有真正修改时才产生拷贝。
- 行从底部移入 GridStorage（反向滚回）时，从 FlatStorage pop，Arc 共享至新写入触发 CoW。

### 4.2 Cell 设计

每个 Cell 固定 **24 字节**（对齐 8 字节）：

```rust
#[repr(C)]
pub struct Cell {
    pub character: Char,       // 4 字节 + 4 字节填充
    pub text_style: Option<NonNull<TextStyleRefCell>>,  // 8 字节
    pub background: Option<NonNull<BackgroundRefCell>>, // 8 字节
}
```

- `Char`：Unicode 标量值 + 变体选择器 + 零宽标志。非 BMP 字符（emoji 等）使用 surrogates 编码。
- `TextStyleRefCell` / `BackgroundRefCell`：通过 `Option<NonNull<T>>` 实现共享样式——相同样式的大量 Cell 指向同一个堆分配对象，节省内存。
- 对于空格/空 Cell，`character` 为空格且 `text_style` 为 `None`，`background` 为 `None`，仅占用 24 字节。

### 4.3 BlockGrid 封装

`BlockGrid` 是 `GridStorage` 的上层封装，暴露行/列操作、滚动、光标移动等接口。它实现了 `Handler` trait，是 VT 序列解析的最终消费端。

---

## 5. Shell 集成

### 5.1 DCS 协议

Warp 通过 **DCS（Device Control String）** 转义序列与 shell 集成脚本通信。格式为：

```
\eP<semicolon_separated_parameters> <payload> \e\
```

DCS 参数的首字段为**钩子类型**（Hook Type），标识操作语义。协议钩子定义在 `DProtoHook` 枚举中，包含 **20+ 种类型**：

```
DProtoHook 主要类型：
  PostEnv             → shell 启动后上报环境变量
  Prompt              → 提示符开始/结束标记
  PreExec             → 命令即将执行
  PostEnvResponse     → 环境变量响应
  SetTitle            → 设置窗口标题
  CurrentWorkingDirectory → 当前工作目录
  CommandValue        → 命令完成状态码
  Semantic             → 语义标记（输出/提示/输入区域）
  PostExec             → 命令执行完毕
  AiSuggestion         → AI 建议
  ResumeOngoingCommand → 恢复正在执行的命令
  Features             → 特性协商
  FeatureResponse      → 特性协商响应
  Banner              → 横幅消息
  ForceResizeGrid     → 强制网格重设尺寸
  GridDimensions      → 网格尺寸报告
  ShellVersion        → Shell 版本信息
  UserShell           → 用户选择的 shell 类型
  RightPrompt         → 右侧提示符
  ...
```

### 5.2 引导流程

`bootstrap.sh` 是注入到 shell 启动流程中的模板脚本，自动追加到 `.bashrc`/`.zshrc` 等效内容中。它注册三个钩子函数：

```
precmd               → 命令执行前（提示符显示前）
    │  发送 DCS Prompt + DCS PreExec
    ▼
preexec              → 命令执行后、等待输出前
    │  发送 DCS CommandValue（上一条命令退出码）
    ▼
command_finished     → 输出结束后
    │  发送 DCS PostExec
    ▼
                     → 下一次 precmd
```

引导流程：

1. Shell 启动，bootstrap.sh 执行。
2. 发送 `DCS PostEnv` 上报环境。
3. Warp 回复环境变量设置（`TERM` 等）。
4. bootstrap.sh 注册 `precmd` / `preexec` / `command_finished`。
5. 进入正常命令周期。

### 5.3 提示 / 输出边界检测

Shell 集成脚本标记输出区域边界：

```
precmd:
  ┌─ DCS Prompt        → 提示符开始
  输出提示符文本
  └─ DCS Prompt (end)  → 提示符结束

preexec:
  ┌─ DCS PreExec       → 命令开始
  命令回显（如果 echo on）
  └─ (等待命令执行)

command_finished:
  ┌─ DCS PostExec      → 输出结束
  ┌─ DCS CommandValue  → 退出码
  输出后续 shell 信息
```

Warp 根据这些标记确定：
- 哪个范围属于提示符（渲染时特殊颜色）。
- 哪个范围属于命令输出（作为块内容）。
- 命令是否仍在执行（未收到 PostExec）。

---

## 6. 终端块系统

### 6.1 Block 结构

每次命令执行构成一个 **Block**（终端块）：

```rust
pub struct Block {
    pub command_grid: Option<BlockGrid>,      // 命令文本区域
    pub output_grid: Option<BlockGrid>,        // 输出文本区域
    pub rich_content: Option<BlockRichContent>, // 富内容（图片、链接、结构化数据）
    pub gap: Gap,                              // 底部间隙（空行区域）
}
```

- `command_grid`：命令文本的只读快照。
- `output_grid`：命令输出的只读快照。
- `rich_content`：如果输出包含结构化数据（表格、链接预览等），存储在此。
- `gap`：块之间的空白行区域，由块高度计算而来（详见 6.3）。

### 6.2 BlockList（SumTree）

所有 Block 的组织结构是 `BlockList`，底层实现为 **SumTree**（求和树）：

```
BlockList (SumTree<BlockHeightItem>)
                    │
      ┌─────────────┼─────────────┐
      ▼             ▼             ▼
   Block 0      Block 1      Block 2
   (height: 20)  (height: 35)  (height: 18)
                    │
      ┌─────────────┴─────────┐
      ▼                       ▼
   [可见范围]             [滚出范围]
```

SumTree 的每个节点缓存以其为根的子树高度总和，使得：

- **通过 Y 坐标定位 Block**：O(log n)——二分查找累计高度。
- **插入/删除 Block**：O(log n)。
- **范围查询**（哪些 Block 在当前视口中）：O(log n + k)。

### 6.3 间隙系统

每个 Block 底部有一个 `Gap`，其高度 = `max(0, viewport_height - block_total_height - fixed_gap)`。

```
┌─────────────────────┐
│  Block 0            │
│  (command + output) │  ← 高度自适应
├─────────────────────┤
│  Gap                │  ← 补充至视口边缘
│  (空行)             │
├─────────────────────┤  ← 下一个 Block 顶部
│  Block 1            │
│  ...                │
└─────────────────────┘
```

Gap 机制保证至少有一个完整"块"填满终端窗口，避免终端底部出现无归属空白。

### 6.4 状态机

每个 Block 的状态在块生命周期中转换：

```
         Idle
           │
           │ 收到 DCS PreExec 或检测到新命令
           ▼
       Executing
           │
           │ 收到 DCS PostExec（含退出码）
           ▼
   DoneWithExecution
           │
           │ 用户上下滚动、Block 滚出/入可见区域
           ▼
         Idle（或复用）
```

---

## 7. 渲染管线

### 7.1 架构概览

渲染从 `GridRenderer` 开始，最终输出到 GPU 纹理：

```
GridRenderer::render_frame()
    │
    ├── 收集脏区域（dirty lines / dirty blocks）
    │
    ├── QuadGeneration（生成着色器需要的顶点数据）
    │   ├── CellQuad：每个字符的纹理坐标 + 前景/背景色 + 位置
    │   ├── CursorQuad：光标的样式（块/下划线/竖线）
    │   ├── SelectionQuad：选中背景高亮
    │   └── ImageQuad：终端内嵌图片（Kitty 协议）
    │
    ├── CellGlyphCache 查找/生成字形纹理
    │
    └── 提交 GPU 绘制调用
```

### 7.2 CellGlyphCache

字形缓存桥接文本渲染：

```
CellGlyphCache
    │
    ├── 字形纹理图集（Glyph Atlas）
    │   ├── 最近使用的 glyph 缓存在 GPU 纹理上
    │   └── 新 glyph 按需 rasterize + 上传
    │
    ├── 字体回退链
    │   ├── 主字体 → 按优先级回退字体
    │   └── 每个字符族走不同回退路径
    │
    └── 特殊字形处理
        ├── 连字（Ligature）检测
        ├── Emoji（彩色位图 vs 单色 fallback）
        └── 零宽字符（组合标记）
```

### 7.3 附加渲染层

- **光标渲染**：支持块（Block）、下划线（Underline）、竖线（Bar）三种样式，由 VT 序列 `\e[ q` 系列控制。
- **选择渲染**：用户鼠标选择的文本区域背景高亮。跨块选择通过 SumTree 定位。
- **图片渲染**：Kitty 图形协议支持，`DCS Gi...` 序列解析后生成 ImageQuad。
- **URL 高亮**：输出文本中检测到的 URL，在渲染时加下划线 + 特殊颜色。
- **秘密模糊处理**：`DCS Secrets` 标记的敏感内容在渲染时用模糊方块遮盖（屏幕共享 / 录屏保护）。

### 7.4 ColorSampler

终端输出中的 256 色 / TrueColor 由 `ColorSampler` 处理色彩映射。它处理：

- 标准 16 色映射到主题色板。
- 256 色扩展映射。
- TrueColor（24-bit）直通。
- 透明/默认颜色的回退。
- 粗体/斜体/下划线的色彩亮度调整。

---

## 8. 输入分类

### 8.1 架构

`InputClassifier` trait 定义统一接口：

```rust
pub trait InputClassifier {
    fn classify(&self, input: &str, context: &InputContext) -> Classification;
}
```

返回的 `Classification` 枚举：

```rust
pub enum Classification {
    ShellInput,        // 纯 Shell 命令/快捷键
    NaturalLanguage,   // 自然语言（AI Prompt）
    PartialCommand,    // 不完整命令（需要更多上下文）
}
```

### 8.2 分类器实现

#### HeuristicClassifier（默认，始终启用）

启发式决策树按顺序做短路判断：

```
输入字符串
    │
    ├── 以 CJK 字符开头 → NaturalLanguage
    │
    ├── 以 shell 关键字开头（cd / ls / git / sudo / ...）→ ShellInput
    │
    ├── 包含等号且无空格 → ShellInput（环境变量赋值）
    │
    ├── 以特殊字符开头（/ . ~ $ %）→ ShellInput
    │
    ├── 自然语言评分（NL Score）
    │   ├── 首词在自然语言词典中且不在 shell 命令表中 → 加分
    │   ├── 包含 "is / are / what / how / why / when / who" → NL 倾向
    │   ├── 包含 shell 特殊符号（| > < & ;）→ Shell 倾向
    │   └── 总评分 > 阈值 → NaturalLanguage
    │
    └── 默认 → ShellInput
```

#### ML 分类器（可选，通过 feature flag 启用）

- **ONNX 模型**（Linux/macOS/Windows）：加载 `.onnx` 模型文件，对输入进行 token embedding + 分类。
- **fastText 模型**（轻量备选）：基于词向量的快速分类，启动快、体积小。
- 运行时通过 `FeatureFlag` 控制启用，回退到 HeuristicClassifier。

### 8.3 自然语言探测子 crate

`crates/natural_language_detection` 提供底层 NLP 检测能力（CJK 范围判断、英语词典、常见 shell 命令表），被 `input_classifier` 使用。

---

## 9. 关键设计模式

### 9.1 Handler trait

将 VT 解析与网格操作解耦。`Processor`（通用 VT 解析器）调用 `Handler` trait 方法，`GridHandler` 是具体实现。可以插入中间层（如 DCS 钩子处理、Tmux 解包）而不修改解析逻辑。

### 9.2 写时复制（Copy-on-Write）

FlatStorage 中每行是 `Arc<Row>`。滚动时将 GridStorage 中的行移出，克隆为 `Arc`；需要修改已移出的行时才真正 clone。适用于终端的"写少读多"模式——大部分时间只有最底部几行在被修改。

### 9.3 SumTree 块列表

BlockList 使用 SumTree 存储块元数据，解决"通过屏幕 Y 坐标快速定位块"的核心需求。缓存子树高度总和使 O(n) 的遍历降为 O(log n)。SumTree 也是编辑器和 Notebook crate 的核心数据结构，跨模块复用。

### 9.4 事件驱动的块检测

Shell 集成不足时（如 Docker 容器内、旧版 shell），回退到**启发式块检测**：

- **Timing heuristic**：输出暂停超过阈值 ⇒ 上一个块结束。
- **Prompt 模式匹配**：检测常见的提示符结尾特征（`$ `、`% `、`# `）。
- **等待/就绪探测**：向 shell 发送信号/空命令，等待输出稳定。

三种策略结合，保证即使无 DCS 钩子也能合理分块。

### 9.5 并发与锁

TerminalModel 在多线程环境中被访问：

- **Renderer 线程**：只读读取 Grid + BlockList 用于渲染。
- **EventLoop 线程**：写入 Grid、更新 BlockList。
- **UI 线程**：读取选择状态、滚动位置。

使用 **FairMutex**（`parking_lot::FairMutex`）保证写线程不饿死。读操作通过 Arc 共享 FlatStorage 行，读时不阻塞写。

```
EventLoop 线程        Renderer 线程         UI 线程
    │                    │                    │
    │  lock()            │  try_lock()        │  try_lock()
    ▼                    ▼                    ▼
TerminalModel (FairMutex 保护)
    │
    ├── Grid (GridStorage + FlatStorage)
    ├── BlockList (SumTree)
    ├── Cursor 状态
    ├── Selection 状态
    └── Scroll 位置
```

---

## 附录

### A. 文件布局速查

| 路径 | 职责 |
|------|------|
| `crates/warp_terminal/src/grid/` | GridStorage、FlatStorage、Cell、Row |
| `crates/warp_terminal/src/block/` | Block、BlockList、BlockHeightItem、Gap |
| `crates/warp_terminal/src/processor/` | Processor、Handler trait、GridHandler |
| `crates/warp_terminal/src/shell_integration/` | DCS 协议解析、DProtoHook 枚举、DCS 调度 |
| `crates/warp_terminal/src/input_classifier/` | InputClassifier trait、HeuristicClassifier、ML 分类器 |
| `crates/input_classifier/` | 输入分类器独立 crate（被 warp_terminal 依赖） |
| `crates/natural_language_detection/` | CJK 检测、shell 命令表、NL 评分 |
| `app/src/terminal/` | PTY 生命周期、EventLoop、TerminalModel、GridRenderer 集成 |
| `app/src/terminal/event_loop.rs` | mio 事件循环，PTY 读写驱动 |
| `app/src/terminal/pty.rs` | PtySpawner、PtyPair、平台 PTY 抽象 |
| `app/src/terminal/renderer/` | GridRenderer、CellGlyphCache、QuadGeneration |
| `resources/shell_integration/` | bootstrap.sh 模板 |


### B. 相关 crate 依赖

```
warp_terminal
  ├── vte (VT 序列解析器)
  ├── sum_tree (BlockList 底层)
  ├── parking_lot (FairMutex)
  ├── input_classifier
  └── natural_language_detection

app (warp 主二进制)
  ├── warp_terminal
  ├── warpui / warpui_core (UI 框架)
  ├── persistence (SQLite)
  └── ...
```
