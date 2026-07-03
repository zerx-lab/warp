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
