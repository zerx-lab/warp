# Comet Design Handoff

- Change: add-shift-insert-paste
- Phase: design
- Mode: compact
- Context hash: 59e4474b4b669d48c93db23b187e6f90d04d1cce4cf6dc972de207598a0c7247

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/add-shift-insert-paste/proposal.md

- Source: openspec/changes/add-shift-insert-paste/proposal.md
- Lines: 1-25
- SHA256: 959c3766749f99845aea5026369a8cdc050bc457d93b6f047d206bd4b00b3327

```md
## Why

终端用户在 shell 集成不可用（回退到普通模式）时，缺乏标准的 Shift+Insert 粘贴快捷键。Shift+Insert 是 xterm、gnome-terminal 等广泛采用的标准粘贴方式，Warp 应当支持这一通用约定，提升兼容性和用户体验。

## What Changes

- 在终端视图的固定快捷键绑定中注册 `shift-insert` → `TerminalAction::Paste`
- 不限平台（Linux / macOS / Windows），在所有平台上生效
- 不影响现有的 Ctrl+V / Ctrl+Shift+V / Cmd+Shift+V 粘贴快捷键
- 不改变粘贴逻辑本身

## Capabilities

### New Capabilities

无新增 capability。不涉及规范级行为变化。

### Modified Capabilities

无现有规范变更。

## Impact

- `app/src/terminal/view/init.rs` — 添加一行 `FixedBinding` 注册
- 仅限键绑定注册，无运行时行为变更，无需 feature flag
```

## openspec/changes/add-shift-insert-paste/design.md

- Source: openspec/changes/add-shift-insert-paste/design.md
- Lines: 1-48
- SHA256: b155667517265e16492fef64161cb3defa02717b8ace36591e9b4a63f683a053

```md
## Context

当前终端粘贴快捷键：

| 快捷键 | 范围 | 类型 |
|--------|------|------|
| Ctrl+Shift+V / Cmd+Shift+V | 所有平台 | FixedBinding |
| Ctrl+V | Windows 仅 | EditableBinding |
| Insert（无修饰键） | 所有平台 | 发送 `\x1b[2~` 到终端 |

缺少 Shift+Insert，这是 xterm、gnome-terminal、Windows Terminal 等终端广泛使用的标准粘贴快捷键。

绑定注册在 `app/src/terminal/view/init.rs` 的 `register_terminal_bindings()` 函数中，使用 `FixedBinding::new()` 模式。

## Goals / Non-Goals

**Goals:**
- 在所有平台上注册 `shift-insert` → `TerminalAction::Paste` 的固定快捷键绑定
- 与现有粘贴快捷键共存，互不冲突

**Non-Goals:**
- 不修改粘贴逻辑（`TerminalView::paste()`）
- 不修改 context_predicate 上下文判断
- 不需新增命令行/配置项/UI 选项
- 不涉及 feature flag

## Decisions

**方案：添加一行 FixedBinding**

```rust
FixedBinding::new(
    "shift-insert",
    TerminalAction::Paste,
    id!("Terminal") & !id!("IMEOpen"),
),
```

放置在 init.rs 中现有 Paste 绑定附近（`cmd_or_ctrl_shift("v")` 条目之后）。

**选择 FixedBinding 而非 EditableBinding 的原因：**
- Shift+Insert 是固定的标准终端行为（类似 NumPadEnter），用户不应也不需要重新绑定
- 与现有 `FixedBinding` 中的 Insert/Delete/PageUp 等键保持一致的注册风格

## Risks / Trade-offs

- **Insert 键兼容性**：无 Shift 的 `insert` 已绑定为发送 `\x1b[2~` 控制序列，Shift+Insert 是不同的键码，不会冲突
- **macOS 兼容性**：macOS 上 Insert 键通常不存在（Apple 键盘无 Insert 键）或通过 Fn 组合实现；Shift+Insert 绑定仅在使用全键盘或有 Insert 键的外接键盘时生效，不影响普通 macOS 用户体验
```

## openspec/changes/add-shift-insert-paste/tasks.md

- Source: openspec/changes/add-shift-insert-paste/tasks.md
- Lines: 1-10
- SHA256: 17399468e3d093d90dfa5d9d972dc1a216857df0f4cbf6c07fbf507ab46cb331

```md
## 1. 键绑定注册

- [ ] 1.1 在 `app/src/terminal/view/init.rs` 中添加 `shift-insert` → `TerminalAction::Paste` 的 `FixedBinding`
- [ ] 1.2 验证与现有 Insert / Ctrl+Shift+V / Ctrl+V 绑定的兼容性

## 2. 验证

- [ ] 2.1 `cargo check` 通过
- [ ] 2.2 确认 Shift+Insert 在焦点位于终端输入框时粘贴到输入框
- [ ] 2.3 确认 Shift+Insert 在焦点位于终端输出时通过 PTY 写入
```

