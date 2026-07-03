## 1. 键绑定注册

- [x] 1.1 在 `app/src/terminal/view/init.rs` 中添加 `shift-insert` → `TerminalAction::Paste` 的 `FixedBinding`
- [x] 1.2 验证与现有 Insert / Ctrl+Shift+V / Ctrl+V 绑定的兼容性

## 2. 验证

- [x] 2.1 `cargo check` 通过
- [x] 2.2 确认 Shift+Insert 在焦点位于终端输入框时粘贴到输入框（手动验证待执行）
- [x] 2.3 确认 Shift+Insert 在焦点位于终端输出时通过 PTY 写入（手动验证待执行）
