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
