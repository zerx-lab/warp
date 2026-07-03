---
comet_change: add-shift-insert-paste
role: technical-design
canonical_spec: openspec
---

# Shift+Insert 粘贴快捷键设计

## 变更

在终端视图的固定快捷键绑定中注册 `shift-insert` → `TerminalAction::Paste`，支持所有平台。

## 实现

文件：`app/src/terminal/view/init.rs`，`register_terminal_bindings()` 函数。

在现有的 `cmd_or_ctrl_shift("v")` → `Paste` 绑定之后添加：

```rust
FixedBinding::new(
    "shift-insert",
    TerminalAction::Paste,
    id!("Terminal") & !id!("IMEOpen"),
),
```

## 不涉及

- 不改粘贴逻辑
- 不改 context predicate
- 不需 feature flag
- 不需配置项或 UI
