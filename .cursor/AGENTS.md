# AGENTS.md (Cursor)

> Navigation and engineering guide for AI agents working in the **Zap** repository.
>
> **Companion docs:** [Root AGENTS.md](../AGENTS.md) (full codebase map) · [Agentic AI Primer](../docs/agentic-ai-primer-for-maintainers.md) · [Agentic AI Investigation](../docs/agentic-ai-investigation.md) · [WARP.md](../WARP.md) (engineer handbook)

---

## Language (mandatory)

**Always use English** in this repository when acting as an agent:

- **User-facing responses** — write all chat replies, explanations, PR summaries, and commit messages in English.
- **Code comments** — write all new or edited comments, doc comments (`///`, `//!`), and test descriptions in English.
- **Do not** follow the Chinese-language convention in the root `AGENTS.md` §5.1; this `.cursor/AGENTS.md` overrides it for Cursor agents.

Existing Chinese comments in the codebase may remain unless you are editing that code for other reasons.

---

## What is this repo?

**Zap** is an open-source, **local-first** fork of [Warp](https://github.com/warpdotdev/warp) (formerly OpenWarp). Warp is the upstream commercial AI-native terminal; **this repo is Zap**, not Warp.

| Topic | Warp (upstream) | Zap (this repo) |
|-------|-----------------|-----------------|
| Account | Required for AI | **No Warp cloud required**; local auth stub |
| LLM calls | Warp cloud API | **BYOP** — user configures providers in Settings |
| API keys | Managed by Warp | Stored locally (OS keychain + `settings.toml` metadata) |
| Drive / cloud sync | Core product | Largely **removed or stubbed** |
| Cloud ambient agents | Supported | **Removed** in Wave 7 migrations |
| Telemetry | On by default | **Off by default** |
| Disable all AI | Settings toggle | **`is_any_ai_enabled()` always true** in Zap |

Search for comments tagged **`Zap Wave`** or **`Zap BYOP`** to find intentional fork changes.

---

## Agentic AI: start here

Most "agent doesn't work" bugs live in **Blocklist AI + BYOP**, not in future roadmap harness work.

### Mental model

A user types in **Agent Mode** at the bottom of a terminal tab. `BlocklistAIController` builds a request from conversation history and session context, runs a **BYOP readiness** check (`byop_readiness`), then `chat_stream` calls the user's LLM via **genai**. The model returns text and/or **tool calls** (shell, read files, MCP, etc.). `BlocklistAIActionModel` executes tools in the PTY, writes results back to history, and the controller may **auto-resume** another model turn.

```
User → terminal/input.rs → BlocklistAIController
  → byop_readiness (preflight)
  → chat_stream.rs → user's LLM (genai)
  → BlocklistAIActionModel (tools) → history → auto-resume
```

### Key files (fix these first)

| Role | Path |
|------|------|
| Orchestrator | `app/src/ai/blocklist/controller.rs` |
| LLM / BYOP adapter | `app/src/ai/agent_providers/chat_stream.rs` |
| Provider config → picker | `app/src/ai/agent_providers/mod.rs` |
| Readiness gate | `app/src/ai/byop_readiness/mod.rs` |
| Tool execution | `app/src/ai/blocklist/action_model.rs` |
| Settings schema | `app/src/settings/ai.rs` |
| Per-tab wiring | `app/src/terminal/view.rs` (~line 3180) |
| App init / flags | `app/src/lib.rs` |

### Agent surfaces: what to investigate when

| Surface | User-visible behavior | Investigate if… | Skip if… |
|---------|----------------------|-----------------|----------|
| **Blocklist AI + BYOP** | Agent input in terminal; AI blocks; tools | Default "agent doesn't work" | — **start here** |
| **Agent View** | Full-screen agent conversation panel | UI/list/navigation only | Simple prompts never get any response |
| **CLI tag-in / LRC** | Agent attached to long-running command | Only fails on LRC-style blocks | Hello prompt fails too |
| **`zap agent` CLI** | Scriptable headless agent | Only CLI path fails | GUI agent works |
| **Ambient agents** | Background tasks / notifications | Only background spawn fails | In-tab agent broken |
| **`ai_assistant` panel** | Legacy side-panel AI | Explicitly that panel | Agent Mode blocks |

### Failure buckets (classify before fixing)

| Bucket | Symptom | Likely modules |
|--------|---------|----------------|
| **A** | No AI block; provider/network error | `agent_providers`, `chat_stream`, settings |
| **B** | Text works; shell/file tools fail | `action_model/execute/*`, `permissions.rs` |
| **C** | First turn OK; second turn blocked | `controller.rs` preflight, `byop_readiness` |
| **D** | Only one surface broken | LRC, MCP, Agent View, restore |

### BYOP configuration checklist

1. Settings → AI has at least one `agent_providers` entry with valid `base_url`, `api_type`, and model id.
2. API key in keychain (or provider allows empty key, e.g. Ollama at `http://localhost:11434`).
3. Active model in `LLMPreferences` resolves via `lookup_byop()`.
4. Logs: filter by `[byop-readiness]`, `chat_stream`, `BlocklistAI`, `genai`.

**On disk (Linux):** config at `${XDG_CONFIG_HOME:-~/.config}/zap/`; MCP at `~/.zap/.mcp.json`.

### Known risk areas

1. **BYOP tool-result integrity** — `specs/byop-placeholder-tool-results/`: missing or out-of-order tool results block requests or confuse the model.
2. **Issue #94 task linearization** — regression tests in `chat_stream.rs` (`issue_94_task_linearization_tests`).

### Roadmap vs today

`docs/roadmap.md` describes a **future** standalone agent harness. **It does not exist yet.** Do not wait for it when fixing current BYOP/tool/readiness bugs.

Integration tests in `crates/integration/src/test/agent_mode.rs` mostly use dummy AI blocks, not real BYOP LLM calls.

---

## Repository overview

Zap is a Rust **agentic terminal / dev environment** built on WarpUI: terminal emulation, AI Agent, code review, completion, Notebook, settings, IPC, and more.

| Directory | Purpose |
|-----------|---------|
| `app/` | Main binary crate (`warp`); assembles subsystems, UI, migrations, platform glue |
| `crates/` | 67 workspace library crates |
| `command-signatures-v2/` | Standalone subproject (excluded from nextest) |
| `script/` | Bootstrap, build, presubmit scripts |
| `resources/` | Fonts, icons, shell scripts, shaders |
| `specs/` | Product/tech specs |
| `.agents/skills`, `.claude/skills` | Agent workflow skills |
| `lib/rust-genai/` | Vendored genai client for BYOP streaming |

**Licenses:** `warpui` / `warpui_core` → MIT; everything else → AGPL-3.0-only.

### Architecture layers (bottom → top)

```
Infrastructure: warp_core, warp_util, http_client, websocket, ipc, persistence, …
Framework:      warpui, warpui_core, editor, ui_components, sum_tree, syntax_tree
Product crates: ai, computer_use, vim, warp_completer, lsp, languages, …
App binary:     app/ — assembly, entry, platform glue, UI root
```

**Do not invert dependencies across layers.**

Key patterns (see `WARP.md`):

1. **Entity-Handle system** — `App` owns view/model entities; views reference via `ViewHandle<T>`.
2. **Element / Action** — declarative UI tree + action event system.
3. **Cross-platform** — macOS / Windows / Linux + WASM; platform code via `#[cfg(...)]`.
4. **AI** — Agent Mode in `app/src/ai/` (~389 files) and `crates/ai/`.
5. **Feature flags** — prefer runtime `FeatureFlag::*` over `#[cfg]`; defined in `crates/warp_core/src/features.rs`.

### AI-relevant layering

```
app/src/ai/           Product AI domain (largest subtree)
app/src/terminal/     PTY, blocks; TerminalView owns BlocklistAIController
app/src/settings/ai.rs AISettings, agent_providers, permissions
crates/ai/            Shared types: actions, conversions, api_keys
lib/rust-genai/       Vendored genai client for BYOP
```

**Rule of thumb:** UI and orchestration in `app/src/ai/`; portable primitives in `crates/ai/`; HTTP to LLM providers via `chat_stream.rs` + genai.

Each **terminal tab** constructs its own AI stack in `app/src/terminal/view.rs`:

- `BlocklistAIInputModel`, `BlocklistAIContextModel`, `BlocklistAIActionModel`
- `BlocklistAIController`, `BlocklistAIHistoryModel` (singleton)
- `AgentViewController` (optional)

---

## Engineering discipline

> Validation bar before PR: **`cargo check`**. Full handbook: `WARP.md`.

### Required conventions

- Use `rg` or workspace search tools for code search; reserve file reads for images/binaries.
- **Every line changed must trace to the user request** — no drive-by refactors, comment edits, or formatting.
- Prefer minimal, focused diffs; do not introduce abstractions for one-off use.
- Explain trade-offs when multiple approaches exist; do not silently choose for the user.
- Worktree path pattern: `.worktrees/<worktree_name>/`

### Rust style (from `WARP.md`)

- No redundant type annotations on closure parameters.
- Top-level `use` imports; avoid long path qualifiers (except inside `#[cfg]` branches).
- Context parameter named `ctx`, placed last; closure last if both exist.
- Delete unused parameters instead of prefixing with `_`; update call sites.
- Use inline format args: `"{x}"` not `"{}", x`.
- Avoid `_` wildcards in `match` unless truly needed; prefer exhaustive matching.
- Do not delete or rewrite existing comments unrelated to your change.

### Terminal model lock (critical)

- `TerminalModel::lock()` can deadlock (macOS beach ball).
- Before adding `model.lock()`, verify no caller already holds the lock; pass locked references down the stack.
- Minimize lock scope; never call functions that may re-lock while holding the lock.

### Feature flags

- Add variants in `crates/warp_core/src/features.rs`; wire into `DOGFOOD_FLAGS` / `PREVIEW_FLAGS` / `RELEASE_FLAGS` as needed.
- Prefer runtime `FeatureFlag::Xxx.is_enabled()` over `#[cfg]`.
- Gate whole features, not every call site; remove flags after stabilization.
- UI entry points must use the same flag as the code path.

### Database

- Diesel + SQLite.
- Schema changes require migrations in `migrations/` (`up.sql` / `down.sql`); do not hand-edit `app/src/persistence/schema.rs`.

### Tests

```sh
cargo check
cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2
```

Unit tests: `${filename}_tests.rs` or `mod_test.rs`, included via:

```rust
#[cfg(test)]
#[path = "filename_tests.rs"]
mod tests;
```

Agentic AI test filters:

```sh
cargo nextest run -p warp byop_readiness
cargo nextest run -p warp serializer_readiness
cargo nextest run -p warp issue_94
cargo nextest run -p warp smoke_
```

Build requires `protoc` (`protobuf-compiler` on Debian/Ubuntu).

### Subprocesses

- Never use `std::process::Command::new(...)` directly; use `crates/command` (Windows no-window handling).

### Subagents

- Split large tasks into non-overlapping write domains for parallel work.
- Simple tasks: do them directly; do not over-split.

---

## Quick reference

| Task | Starting point |
|------|----------------|
| Terminal grid / shell integration | `crates/warp_terminal/src/`, `app/src/terminal/` |
| Agent UI / conversation | `app/src/ai/` — grep `agent_*`, `conversation_*`, `blocklist`, `mcp` |
| BYOP / LLM streaming | `app/src/ai/agent_providers/chat_stream.rs` |
| Command completion | `crates/warp_completer/` (`--features v2`) |
| AI model / tool protocol types | `crates/ai/` |
| New settings | `crates/settings_value*`, `crates/settings`; UI in `app/src/settings_view/` |
| Feature flag | `crates/warp_core/src/features.rs` + usage sites |
| Persistence / schema | `migrations/` + `crates/persistence` |
| Cross-platform processes | `crates/command` |
| File search / watch | `crates/repo_metadata`, `crates/watcher`, `crates/warp_ripgrep` |

### Run and debug

```sh
./script/run                              # Build and launch zap-oss
RUST_LOG=debug ./script/run               # BYOP debug logging
./target/debug/zap-oss agent run --help   # Headless CLI agent
```

---

## Pre-change checklist

1. Which layer / crate / `app/src/<module>` does this belong to? Cross-layer?
2. Can an existing workspace dependency be reused?
3. Is this a product feature needing a Feature Flag?
4. Terminal model involved — is the lock already held upstream?
5. Subprocess — using `crates/command`?
6. Persistence — migration needed?
7. Tests added or updated?
8. Will `cargo check` pass?
9. Does every changed line map to the request?

---

## Related documentation

| Doc | Purpose |
|-----|---------|
| [docs/agentic-ai-primer-for-maintainers.md](../docs/agentic-ai-primer-for-maintainers.md) | Onboarding for agentic AI bug fixes |
| [docs/agentic-ai-investigation.md](../docs/agentic-ai-investigation.md) | Technical investigation map, hypothesis backlog |
| [AGENTS.md](../AGENTS.md) | Full codebase map (Chinese engineering notes) |
| [WARP.md](../WARP.md) | Commands, style, presubmit workflow |
| [docs/migrate-from-warp.md](../docs/migrate-from-warp.md) | On-disk paths for settings/MCP/skills |
| [specs/byop-placeholder-tool-results/](../specs/byop-placeholder-tool-results/) | BYOP tool-result integrity spec |
