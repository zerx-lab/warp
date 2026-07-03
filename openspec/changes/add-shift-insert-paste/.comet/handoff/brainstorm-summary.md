# Brainstorm Summary

- Change: add-shift-insert-paste
- Date: 2026-07-03

## Confirmed Technical Approach

添加一行 `FixedBinding::new("shift-insert", TerminalAction::Paste, id!("Terminal") & !id!("IMEOpen"))` 到 `app/src/terminal/view/init.rs`。

## Key Trade-offs and Risks

- Insert 键（无 Shift）已绑定为发送 `\x1b[2~`，Shift+Insert 是不同的键码，不会冲突
- macOS 键盘无 Insert 键，仅外接键盘时生效

## Testing Strategy

- cargo check 通过
- 手动验证粘贴行为

## Spec Patches

None
