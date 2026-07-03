# 编辑器、补全与搜索模块设计文档

> 本文档涵盖 `crates/editor`(文本编辑器)、`crates/warp_completer`(补全引擎)、`app/src/code_review`(代码评审)以及 `app/src/search`(搜索系统)四大模块。所有模块构建在 SumTree、Entity-Handle 以及增量处理等基础设施之上。

---

## 目录

1. [编辑器架构总览](#1-编辑器架构总览)
2. [Buffer 系统](#2-buffer-系统)
3. [撤销系统](#3-撤销系统)
4. [选择与导航](#4-选择与导航)
5. [渲染层](#5-渲染层)
6. [语法高亮](#6-语法高亮)
7. [LSP 集成](#7-lsp-集成)
8. [补全引擎](#8-补全引擎)
9. [代码评审](#9-代码评审)
10. [搜索系统](#10-搜索系统)
11. [关键模式与工程纪律](#11-关键模式与工程纪律)

---

## 1. 编辑器架构总览

编辑器位于 `crates/editor`(包名 `warp_editor`)，是整个 Warp UI 中最核心的文本操作基础设施。终端、AI 对话输入、代码编辑、Notebook 等所有文本输入场景均构建于其上。

### 1.1 四层架构

编辑器从底向上分为四个抽象层：

```
┌──────────────────────────────────────────────────────────┐
│                     RenderElement                         │
│   (Element trait 实现，声明式 UI 描述)                     │
│   +── RichTextElement: 富文本段落/块渲染                     │
│   +── InlineElement: 行内元素(文本/颜色/样式)               │
├──────────────────────────────────────────────────────────┤
│                     RenderState                           │
│   (换行计算、视口裁剪、行→屏幕位置偏移图)                   │
│   +── 换行算法(wrap/nowrap)                                │
│   +── PositionCache: byte offset ↔ screen coordinate      │
│   +── LayoutLines 集合                                     │
├──────────────────────────────────────────────────────────┤
│                     SelectionModel                         │
│   (多选管理、TextUnit 导航、Anchor 夹紧)                    │
│   +── BufferSelection: anchor + head + gravity             │
│   +── goal_x: 垂直移动保持列                               │
│   +── 多选区支持                                           │
├──────────────────────────────────────────────────────────┤
│                     Buffer                                │
│   (文本存储核心: SumTree, 版本化, 编辑事务)                │
│   +── 纯文本模式(PlainTextBuffer)                          │
│   +── Markdown 模式(AstBuffer)                             │
│   +── BufferVersion + PreciseDelta                         │
└──────────────────────────────────────────────────────────┘
```

每一层只依赖下层，上层通过 `CoreEditorModel` trait 统一访问。

### 1.2 CoreEditorModel trait

`CoreEditorModel` 是编辑器的核心抽象 trait，定义所有编辑器实现必须提供的方法：

```rust
pub trait CoreEditorModel: 'static + Send {
    fn buffer(&self) -> &dyn Buffer;           // 底层 buffer
    fn buffer_mut(&mut self) -> &mut dyn Buffer;
    fn selection(&self) -> &SelectionModel;    // 选区模型
    fn selection_mut(&mut self) -> &mut SelectionModel;
    fn render_state(&self) -> &RenderState;    // 渲染状态
    fn render_state_mut(&mut self) -> &mut RenderState;
    fn undo_manager(&self) -> &UndoManager;    // 撤销管理器
    fn undo_manager_mut(&mut self) -> &mut UndoManager;

    // 核心操作
    fn edit(
        &mut self,
        action: impl Into<BufferEditAction>,
        ctx: &mut AppContext,
    );

    fn insert(&mut self, text: &str, ctx: &mut AppContext);
    fn backspace(&mut self, ctx: &mut AppContext);
    fn newline(&mut self, ctx: &mut AppContext);

    // 渲染
    fn build_elements(&self, ctx: &mut AppContext) -> Vec<Dom>;
}
```

### 1.3 两个具体实现

| 实现 | 用途 | 特点 |
|------|------|------|
| `PlainTextEditorModel` | 终端、纯文本编辑器、代码编辑器 | SumTree 直接存储，位置映射简单，效率高 |
| `RichTextEditorModel` | AI 对话、Markdown 编辑、Notebook | 底层 AstBuffer(Markdown AST)，支持富文本块渲染，行内样式 |

```
PlainTextEditorModel          RichTextEditorModel
       │                              │
       ▼                              ▼
  PlainTextBuffer                AstBuffer
  (SumTree<String>)          (Markdown AST)
       │                              │
       ▼                              ▼
  SelectionModel                SelectionModel
  (byte offset)                (byte offset + AST node)
       │                              │
       ▼                              ▼
  RenderState                   RenderState
  (等宽字体，无换行或简单换行)   (可变字体，富文本换行，块级渲染)
       │                              │
       ▼                              ▼
  PlainTextElement              RichTextElement
  (单一文本行)                  (段落/标题/代码块/表格/图片/…)
```

### 1.4 Entity-Handle 集成

编辑器实例通过 ViewHandle 体系管理：

```
App
├── EditorViewHandle                 // 指向编辑器实体的句柄
│   └── EditorModel                  // Entity 内的实际数据
│       ├── model: Box<dyn CoreEditorModel>
│       ├── focus_handle: FocusHandle
│       ├── scroll_position: Point
│       └── ...
├── TerminalViewHandle
├── SearchViewHandle
└── ...
```

编辑器作为 `Entity` 注册在 `App` 的实体注册表中，通过 `ViewHandle<EditorModel>` 引用。视图层负责将 `RenderElement` 树转换为实际 UI。

---

## 2. Buffer 系统

Buffer 是编辑器的文本存储核心，位于 `crates/editor/src/buffer/`。

### 2.1 SumTree 文本存储

Buffer 底层使用 `crates/sum_tree` 中的持久化平衡 B-树存储文本：

```
SumTree 结构:
┌──────────────────────────────┐
│           Root Node           │
│  ┌──────┬──────┬──────┐     │
│  |  N1  |  N2  |  N3  |     │
│  └──┬───┴──┬───┴──┬───┘     │
│     │      │      │          │
│  ┌──▼─┐ ┌──▼─┐ ┌──▼─┐       │
│  | L1 | | L2 | | L3 |       │
│  | buf| | buf| | buf|       │
│  └────┘ └────┘ └────┘       │
└──────────────────────────────┘

每个叶节点包含一段连续文本(String)
每个内部节点维护子树总字符数(用于 O(log N) 偏移量查找)
```

SumTree 的选择理由：
- **持久化**: 编辑操作返回新树，旧树不变，支持无锁撤销
- **O(log N) 随机访问**: 通过节点缓存的宽度(字符数)快速定位
- **差分友好**: 相邻版本共享大部分节点

### 2.2 两种 Buffer 模式

#### 2.2.1 PlainTextBuffer

```rust
pub struct PlainTextBuffer {
    text: SumTree<TextChunk>,       // 文本存储
    version: BufferVersion,         // 当前版本
    edit_history: Vec<BufferEditAction>,  // 编辑历史(供撤销使用)
}

struct TextChunk {
    text: String,
    width: usize,                   // 字符数(缓存)
}
```

- 纯文本模式，没有 AST
- 所有操作在 byte offset 级别进行
- `text()` 返回完整字符串，`slice(range)` 返回子串

#### 2.2.2 AstBuffer(Markdown 模式)

```rust
pub struct AstBuffer {
    source: String,                  // 原始 Markdown 文本
    ast: MarkdownAst,                // 解析后的 AST
    version: BufferVersion,
}

pub enum MarkdownAst {
    Document(Vec<MarkdownBlock>),
}

pub enum MarkdownBlock {
    Paragraph(MarkdownParagraph),
    Heading { level: u8, children: Vec<InlineItem> },
    CodeBlock { language: Option<String>, code: String },
    List { ordered: bool, items: Vec<ListItem> },
    Table(TableData),
    MermaidBlock { code: String },
    Image { url: String, alt: String },
    // ...
}
```

AstBuffer 的特点：
- 同时保留原始文本和 AST，两者通过 offset mapping 同步
- 编辑操作同时更新 source 和 AST（增量解析）
- 编辑器展示时使用 AST 的语义渲染

### 2.3 版本系统

#### BufferVersion

```rust
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BufferVersion {
    pub epoch: u64,     // 每次"有意义变更"递增
    pub tx_id: u64,     // 事务内递增(每次 edit 递增)
}
```

版本号用于：
- 协调多个消费者(渲染、语法高亮、撤销)
- 判断缓存是否过期
- 多线程场景下的无锁读取

#### PreciseDelta

```rust
pub struct PreciseDelta {
    pub old_start: Offset,
    pub old_end: Offset,
    pub new_start: Offset,
    pub new_end: Offset,
    pub text_utf8: String,
}
```

每个编辑操作产生一个 PreciseDelta，描述文本的变化范围。增量消费者(语法高亮、渲染缓存)据此更新自身状态。

### 2.4 BufferEditAction

```rust
pub enum BufferEditAction {
    Insert {
        offset: Offset,
        text: String,
    },
    Delete {
        start: Offset,
        end: Offset,
    },
    Replace {
        start: Offset,
        end: Offset,
        text: String,
    },
    Multiple(Vec<BufferEditAction>),  // 复合操作
}
```

`edit()` 方法接收 `impl Into<BufferEditAction>`，执行操作并：
1. 更新 SumTree
2. 更新 BufferVersion
3. 记录编辑历史
4. 通知撤销管理器
5. 触发相关回调(如语法高亮增量更新)

### 2.5 Offset 类型

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Offset(pub usize);  // byte offset from start of buffer

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TextUnit {
    pub kind: TextUnitKind,
    pub direction: Direction,
}

pub enum TextUnitKind {
    Character,
    Grapheme,       // Unicode 字素簇
    Word,
    WordBoundary,
    Subword,        // camelCase/snake_case 子词
    Line,
    Document,
    Paragraph,
    // Markdown-aware:
    MarkdownBlock,
    MarkdownSibling,
}
```

`TextUnit` 配合 `Direction`(Left/Right/Up/Down) 组成通用的导航语义，用于光标移动、选择扩展、删除字词等操作。

---

## 3. 撤销系统

撤销系统位于 `crates/editor/src/undo.rs`。

### 3.1 核心数据结构

```rust
pub struct UndoManager {
    undo_stack: Vec<UndoStackItem>,
    redo_stack: Vec<UndoStackItem>,
    merge_timer: Option<Instant>,        // 500ms 合并窗口
    max_undo_depth: usize,               // 通常为 500
}

struct UndoStackItem {
    actions: Vec<ReversibleEditorAction>,  // 该批次内的所有操作
    selection_before: SelectionSnapshot,   // 操作前的选区
    selection_after: SelectionSnapshot,    // 操作后的选区
    timestamp: Instant,                    // 合并时间戳
    version_at_creation: BufferVersion,    // 当时的 buffer 版本
}
```

### 3.2 撤销/重做流程

```
Undo 流程:

1. User presses Ctrl+Z
2. UndoManager.pop() → 取出 UndoStackItem
3. 对 actions 逆序应用 reverse() 操作
4. 将恢复的 actions 压入 redo_stack
5. 恢复 selection_before

Redo 流程 (对称):

1. User presses Ctrl+Shift+Z
2. UndoManager.redo() → 取出 RedoStackItem
3. 按原始顺序应用 forward() 操作
4. 压入 undo_stack
5. 恢复 selection_after
```

### 3.3 ReversibleEditorAction

```rust
pub struct ReversibleEditorAction {
    forward: BufferEditAction,    // 正向操作
    reverse: BufferEditAction,    // 逆向操作
}

impl ReversibleEditorAction {
    pub fn perform_forward(&self, buffer: &mut dyn Buffer) -> PreciseDelta {
        // 执行 forward 操作
    }

    pub fn perform_reverse(&self, buffer: &mut dyn Buffer) -> PreciseDelta {
        // 执行 reverse 操作
    }
}
```

构造时即计算 reverse 操作，确保撤销与重做完全对称。

### 3.4 合并策略

撤销条目在特定条件下合并为一批，使连续输入可以被一次撤销：

```rust
impl UndoManager {
    fn push_action(&mut self, action: ReversibleEditorAction, selection: SelectionSnapshot) {
        if let Some(last) = self.undo_stack.last_mut() {
            let elapsed = last.timestamp.elapsed();
            // 合并条件:
            // 1. 距离上次操作 < 500ms
            // 2. 操作在相同区间(连续输入)
            // 3. 没有选区变化
            if elapsed < Duration::from_millis(500)
                && self.is_adjacent(&last.actions.last(), &action)
                && last.selection_after == selection
            {
                last.actions.push(action);
                last.selection_after = selection;
                last.timestamp = Instant::now();
                return;
            }
        }

        // 不满足合并条件，新建条目
        self.undo_stack.push(UndoStackItem {
            actions: vec![action],
            selection_before: self.current_selection(),
            selection_after: selection,
            timestamp: Instant::now(),
            version_at_creation: self.current_version(),
        });
    }
}
```

合并阈值 500ms 是人工调优的结果——太快则单字符合并不够，太慢则用户难以精确回退到某一步。

### 3.5 版本化撤销条目

每个 UndoStackItem 在创建时记录了 `version_at_creation`。当 buffer 版本因外部变更(如同步、AI 修改)产生跳跃时，对应的 undo 条目标记为失效，防止撤销导致不一致。

---

## 4. 选择与导航

位于 `crates/editor/src/selection/`。

### 4.1 SelectionModel

```rust
pub struct SelectionModel {
    selections: Vec<BufferSelection>,
    preferred_selection: usize,      // 主选区(接受输入)
    last_operation: SelectionOperation,
}

pub struct SelectionSnapshot {
    pub selections: Vec<BufferSelection>,
    pub preferred_selection: usize,
}

pub struct BufferSelection {
    pub start: Offset,        // anchor(固定端)
    pub end: Offset,          // head(移动端)
    pub gravity: Gravity,     // 插入时向哪端扩展
    pub goal_x: Option<f32>,  // 垂直移动时的目标列
    pub reversed: bool,       // 视觉方向(start > end)
}

pub enum Gravity {
    Left,
    Right,
}
```

### 4.2 Anchor 夹紧规则

当编辑操作导致选区位置变化时，`BufferSelectionModel` 负责夹紧(Clamp)选区：

```
Anchor Left 夹紧:
  文本: "Hello World"
  选区: [2, 7]  → "llo W"
  删除 [0, 5]: "Hello" → " World"
  夹紧后: [0, 2]  (因为 anchor 从 2 向左夹紧到 0)

Anchor Right 夹紧:
  文本: "Hello World"
  选区: [2, 7]  → "llo W"
  删除 [5, 11]: " World" → "Hello"
  夹紧后: [2, 5]  (因为 head 从 7 向右夹紧到 5)
```

```rust
pub struct BufferSelectionModel;

impl BufferSelectionModel {
    pub fn clamp(selection: &BufferSelection, delta: &PreciseDelta) -> BufferSelection {
        // 根据 gravity 方向，将 anchor/head 映射到编辑后的位置
        // Left: anchor 不动，head 被推/拉
        // Right: head 不动，anchor 被推/拉
        let new_start = match selection.gravity {
            Gravity::Left => clamp_offset(selection.start, delta, ClampMode::Anchor),
            Gravity::Right => clamp_offset(selection.start, delta, ClampMode::Slack),
        };
        let new_end = match selection.gravity {
            Gravity::Left => clamp_offset(selection.end, delta, ClampMode::Slack),
            Gravity::Right => clamp_offset(selection.end, delta, ClampMode::Anchor),
        };
        BufferSelection { start: new_start, end: new_end, ..*selection }
    }
}
```

### 4.3 多选支持

SelectionModel 支持同时存在多个选区：

```rust
impl SelectionModel {
    pub fn add_selection(&mut self, sel: BufferSelection);
    pub fn remove_selection(&mut self, idx: usize);
    pub fn clear_selections(&mut self);
    pub fn set_cursor(&mut self, offset: Offset);

    // 多选编辑: 对每个选区分别应用操作
    pub fn edit_each_selection(
        &mut self,
        f: impl Fn(&BufferSelection) -> BufferEditAction,
    ) -> Vec<BufferEditAction>;
}
```

多选编辑时，按照选区从后到前的顺序执行，以避免偏移量因前面的编辑而失效。

### 4.4 TextUnit 导航

```rust
impl SelectionModel {
    pub fn move_cursor(
        &mut self,
        unit: TextUnit,
        direction: Direction,
        buffer: &dyn Buffer,
    );
}
```

导航算法：
1. 根据 `TextUnitKind` 确定边界函数(如 `next_word_boundary`, `prev_line_start`)
2. 从当前 offset 沿 direction 计算新 offset
3. 如果 shift 键按下，扩展选区而非移动
4. 如果是垂直移动(Up/Down)，使用 `goal_x` 保持列位置

```
垂直移动 goal_x 算法:

当前行: "Hello World"
光标在: "Hello| World"  (x=5)

按 ↓:
  下一行: "Foo Bar Baz"
  计算: 取 min(下一行长度, goal_x=5)
  光标到: "Foo B|ar Baz"

按 ↑ 回到第一行:
  使用存储的 goal_x=5
  光标回到: "Hello| World"  (而非上次的 x 位置)
```

### 4.5 Markdown 感知导航

RichTextEditorModel 的导航额外感知 Markdown 结构：

```
行首导航 (Markdown 模式):

  文本: "- [ ] 任务项"
                ↑
       按 Home → 跳到 "- " 之后(列表标记后)
       再按 Home → 跳到行首

块导航:

  光标在段落中 → Ctrl+↑ → 跳到段落开头(而不是上一行)
  再按 Ctrl+↑ → 跳到上一个块的末尾
```

---

## 5. 渲染层

位于 `crates/editor/src/render/`。

### 5.1 RenderState

```rust
pub struct RenderState {
    wraps: bool,                              // 是否换行
    viewport: Viewport,                       // 可见区域
    layout_lines: Vec<LayoutLine>,            // 布局后的行集合
    position_map: PositionCache,              // offset ↔ row/col 映射

    // 性能优化
    dirty_lines: RangeSet<usize>,             // 需要重新布局的行
    last_built_version: BufferVersion,        // 上次构建时的版本
}

pub struct LayoutLine {
    pub start_offset: Offset,                 // 行在 buffer 中的起始偏移
    pub end_offset: Offset,                   // 行结束偏移(含换行符)
    pub is_wrap_continuation: bool,           // 是否是软换行的续行
    pub visual_line_index: usize,             // 视觉行号
    pub height: f32,                          // 行高(富文本中可变)
}

pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

### 5.2 位置缓存 (PositionCache)

```rust
pub struct PositionCache {
    lines: Vec<PositionCacheLine>,
    total_rows: usize,
}

struct PositionCacheLine {
    offset: Offset,              // buffer 中的起始偏移
    length: usize,               // 字符数
    visual_rows: usize,          // 因软换行占据的行数
    positions: Vec<ColumnPosition>,  // 每个字符的 x 位置
}
```

可以实现 O(log N) 的 `offset_to_point(offset) → (row, col)` 和 `point_to_offset(row, col) → offset` 查询。

### 5.3 换行算法

```rust
impl RenderState {
    fn relayout(&mut self, buffer: &dyn Buffer, width: f32) {
        if !self.wraps {
            self.relayout_nowrap(buffer);
            return;
        }

        let mut new_lines = Vec::new();
        for logical_line in buffer.lines() {
            if self.needs_wrap(logical_line, width) {
                self.wrap_line(logical_line, width, &mut new_lines);
            } else {
                new_lines.push(LayoutLine::from_logical_line(logical_line));
            }
        }
        self.layout_lines = new_lines;
        self.rebuild_position_cache();
    }
}
```

### 5.4 RichTextElement

`RichTextElement` 是富文本编辑器的渲染元素：

```rust
pub struct RichTextElement {
    pub children: Vec<RenderableBlock>,
    pub width: f32,
    pub height: f32,
}

pub enum RenderableBlock {
    Paragraph {
        lines: Vec<TextLine>,
        styles: Vec<TextStyleRange>,
    },
    Heading {
        level: u8,
        lines: Vec<TextLine>,
        styles: Vec<TextStyleRange>,
    },
    CodeBlock {
        language: Option<String>,
        lines: Vec<String>,
        highlighted_tokens: Vec<SyntaxToken>,
    },
    List {
        ordered: bool,
        bullet: String,        // "-", "1.", etc.
        items: Vec<RenderableBlock>,
        indent_level: usize,
    },
    Blockquote {
        lines: Vec<TextLine>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        column_alignments: Vec<Alignment>,
    },
    Mermaid {
        code: String,
        rendered_image: Option<ImageData>,
    },
    Image {
        url: String,
        alt: String,
        loaded: bool,
        dimensions: Option<(f32, f32)>,
    },
    HorizontalRule,
}
```

### 5.5 块渲染流程

```
AstBuffer  →  RenderableBlock Vec  →  RichTextElement  →  UI

1. AstBuffer.ast  →  遍历 MarkdownAst
2. 每个 MarkdownBlock → RenderableBlock 转换
   - 应用语法高亮(如果可用)
   - 计算行高(标题、代码块使用不同字体大小)
   - 解析行内样式(粗体/斜体/代码/链接)
3. 收集到 RichTextElement.children
4. Element 树 → 实际的 GPU 渲染
```

### 5.6 增量渲染

渲染层只重新构建变化的部分：

```
编辑发生
  │
  ▼
PreciseDelta  →  标记 dirty_lines 范围
  │
  ▼
Relayout dirty_lines  +  相邻行(因换行变化)
  │
  ▼
重建 PositionCache 中受影响的行段
  │
  ▼
生成新的 RenderElement 子树(只替换变化节点)
  │
  ▼
WarpUI diff → 最小化实际 DOM 更新
```

---

## 6. 语法高亮

位于 `crates/editor/src/syntax/`，基于 Tree-sitter 构建。

### 6.1 架构总览

```
┌────────────────────────────────────────────┐
│           SyntaxTreeState                   │
│  ├── tree: Tree (Tree-sitter 解析树)        │
│  ├── language: &Language                    │
│  ├── highlight_cache: HighlightCache        │
│  ├── pending_edit: Option<InputEdit>        │
│  └── async_parse_task: Option<JoinHandle>   │
├────────────────────────────────────────────┤
│           HighlightCache                    │
│  ├── captures_by_line: Vec<Vec<Capture>>    │
│  ├── version: BufferVersion                 │
│  └── color_map: ColorMap                    │
├────────────────────────────────────────────┤
│           ColorMap                          │
│  ├── semantic_token → style mapping         │
│  └── theme-aware color resolution           │
├────────────────────────────────────────────┤
│         Tree-sitter Layer                   │
│  ├── Language 注册表                        │
│  ├── Query 文件(.scm) 管理                  │
│  └── 增量解析引擎                           │
└────────────────────────────────────────────┘
```

### 6.2 增量解析

Tree-sitter 支持基于 `InputEdit` 的增量解析，这是高性能高亮的关键：

```rust
pub struct InputEdit {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start_position: Point,    // {row, column}
    pub old_end_position: Point,
    pub new_end_position: Point,
}

impl SyntaxTreeState {
    pub fn apply_edit(&mut self, delta: &PreciseDelta, buffer: &dyn Buffer) {
        let input_edit = InputEdit {
            start_byte: delta.old_start.0,
            old_end_byte: delta.old_end.0,
            new_end_byte: delta.new_end.0,
            // 位置信息从 position cache 获取
            start_position: buffer.offset_to_point(delta.old_start),
            old_end_position: buffer.offset_to_point(delta.old_end),
            new_end_position: buffer.offset_to_point(delta.new_end),
        };

        // 同步更新语法树
        self.tree.edit(&input_edit);

        // 标记需要重解析
        self.pending_edit = Some(input_edit);
    }

    /// 在主线程(或后台线程)执行增量解析
    pub fn reparse(&mut self, buffer_text: &[u8]) -> Result<(), ParseError> {
        let new_tree = self.tree.clone();
        let cursor = new_tree.walk();

        // Tree-sitter 增量解析: 传入旧树，只重新解析变化区域
        let result = parser.parse_with(
            buffer_text,
            Some(&self.tree),  // 旧树
        );

        if let Ok(new_tree) = result {
            self.tree = new_tree;
            self.pending_edit = None;
            self.update_highlights(buffer_text);
        }
    }
}
```

### 6.3 异步后台解析

为了避免输入卡顿，重解析在后台线程异步执行：

```
用户输入
  │
  ▼
apply_edit()  →  同步更新树结构(O(1))
  │
  ▼
标记 pending_edit
  │
  ▼
触发异步任务:
  1. 克隆当前 tree
  2. 在后台线程执行 parser.parse_with()
  3. 完成后通过 channel 发送结果
  │
  ▼
主线程接收结果:
  1. 检查 buffer_version 是否已过时
  2. 如果匹配 → 替换 tree + 更新 HighlightCache
  3. 如果不匹配 → 丢弃，等待下一个结果
```

### 6.4 HighlightCache

```rust
pub struct HighlightCache {
    row_count: usize,
    captures: Vec<Vec<HighlightCapture>>,
    version: BufferVersion,
    color_map: ColorMap,
}

pub struct HighlightCapture {
    pub start: Offset,
    pub end: Offset,
    pub kind: HighlightKind,
}

pub enum HighlightKind {
    Keyword,
    String,
    Comment,
    Function,
    Type,
    Variable,
    Constant,
    Operator,
    Punctuation,
    // ... 按语言扩展
    Custom(String),  // 语言特定
}
```

`HighlightCache` 在重解析完成后重建，为每个行生成对应的 captures 列表。渲染层据此为 `RichTextElement` 的行内文本添加颜色样式。

### 6.5 ColorMap

```rust
pub struct ColorMap {
    rules: Vec<(HighlightKind, Style)>,
    default_style: Style,
}

pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub underline: Option<bool>,
}
```

ColorMap 将语义标记(tag)映射到具体样式，与主题系统联动：

```
Tree-sitter query capture  "keyword" "function.builtin" "string"
        │                      │           │               │
        ▼                      ▼           ▼               ▼
      HighlightKind        Keyword     Function        String
        │                      │           │               │
        ▼                      ▼           ▼               ▼
      ColorMap: semantic → style resolution with current theme
        │
        ▼
      Style(foreground: #569CD6, font_weight: Bold, ...)
```

### 6.6 Language/Query 注册

```rust
pub struct LanguageRegistry {
    languages: HashMap<String, LanguageInfo>,
}

pub struct LanguageInfo {
    pub name: String,
    pub grammar: Language,            // Tree-sitter grammar
    pub highlights_query: Query,      // 高亮查询
    pub injections_query: Option<Query>,  // 嵌入语言(HTML 中的 JS 等)
    pub file_extensions: Vec<String>,
    pub shebangs: Vec<String>,        // #!/usr/bin/env python
}
```

语言注册在启动时完成，grammar 编译为 WASM 或原生共享库。

---

## 7. LSP 集成

### 7.1 当前状态

Warp 编辑器**当前不存在 LSP 集成**。代码智能和分析能力主要依赖：

1. **Tree-sitter 语法解析**: 语法高亮、基础符号结构
2. **AI Agent 代码理解**: 通过 Agent 模式提供代码解释、重构建议等
3. **Cargo/Rust Analyzer 不集成**: 编辑器中不作类型检查或诊断

### 7.2 计划架构

未来的 LSP 集成将在 `crates/lsp`(已存在 LSP 客户端框架)基础上构建：

```
┌─────────────────────────────────────┐
│          Editor LSP 层               │
│  (app/src/editor/lsp/)              │
├─────────────────────────────────────┤
│            LSP 客户端                 │
│  (crates/lsp)                       │
│  ├── 协议类型定义(JSON-RPC)           │
│  ├── 请求/响应序列化                  │
│  ├── server 生命周期管理              │
│  └── 能力协商(initialize)             │
├─────────────────────────────────────┤
│          Tree-sitter 本地解析         │
│  (用于 LSP 不可用时的基础分析)          │
│  + 语法令牌                          │
│  + 符号导航(breadcrumb)              │
│  + 块折叠                            │
└─────────────────────────────────────┘
```

### 7.3 与 AI Agent 的分工

```
              代码智能分工
          ┌─────────────────┐
          │                 │
  语法层   │   Tree-sitter   │   ← 100% 本地，离线可用
          │   (高亮/折叠/大纲)│
          ├─────────────────┤
          │                 │
  语义层   │   LSP           │   ← 计划中，提供诊断/补全/跳转
          │   (诊断/补全/引用)│
          ├─────────────────┤
          │                 │
  意图层   │   AI Agent      │   ← 复杂理解/重构/解释
          │   (解释/重构/搜索)│
          └─────────────────┘
```

---

## 8. 补全引擎

补全引擎位于 `crates/warp_completer`，是 Warp 中负责所有补全场景的模块。

### 8.1 整体架构

```
用户输入
  │
  ▼
┌──────────────────┐
│    Parser         │   解析当前输入，提取上下文
│  (上下文提取器)    │   - 终端 shell 解析(PS1/Zsh/Bash/Fish)
│                   │   - 编辑器语法解析
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   Classifier      │   分类补全类型
│  (意图分类器)      │   - 命令名(Ssh, Git, FilePath, etc.)
│                   │   - 参数名/标志
│                   │   - 文件路径
│                   │   - 历史命令
│                   │   - 自然语言
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   Locator         │   定位补全源
│  (源定位器)        │   - CommandRegistry(已注册命令)
│                   │   - FileSystem(文件路径)
│                   │   - 历史记录
│                   │   - 自定义资源
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   Suggestion Gen  │   生成候选列表
│  (候选生成器)      │   - MatchStrategy 过滤
│                   │   - 排序/评分
│                   │   - 去重
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   Combiner        │   组合候选并展示
│  (展示组合)        │   - 合并多源结果
│                   │   - 最终排序/截断
│                   │   - 渲染到 UI
└──────────────────┘
```

### 8.2 CommandRegistry

```rust
pub struct CommandRegistry {
    commands: Vec<CommandSpec>,
    aliases: HashMap<String, String>,
}

pub struct CommandSpec {
    pub name: String,
    pub category: CommandCategory,
    pub arguments: Vec<ArgSpec>,
    pub subcommands: Vec<CommandSpec>,
    pub completion: CompletionStrategy,
}

pub enum CommandCategory {
    Builtin,       // cd, ls, echo
    Git,           // git commit, git push
    Ssh,           // ssh hostname
    Docker,        // docker run
    Npm,           // npm install
    Custom,        // alias 或自定义命令
    AiGenerated,   // AI 动态生成
}

pub enum CompletionStrategy {
    /// 由固定逻辑提供补全
    Static { candidates: Vec<String> },
    /// 动态生成(如文件列表)
    Dynamic(Box<dyn DynamicCompleter>),
    /// 委托给 shell 自身
    ShellDelegate,
    /// AI 根据上下文推断
    AiInferred,
}
```

### 8.3 CompletionLocation

```rust
pub struct CompletionLocation {
    pub current_word: String,           // 当前正在输入的内容
    pub preceding_text: String,         // 光标之前的所有文本
    pub cursor_offset: Offset,
    pub line_text: String,
    pub line_number: usize,

    // 解析后的结构
    pub parsed: Option<ParsedCommand>,

    // 终端上下文
    pub shell_type: Option<ShellType>,  // Bash/Zsh/Fish
    pub working_directory: Option<PathBuf>,
    pub environment: HashMap<String, String>,
}

pub struct ParsedCommand {
    pub command_name: String,
    pub args: Vec<String>,
    pub cursor_at_arg: usize,            // 光标在参数中的位置
    pub cursor_in_flag: Option<String>,  // 正在输入哪个 flag
    pub subcommand_path: Vec<String>,
}
```

### 8.4 MatchStrategy

```rust
pub enum MatchStrategy {
    /// 前缀匹配: "co" → "commit"
    Prefix,
    /// 子串匹配: "mit" → "commit"
    Substring,
    /// 模糊匹配: "cmt" → "commit"
    Fuzzy,
    /// 大小写敏感的精确匹配
    CaseSensitive,
    /// 智能(混合策略)
    Smart,
}
```

模糊匹配使用 `crates/fuzzy_match`：

```rust
/// 模糊评分: 返回匹配分和匹配位置
pub fn fuzzy_match(query: &str, candidate: &str) -> Option<(u32, Vec<usize>)> {
    // 算法: 基于 Smith-Waterman 的局部对齐
    // 评分:
    //   - 连续匹配 +10
    //   - 首字母匹配 +20
    //   - 边界匹配(如 /, _, 大写) +15
    //   - 不匹配 -2
}
```

### 8.5 v1 vs v2 特性

补全引擎通过 Cargo feature `v2` 控制两套实现：

```toml
# crates/warp_completer/Cargo.toml
[features]
default = ["v1"]
v1 = []
v2 = []
```

| 维度 | v1 | v2 |
|------|----|----|
| 架构 | 同步单线程 | 异步管道 |
| 补全源 | 静态命令 + 文件系统 | 动态 + AI + Shell Delegate |
| 分类器 | 简单字符串匹配 | ML/规则混合分类 |
| 命令注册 | 手动注册 | 自动扫描 + command-signatures-v2 |
| 多段回退 | 不支持 | 支持(shell → git → subcommand) |
| AI 补全 | 无 | AI Agent 参与 |

`command-signatures-v2` 是独立子项目，为 v2 补全提供命令结构元数据：

```rust
// command-signatures-v2 产出的数据结构示例
CommandSignature {
    name: "git push",
    args: [
        Arg {
            name: "remote",
            short: None,
            long: None,
            kind: ArgKind::Positional {
                completions: CompletionKind::GitRemote,
            },
        },
        Arg {
            name: "branch",
            short: None,
            long: None,
            kind: ArgKind::Positional {
                completions: CompletionKind::GitBranch,
            },
        },
        Arg {
            name: "force",
            short: Some('f'),
            long: Some("force"),
            kind: ArgKind::Flag,
        },
    ],
}
```

### 8.6 补全流程(以终端为例)

```
用户在终端输入: "git checkout feat"

1. Parser:
   - preceding_text: "git checkout feat"
   - current_word: "feat"
   - parsed.cursor_at_arg: 2 (第三个参数)

2. Classifier:
   - 命令 "git" → CommandCategory::Git
   - "checkout" → Git 子命令

3. Locator:
   - CommandRegistry.lookup("git checkout")
   - 获取 Branch 补全策略

4. Suggestion Generator:
   - 执行 `git branch --list` (或使用缓存)
   - 获取分支列表: ["main", "feature/login", "feature/search", "fix/bug"]
   - MatchStrategy::Smart 模糊匹配 "feat"
   - 评分:
     "feature/login"   → 85 (前缀匹配)
     "feature/search"  → 65 (前缀匹配)
     "main"            → 30 (不匹配)
     "fix/bug"         → 20 (不匹配)

5. Combiner:
   - 排序: ["feature/login", "feature/search"]
   - 截断(最多 20 条)
   - 渲染到 UI 补全弹出框
```

### 8.7 编辑器内补全

编辑器内补全(如代码补全、AI 对话补全)复用相同的管道架构，但使用不同的 Parser 和 Locator：

```
编辑器补全流程:
1. Parser: 使用 Tree-sitter AST 提取光标处的语法上下文
2. Classifier:
   - 是否在 import 语句中 → 模块路径补全
   - 是否在函数调用中 → 参数补全
   - 是否在赋值/声明中 → 变量/类型补全
   - 是否是自然语言 → AI Prompt 补全
3. Locator:
   - Tree-sitter 符号表
   - AI Agent 上下文
   - 用户 open tabs 中的符号
4. Suggestion Generator: 略
5. Combiner: 略
```

---

## 9. 代码评审

位于 `app/src/code_review/`。

### 9.1 架构总览

```
app/src/code_review/
├── mod.rs              // 模块入口，CodeReviewView 实体
├── diff_state.rs       // DiffStateModel — diff 状态管理
├── diff_parser.rs      // Git diff 文本解析
├── code_review_view.rs // 评审视图(UI 挂载点)
├── inline_comments.rs  // 内联评论
├── git_operations.rs   // Git 操作封装
└── code_review_tests.rs
```

```
┌──────────────────────────────────────────────┐
│              CodeReviewView                    │
│  (Entity, 顶层视图)                           │
├──────────────────────────────────────────────┤
│              DiffStateModel                    │
│  ├── files: Vec<ReviewedFile>                 │
│  ├── active_file: Option<usize>               │
│  ├── expanded_hunks: Set<HunkId>             │
│  └── comment_state: CommentState              │
├──────────────────────────────────────────────┤
│           Git 操作层                           │
│  ├── diff(git diff HEAD~1) → DiffOutput       │
│  ├── blame(file) → Vec<BlameLine>             │
│  └── show(commit) → CommitInfo                │
└──────────────────────────────────────────────┘
```

### 9.2 DiffStateModel

```rust
pub struct DiffStateModel {
    pub diff: Vec<DiffFile>,            // 所有变更文件
    pub active_file_index: usize,
    pub expanded_hunks: HashSet<usize>,
    pub comments: Vec<ReviewComment>,
}

pub struct DiffFile {
    pub old_path: String,              // a/file.rs
    pub new_path: String,              // b/file.rs
    pub status: FileStatus,            // Added/Modified/Deleted/Renamed
    pub hunks: Vec<DiffHunk>,
    pub is_expanded: bool,
}

pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed { from: String, to: String },
}
```

### 9.3 Git Diff 解析

```rust
pub struct DiffHunk {
    pub id: HunkId,
    pub header: String,                // "@@ -1,5 +1,7 @@ ..."
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
}

pub enum DiffLineKind {
    Context,    // 上下文(不变)
    Addition,  // 新增(前缀 +)
    Deletion,  // 删除(前缀 -)
    Header,    // hunk header
    NoNewline,  // no newline at end of file
}
```

解析算法:

```
原始 Git Diff 输出:
@@ -10,6 +10,7 @@
 fn foo() {
     let x = 1;
+    let y = 2;
-    let z = 3;
     return x;
 }

解析后:
DiffHunk {
    header: "@@ -10,6 +10,7 @@",
    old_start: 10, old_lines: 6,
    new_start: 10, new_lines: 7,
    lines: [
        DiffLine { kind: Context,  content: "fn foo() {",          old: 10, new: 10 },
        DiffLine { kind: Context,  content: "    let x = 1;",      old: 11, new: 11 },
        DiffLine { kind: Addition, content: "+    let y = 2;",     old: None, new: 12 },
        DiffLine { kind: Deletion, content: "-    let z = 3;",     old: 12, new: None },
        DiffLine { kind: Context,  content: "    return x;",       old: 13, new: 13 },
        DiffLine { kind: Context,  content: "}",                   old: 14, new: 14 },
    ]
}
```

### 9.4 CodeReviewView

```rust
pub struct CodeReviewView {
    pub diff_model: DiffStateModel,
    // UI 状态
    pub search_query: Option<String>,
    pub filter: ReviewFilter,
    pub view_mode: ReviewViewMode,
}

pub enum ReviewViewMode {
    /// 统一视图(标准的 side-by-side 或 unified)
    Unified,
    /// 文件列表(只显示文件名和状态)
    FileList,
    /// 只显示与自己相关的改动
    MyChanges,
}

impl CodeReviewView {
    pub fn load_diff(&mut self, base_ref: &str, ctx: &mut AppContext);
    pub fn toggle_hunk(&mut self, hunk_id: HunkId);
    pub fn add_comment(&mut self, line: DiffLine, text: String, ctx: &mut AppContext);
    pub fn resolve_comment(&mut self, comment_id: CommentId);
}
```

### 9.5 内联评论

```rust
pub struct ReviewComment {
    pub id: CommentId,
    pub file_path: String,
    pub hunk_id: HunkId,
    pub line_number: usize,        // 新文件行号
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub resolved: bool,
    pub replies: Vec<ReviewComment>,
}

pub struct CommentState {
    comments: Vec<ReviewComment>,
    draft_comments: HashMap<HunkId, String>,  // 未提交的草稿
    resolved_comments: HashSet<CommentId>,
}
```

评论与 diff 行绑定，当 diff 更新时通过 line_number 映射保持位置。

### 9.6 Git 操作

```rust
pub fn get_diff(
    base_ref: &str,
    head_ref: Option<&str>,
) -> Result<Vec<DiffFile>, GitError>;

pub fn get_blame(
    file_path: &Path,
    options: &BlameOptions,
) -> Result<Vec<BlameLine>, GitError>;

pub fn get_commit_info(
    commit: &str,
) -> Result<CommitInfo, GitError>;
```

底层通过 `std::process::Command` 调用 git（注意：必须使用 `crates/command` 代替直接调用，见工程纪律）。

### 9.7 数据流

```
装载 diff:
  CodeReviewView::load_diff("HEAD~1")
    → git operations::get_diff("HEAD~1")
      → 执行 git diff HEAD~1
      → 获取 raw diff text
    → diff_parser::parse(raw_diff)
      → Vec<DiffFile>
    → diff_state.store(diff_files)
    → 触发 UI 更新

用户展开文件:
  → DiffStateModel.expand_file(idx)
  → 标记 is_expanded = true
  → 触发 ViewHandle 更新的 Element 重新构建

用户添加评论:
  → CodeReviewView::add_comment(line, text)
    → ReviewComment { line_number, file, ... }
    → CommentState.draft_comments insert until saved
  → (保存) 序列化评论到持久化/云端
```

---

## 10. 搜索系统

位于 `app/src/search/`。

### 10.1 架构总览

```
app/src/search/
├── mod.rs                    // SearchMixer<T> 主入口
├── mixer.rs                  // SearchMixer 多源混合器
├── file_search_model.rs     // FileSearchModel (文件搜索)
├── search_item.rs           // SearchItem trait
├── query_filter.rs          // QueryFilter 系统
├── tantivy_backend.rs       // Tantivy 全文搜索(warp-served 模式)
├── results_view.rs          // 搜索结果视图
└── search_tests.rs
```

```
                   用户输入查询
                       │
                       ▼
               ┌───────────────┐
               │  QueryFilter   │  ← 解析过滤条件 type:file lang:rust
               │  (过滤解析)     │
               └───────┬───────┘
                       │
                       ▼
               ┌───────────────┐
               │  SearchMixer   │  ← 异步多源混合
               │  (混合调度器)   │
               └───┬───┬───┬───┘
                   │   │   │
          ┌────────┘   │   └────────┐
          ▼            ▼            ▼
    ┌──────────┐ ┌──────────┐ ┌──────────┐
    │ 文件搜索  │ │ 命令搜索  │ │ AI 历史  │ │ ...
    │ Tantivy  │ │ 模糊匹配  │ │ 对话搜索  │
    │ 全文索引  │ │ 命令注册表 │ │ 语义搜索  │
    └──────────┘ └──────────┘ └──────────┘
          │            │            │
          └────────────┼────────────┘
                       ▼
               ┌───────────────┐
               │  Combiner      │  ← 合并/排序/去重
               │  (结果合并)     │
               └───────┬───────┘
                       │
                       ▼
               ┌───────────────┐
               │  Results View  │  ← UI 展示
               │  (分组展示)     │
               └───────────────┘
```

### 10.2 SearchMixer<T>

`SearchMixer<T>` 是通用异步混合器，泛型参数 `<T: SearchItem>` 允许不同搜索类型复用同一框架：

```rust
pub struct SearchMixer<T: SearchItem> {
    sources: Vec<Box<dyn SearchSource<T>>>,
    debounce_ms: u64,            // 防抖时间(通常 150-300ms)
    max_results: usize,          // 最大总结果数
    per_source_limit: usize,     // 每个源最多返回数
}

pub trait SearchSource<T: SearchItem>: Send + Sync {
    fn name(&self) -> &str;
    fn search(
        &self,
        query: &str,
        filter: &QueryFilter,
    ) -> Pin<Box<dyn Future<Output = Vec<T>> + Send>>;
    fn priority(&self) -> u32;   // 优先级，决定排序权重
}

impl<T: SearchItem> SearchMixer<T> {
    pub fn search(
        &self,
        query: &str,
        filter: &QueryFilter,
    ) -> Vec<T> {
        // 1. 防抖(如果变化过快，取消上次未完成的搜索)
        // 2. 并行下发到所有 source
        // 3. 等待所有 source 完成(或超时 200ms)
        // 4. 合并结果
        // 5. 按评分排序
        // 6. 去重(相同 id)
        // 7. 截断到 max_results
    }
}
```

### 10.3 SearchItem trait

```rust
pub trait SearchItem: Clone + 'static {
    type Id: Eq + Hash + Clone;
    fn id(&self) -> Self::Id;
    fn score(&self) -> f32;
    fn display_text(&self) -> String;
    fn category(&self) -> SearchCategory;
    fn icon(&self) -> Option<IconName>;
    fn matches_query(&self, query: &str) -> bool;
}

pub enum SearchCategory {
    File,
    Command,
    AiHistory,
    Setting,
    Symbol,
    Documentation,
    Action,
}
```

具体实现：

```rust
// 文件搜索结果
pub struct FileSearchItem {
    pub path: PathBuf,
    pub score: f32,
    pub match_ranges: Vec<Range<usize>>,  // 匹配区域(用于高亮)
}
impl SearchItem for FileSearchItem {
    type Id = PathBuf;
    fn id(&self) -> PathBuf { self.path.clone() }
    fn score(&self) -> f32 { self.score }
    fn display_text(&self) -> String { self.path.to_string_lossy().to_string() }
    fn category(&self) -> SearchCategory { SearchCategory::File }
}

// 命令搜索结果
pub struct CommandSearchItem {
    pub name: String,
    pub description: String,
    pub category: CommandCategory,
    pub score: f32,
}
impl SearchItem for CommandSearchItem { /* 类似实现 */ }
```

### 10.4 Tantivy 全文搜索

在启用了 warp-server(云端)模式的部署中，文件搜索使用 Tantivy 全文索引：

```rust
pub struct TantivyBackend {
    index: tantivy::Index,
    reader: tantivy::IndexReader,
    searcher: tantivy::Searcher,
}

impl TantivyBackend {
    pub fn new(index_path: &Path) -> Result<Self, TantivyError> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("path", STRING | STORED);
        schema_builder.add_text_field("content", TEXT);
        schema_builder.add_text_field("extension", STRING);
        // ...
        let index = Index::open_in_dir(index_path)?;
        // ...
    }

    pub fn search(
        &self,
        query: &str,
        filter: &QueryFilter,
        limit: usize,
    ) -> Vec<FileSearchItem> {
        // 构建 Tantivy Query
        // 支持: 短语查询("foo bar")、布尔查询(AND/OR)、字段过滤
        // 结合 QueryFilter 限制文件类型、路径等
    }
}
```

本地模式(无云端)回退到 `crates/repo_metadata` 的文件树搜索 + `crates/warp_ripgrep` 的内容搜索。

### 10.5 FileSearchModel

```rust
pub struct FileSearchModel {
    // 来自 workspace 的文件索引
    file_tree: Arc<RwLock<FileTree>>,
    // Tantivy 后端(可选)
    tantivy: Option<TantivyBackend>,
    // 最近打开的文件(提升这些文件的权重)
    recent_files: LruCache<PathBuf, Instant>,
    // 搜索配置
    config: FileSearchConfig,
}

pub struct FileSearchConfig {
    pub max_results: usize,
    pub include_hidden: bool,
    pub include_gitignored: bool,
    pub fuzzy_threshold: f32,    // 默认 0.6
    pub debounce_ms: u64,
}
```

搜索流程：

```rust
impl FileSearchModel {
    pub fn search(&self, query: &str, filter: &QueryFilter) -> Vec<FileSearchItem> {
        if let Some(tantivy) = &self.tantivy {
            tantivy.search(query, filter, self.config.max_results)
        } else {
            // 本地模式: 在文件树上模糊匹配路径名
            self.search_file_tree(query, filter)
        }
    }

    fn search_file_tree(&self, query: &str, filter: &QueryFilter) -> Vec<FileSearchItem> {
        let tree = self.file_tree.read();
        let mut results: Vec<FileSearchItem> = tree
            .files()
            .filter(|f| filter.matches(f))
            .filter_map(|f| {
                let score = fuzzy_match(query, &f.path_string());
                score.map(|(s, ranges)| FileSearchItem {
                    path: f.path.clone(),
                    score: s,
                    match_ranges: ranges,
                })
            })
            .collect();

        // 提升最近打开文件的权重
        for item in &mut results {
            if self.recent_files.contains(&item.path) {
                item.score *= 1.2;
            }
        }

        // 排序并截断
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(self.config.max_results);
        results
    }
}
```

### 10.6 QueryFilter 系统

```rust
pub struct QueryFilter {
    pub file_type: Option<Vec<String>>,     // type:rs, type:toml
    pub directory: Option<PathBuf>,         // path:/src
    pub extension: Option<Vec<String>>,     // ext:rs, ext:js
    pub modified_after: Option<DateTime>,   // modified:today
    pub author: Option<String>,             // author:kevin
    pub case_sensitive: bool,
    pub regex: bool,                        // 正则搜索
}

impl QueryFilter {
    pub fn parse(query: &str) -> (String, QueryFilter) {
        // 从查询字符串中提取过滤语法
        // 例: "hello type:rs path:/src" → ("hello", Filter{file_type: ["rs"], path: ...})
        // 支持的语法:
        //   type:rs, type:toml, type:md
        //   path:/src/core, path:~/projects
        //   ext:rs, ext:jsx
        //   modified:today, modified:2024-01-01
        //   author:username
        //   case:true
        //   regex:true
    }

    pub fn matches(&self, file: &FileInfo) -> bool {
        // 检查文件是否满足所有过滤条件
    }
}
```

### 10.7 搜索去重与排序

```rust
impl<T: SearchItem> SearchMixer<T> {
    fn combine(mut results: Vec<T>) -> Vec<T> {
        // 1. 按 id 去重(保留最高分)
        results.sort_by(|a, b| a.id().cmp(&b.id()));
        results.dedup_by(|a, b| a.id() == b.id());

        // 2. 按评分降序
        results.sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap());

        // 3. 应用分组(如果 category 不同，保留跨类别的 top)
        // 4. 截断
        results.truncate(MAX_RESULTS);
        results
    }
}
```

### 10.8 交互流程

```
用户打开搜索(Cmd+P / Cmd+Shift+P):
  │
  ▼
搜索框获得焦点
  │
  ▼
用户输入 "hello world file:rs"
  │
  ▼
QueryFilter.parse("hello world file:rs")
  → query = "hello world", filter = {file_type: ["rs"]}
  │
  ▼
SearchMixer::search("hello world", filter)
  │
  ├── FileSearchSource  →  模糊匹配路径中的 "hello" 和 "world"
  ├── ContentSearchSource → Tantivy 内容搜索
  ├── CommandSearchSource → 命令名搜索
  │
  ▼ (200ms 后)
结果展示:
  ┌─────────────────────────────────────────────┐
  │ 文件 (3)                                      │
  │  > src/hello_world.rs       █████████████ 95 │
  │  > tests/world_test.rs      ██████████   80  │
  │  > examples/hello.rs        ██████       50  │
  │                                              │
  │ 内容 (5)                                      │
  │  > src/main.rs:42  println!("hello world");  │
  │  > src/lib.rs:17  // TODO: handle world      │
  │                                              │
  │ 命令 (1)                                      │
  │  > hello_command                             │
  └─────────────────────────────────────────────┘
```

---

## 11. 关键模式与工程纪律

### 11.1 实体句柄模式

编辑器、搜索视图、代码评审视图等所有长期存活的对象都通过 Entity-Handle 模式管理：

```
App
 └── entity_map: HashMap<EntityId, Box<dyn Any>>
      │
      ├── EntityId::Editor → EditorModel
      ├── EntityId::Search → SearchModel
      └── EntityId::CodeReview → CodeReviewModel

ViewHandle<T> 是弱引用:
  - 通过 EntityId 查找
  - 可 clone，允许多个消费者
  - 在 Entity 被销毁后自动失效
```

### 11.2 增量处理

所有模块都遵循"最小化增量更新"原则：

```
全量                       增量
──────────────────────────────────────────
Buffer 存储:   String            SumTree(共享子树)
渲染:          重新布局所有行     只重布局 dirty_lines
语法高亮:      全文重新解析       Tree-sitter 增量解析
搜索:          每次重新索引       文件监听器增量更新
撤销:          记录快照           记录 ReversibleAction
```

### 11.3 BufferVersion 协调

多个消费者依赖 BufferVersion 判断缓存有效性：

```
Buffer Version 1
  │
  ├── RenderState: 基于 V1 的 layout_lines ✓
  ├── SyntaxTreeState: 基于 V1 的 tree ✓
  ├── HighlightCache: 基于 V1 的 captures ✓
  │
  ▼
Edit → Buffer Version 2
  │
  ├── RenderState: 标记 dirty_lines，等待 relayout
  ├── SyntaxTreeState: apply_edit() 同步更新
  ├── HighlightCache: 标记为过期，等待后台重解析
  │
  ▼
后台解析完成 → Version 2 确认
  ├── SyntaxTreeState.tree 更新
  ├── HighlightCache 重建
  └── UI 触发重渲染
```

### 11.4 功能标志平台分裂

补全引擎和其他功能通过 FeatureFlag 控制行为：

```rust
// crates/warp_core/src/features.rs
pub enum FeatureFlag {
    // 补全 v2
    CompleterV2,
    // 编辑器特定功能
    EditorInlineAiCompletion,
    EditorCodeLens,
    // 搜索
    SearchTantivyBackend,
    SearchSemantic,
}
```

使用方式：
```rust
if FeatureFlag::CompleterV2.is_enabled() {
    // v2 补全逻辑
} else {
    // v1 回退
}
```

### 11.5 性能剖面

| 场景 | 延迟要求 | 优化手段 |
|------|----------|----------|
| 键盘输入 → 文本显示 | < 16ms (60fps) | SumTree O(log N)、增量渲染 |
| 语法高亮更新 | < 50ms | Tree-sitter 增量解析、异步后台 |
| 补全弹出 | < 100ms | 去抖、预缓存、增量索引 |
| 搜索(文件) | < 200ms | Tantivy 全文索引 + 模糊匹配阈值 |
| 搜索(全文) | < 500ms | Tantivy 倒排索引 + 异步 |
| 撤销/重做 | < 16ms | ReversibleAction 预计算、合并批次 |
| 代码评审 diff 加载 | < 1s | 分页加载、按需展开 hunk |

### 11.6 模块间依赖关系

```
crates/editor
  ├── crates/sum_tree       (Buffer 文本存储)
  ├── crates/syntax_tree    (语法高亮)
  ├── crates/markdown_parser (富文本 AST)
  └── crates/warpui_core    (Entity/ViewHandle)

crates/warp_completer
  ├── crates/fuzzy_match    (模糊匹配)
  ├── crates/command        (进程派生)
  └── crates/languages      (语言注册)

app/src/code_review
  ├── crates/editor         (diff 查看)
  ├── crates/command        (git 操作)
  └── crates/persistence    (评论持久化)

app/src/search
  ├── crates/repo_metadata  (文件树)
  ├── crates/warp_ripgrep   (内容搜索)
  ├── crates/warp_completer (命令搜索)
  └── crates/ai             (AI 对话搜索)
```

---

> **设计原则总结**: 编辑器、补全与搜索三模块共享相同的设计哲学——**分层抽象**(四层编辑器/管道补全/混合搜索)、**增量处理**(SumTree/增量解析/脏区标记)、**异步非阻塞**(后台语法解析/多源搜索混合/防抖合并)，以及 **Entity-Handle 生命周期管理**。三者都是"基础功能"而非"产品功能"，因此不使用 FeatureFlag 包裹，默认始终启用。
