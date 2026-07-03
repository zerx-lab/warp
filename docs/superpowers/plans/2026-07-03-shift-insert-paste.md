---
change: add-shift-insert-paste
design-doc: docs/superpowers/specs/2026-07-03-shift-insert-paste-design.md
base-ref: 5c145ccd27925c6710d6f6194a0f6817866dbc52
---

# Shift+Insert 粘贴快捷键实现计划

> **For agentic workers:** 单文件、单行改动。不需 subagent。

**目标:** 在终端视图注册 `shift-insert` → `TerminalAction::Paste` 固定键绑定。

**架构:** 在 `app/src/terminal/view/init.rs` 的 `init()` 函数中，在现有 `insert` 绑定之后添加一条 `FixedBinding`。不改粘贴逻辑、context predicate、feature flag、配置项。

**Tech Stack:** Rust, WarpUI keymap。

---

### 任务 1: 注册键绑定

**文件:**
- Modify: `app/src/terminal/view/init.rs:153-157`

- [ ] **1.1 在 `insert` 绑定后添加 `shift-insert` 绑定**

在 `app/src/terminal/view/init.rs` 第 157 行（`insert` 绑定的右大括号后加逗号）之后、第 158 行（`delete` 绑定）之前，插入：

```rust
        FixedBinding::new(
            "shift-insert",
            TerminalAction::Paste,
            id!("Terminal") & !id!("IMEOpen"),
        ),
```

- [ ] **1.2 `cargo check` 验证**

Run: `cargo check -p warp`
Expected: 编译通过，无警告。

- [ ] **1.3 提交**

```bash
git add app/src/terminal/view/init.rs
git commit -m "feat(terminal): add shift-insert paste keybinding"
```
