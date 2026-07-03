# AI Agent 模块设计文档

> 本文档描述 Warp 中 AI Agent 模块的整体架构、核心数据流、关键模式及各部件的设计决策。覆盖 `crates/ai`（纯类型/协议层）与 `app/src/ai/`（UI/集成层，389 文件）两大区域。

---

## 1. 架构总览

AI Agent 模块采用**双层架构**，将纯协议/类型定义与 UI/产品集成严格分离：

```
┌──────────────────────────────────────────────────────────────┐
│                     app/src/ai/ (389 files)                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────────┐  │
│  │ Agent UI │ │Conv UI   │ │ Plan/Diff│ │ Artifacts      │  │
│  │ (会话面板)│ │(对话视图) │ │ (方案视图)│ │ (产物视图)     │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├────────────────┤  │
│  │ MCP Mgmt │ │ Tool UI  │ │Blocklist │ │Execution Prof. │  │
│  │ (MCP管理)│ │(工具UI)  │ │ (块列表) │ │ (执行配置)     │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├────────────────┤  │
│  │ Chip     │ │Voice     │ │ Tips     │ │ Coding Entry   │  │
│  │Configurat│ │(语音)    │ │ (提示)   │ │ (编码入口)     │  │
│  └──────────┘ └──────────┘ └──────────┘ └────────────────┘  │
│                       │   组装 / 转换                         │
│                       ▼                                      │
│  ┌────────────────────────────────────────────────────────┐  │
│  │           产品域胶水层 (集成/转换/状态管理)              │  │
│  │  ConversationManager, ToolExecutor, BlockContext,      │  │
│  │  AIAgentExchange, TaskTree, RequestParamsBuilder        │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────┬───────────────────────────────────────┘
                       │ 依赖（依赖倒置，crates/ai 不知晓 app）
                       ▼
┌──────────────────────────────────────────────────────────────┐
│                      crates/ai                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────────┐  │
│  │ LLM      │ │ Agent    │ │ Tool     │ │ Prompt         │  │
│  │ Clients  │ │ Protocol │ │ Registry │ │ Orchestration  │  │
│  │ (BYOP)   │ │ (类型)   │ │ (框架)   │ │ (Jinja2)      │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├────────────────┤  │
│  │ GenAI    │ │ MCP      │ │ Computer │ │ Proactive AI   │  │
│  │ Adapters │ │ Client   │ │ Use      │ │ (oneshot)      │  │
│  └──────────┘ └──────────┘ └──────────┘ └────────────────┘  │
│  纯类型 / 协议 / 客户端，无 UI 依赖，可独立测试               │
└──────────────────────────────────────────────────────────────┘
```

### 1.1 分层职责

| 层 | 职责 | 关键约束 |
|----|------|---------|
| `crates/ai` | LLM 客户端适配、Agent 协议类型、Tool 注册框架、MCP 客户端、Prompt 模板引擎 | 零 UI 依赖，纯 Rust 类型 + async trait |
| `app/src/ai/` | 产品集成：对话管理、Agent 生命周期编排、工具执行、块列表控制、MCP 管理器、视图绑定 | 知晓 `crates/ai`，反向不可 |

### 1.2 为什么是两层而非三层

- `crates/ai` 作为**纯协议/客户端层**，不依赖 WarpUI、不依赖 App 状态、不依赖持久化。这使得它可以在单元测试、wasm target、甚至外部工具中独立使用。
- `app/src/ai/` 承担**所有产品决策**：何时创建对话、如何展示流式响应、怎样把终端 Block 转为上下文、怎样持久化会话。
- 不存在中间的"AI 服务层"——产品胶水代码散在 `app/src/ai/` 各子模块中，通过 `ConversationManager` 等协调者串联。

---

## 2. 模型选择与调用（BYOP 架构）

### 2.1 BYOP（Bring Your Own Provider）

Warp Agent 模型调用的核心设计是 **BYOP**：用户自备 Provider API Key，Warp 作为通用客户端。与 Warp Cloud Agent（由服务端托管模型）并存：

```
                    ┌──────────────────┐
                    │   User Request   │
                    └────────┬─────────┘
                             │
                   ┌─────────▼─────────┐
                   │   Router / Dispatch│
                   └──┬─────────────┬──┘
                      │             │
              ┌───────▼──────┐ ┌────▼──────────┐
              │  BYOP Flow   │ │  Cloud Flow   │
              │ (本地/用户Key)│ │ (服务端托管)  │
              └───────┬──────┘ └────┬──────────┘
                      │             │
         ┌────────────▼────┐  ┌────▼────────────┐
         │  genai SDK      │  │  Warp Backend   │
         │  Adapter Layer  │  │  GraphQL/WS     │
         └────────┬────────┘  └─────────────────┘
                  │
        ┌─────────┼──────────┬──────────────┐
        ▼         ▼          ▼              ▼
   ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────┐
   │OpenAI  │ │Anthropic│ │Gemini  │ │ Ollama / etc │
   │API     │ │API     │ │API     │ │ (本地)       │
   └────────┘ └────────┘ └────────┘ └──────────────┘
```

### 2.2 GenAI SDK Adapter

`crates/ai` 提供统一的 genai 适配层，对 5 种协议类型进行路由：

```
LLMProvider 枚举:
  OpenAI    → https://api.openai.com/v1/chat/completions
  Anthropic → https://api.anthropic.com/v1/messages
  Gemini    → https://generativelanguage.googleapis.com/v1/...
  Ollama    → http://localhost:11434/api/chat
  Custom    → 用户配置的自定义 endpoint

LLMInfo 元数据:
  - 模型名称 (gpt-4o, claude-sonnet-4, gemini-2.5-pro)
  - 上下文窗口大小
  - 支持的工具调用类型（function/tool）
  - 支持的多模态（图像/PDF）
  - 价格/速率限制信息

LLMModelHost 枚举:
  - Zap（内部托管）
  - Claude（Anthropic）
  - Codex（GitHub）
  - Agents（自定义）
```

### 2.3 5 种 API 协议路由

Warp 并未直接使用各 SDK crate，而是自行实现了协议适配，原因是：

1. 需要统一的 `ResponseStream` 类型（`Stream<Item=LLMChunk>`）以支持流式 UI
2. 需要统一的错误处理/重试策略
3. 需要将不同 API 的工具调用格式归一为内部 `AIAgentActionType`
4. 需要支持 Jinja2 系统提示模板注入

```
LLMRequest (统一内部请求)
  ├── system_prompt: String (Jinja2 渲染后)
  ├── messages: Vec<LLMMessage>
  ├── tools: Option<Vec<OpenAiTool>> (归一化格式)
  ├── model: String
  ├── temperature/frequency_penalty/...
  │
  ▼
ProtocolAdapter trait
  ├── adapt_openai()    → POST chat/completions
  ├── adapt_anthropic() → POST messages
  ├── adapt_gemini()    → POST generateContent
  ├── adapt_ollama()    → POST /api/chat
  └── adapt_custom()    → 由用户配置的 URL/Headers
  │
  ▼
LLMResponse (统一内部响应)
  ├── content: String
  ├── tool_calls: Vec<AIAgentActionType>
  ├── finish_reason: String
  ├── usage: TokenUsage
  └── model: String
```

### 2.4 Proactive AI 子链路（oneshot.rs）

Proactive AI（自动检测终端命令意图并触发 AI 建议）是一个独立的子链路，使用 `oneshot.rs` 中的一次性调用逻辑，不经过完整的 Conversation 生命周期：

```
┌──────────────┐    ┌──────────────────┐    ┌──────────────┐
│ Input        │───▶│ Proactive        │───▶│ LLM OneShot  │
│ Classifier   │    │ Trigger Decision │    │ (无状态)     │
│ (终端输入)   │    │ (规则 + 模型)    │    │              │
└──────────────┘    └──────────────────┘    └──────┬───────┘
                                                   ▼
                                           ┌──────────────┐
                                           │ 显示 Quick    │
                                           │ Action / 提示 │
                                           └──────────────┘
```

关键区别：
- **完整 Agent**：有状态 Conversation，持久的 Tool Call 历史，支持多轮交互
- **Proactive AI**：无状态 one-shot，仅用于触发即时 UI 反馈

---

## 3. Agent 生命周期

### 3.1 核心类型

```
AIConversation              AIAgentExchange           Task
┌──────────────────┐       ┌──────────────────┐      ┌──────────────┐
│ id: UUID         │       │ id: UUID         │      │ id: UUID     │
│ exchanges: Vec<> │──1:N──│ conversation_id  │      │ parent: Opt  │
│ status: State    │       │ request: Params  │ 1:N──│ type: Enum   │
│ created_at       │       │ response: Stream │      │ status: Enum │
│ updated_at       │       │ tool_calls: Vec  │      │ optimistic:  │
│ metadata         │       │ status: State    │      │   bool       │
└──────────────────┘       │ error: Option    │      └──────────────┘
                           └──────────────────┘
```

### 3.2 状态机

所有实体共享同一个状态机：

```
                  ┌─────────────────┐
                  │     Pending     │
                  └────────┬────────┘
                           │
                  ┌────────▼────────┐
                  │   InProgress    │ ◄────────── 重试/继续
                  └──┬────┬────┬───┘
                     │    │    │
           ┌─────────┘    │    └──────────┐
           ▼              ▼               ▼
     ┌──────────┐  ┌──────────┐  ┌──────────────┐
     │ Success  │  │  Error   │  │  Cancelled   │
     └──────────┘  └──────────┘  └──────────────┘
                        │
                   ┌────▼────┐
                   │ Retry   │──▶ InProgress
                   └─────────┘
```

### 3.3 完整生命周期流程图

```
┌──────────────┐
│ 用户输入     │
│ (自然语言 /  │
│  命令 / 选中)│
└──────┬───────┘
       │
       ▼
┌──────────────────────────────┐
│ 1. Building RequestParams    │
│    ├── System Prompt (Jinja2)│
│    ├── Context (BlockContext)│
│    ├── User Rules (AIFact)   │
│    ├── Session Context       │
│    ├── Tool Definitions      │
│    └── Model Selection       │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ 2. Conversation Create/Resume│
│    ├── 新 Conversation       │
│    └── 已有 + 追加 Exchange  │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ 3. Send to LLM               │
│    ├── BYOP: genai adapter   │
│    ├── Cloud: graphql ws     │
│    └── ResponseStream 开始   │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐     ┌─────────────────┐
│ 4. ResponseStream Processing │────▶│ Stream Chunks   │
│    ├── Text Chunks → UI      │     │ (SSE-like)      │
│    ├── Tool Call Chunks      │     └─────────────────┘
│    └── Finish Reason         │
└──────────┬───────────────────┘
           │
           ▼ (若需要工具调用)
┌──────────────────────────────┐
│ 5. Tool Execution            │
│    ├── Resolve Tool          │
│    ├── Execute (同步/异步)   │
│    ├── Capture Result        │
│    └── Append ToolCallResult │
└──────────┬───────────────────┘
           │
           ▼ (循环：发回给 LLM)
┌──────────────────────────────┐
│ 6. LLM Resubmit              │
│    ├── ToolCallResult 作为   │
│    │   新消息追加            │
│    └── 回到 Step 3          │
└──────────┬───────────────────┘
           │
           ▼ (达到终止条件)
┌──────────────────────────────┐
│ 7. Finalize Exchange         │
│    ├── 标记 Success/Error    │
│    ├── 更新 Task 树          │
│    ├── 触发 Notifications    │
│    └── 保存到持久化          │
└──────────────────────────────┘
```

### 3.4 Task 树（Optimistic → Server）

Warp Agent 使用 Task 树结构来管理多步骤目标。Task 类型包括：

```
TaskType 枚举:
  ┌─────────────────────────────────────────────┐
  │ Goal         → 用户提出的高级目标            │
  │ SubGoal      → 分解后的子目标                │
  │ FileRead     → 读取文件                      │
  │ FileEdit     → 编辑文件                      │
  │ ShellCommand → 执行 shell 命令               │
  │ CodeChange   → 代码变更                      │
  │ Search       → 搜索                          │
  │ AskUser      → 向用户提问                    │
  │ Approval     → 需要用户批准                  │
  └─────────────────────────────────────────────┘
```

**Optimistic → Server 转换模式**：

```
创建时 (Optimistic):             确认后 (Server):
┌────────────────────┐          ┌────────────────────┐
│ Task {             │          │ Task {             │
│   id: "local-u1"   │          │   id: "server-id"  │
│   optimistic: true │  ──────▶ │   optimistic: false│
│   type: FileEdit   │  同步    │   type: FileEdit   │
│   status: Pending  │          │   status: InProg   │
│ }                  │          │   content: { ... } │
└────────────────────┘          └────────────────────┘
```

设计动机：
1. **即时反馈**：用户发出指令后，UI 立即显示出 Optimistic Task，无需等待服务端往返
2. **去重**：在服务端返回前，本地的 Optimistic Task 可以用作去重键
3. **冲突处理**：如果服务端返回了不同的 Task 结构，会替换 Optimistic 版本

---

## 4. 工具调用框架

### 4.1 AIAgentActionType — 20+ 工具枚举

`AIAgentActionType` 是 Agent 可调用的所有工具的单一枚举，定义在 `crates/ai` 中：

```
AIAgentActionType 枚举（核心子集）:
├── Shell           → 执行 shell 命令 (同步)
├── LongShell       → 执行长时间 shell 命令 (流式)
├── ReadFile        → 读取文件
├── WriteFile       → 写入文件
├── EditFile        → 编辑文件 (diff 格式)
├── Search          → 代码搜索
├── ReadDirectory   → 读取目录
├── Glob            → 文件模式匹配
├── Grep            → 内容搜索
├── ComputerUse     → 屏幕截图/点击/键入 (crates/computer_use)
├── WebFetch        → 抓取网页
├── WebSearch       → 搜索网络
├── MemorySet       → 设置 AI 记忆 (User Rules)
├── MemoryGet       → 读取 AI 记忆
├── AskUser         → 向用户提问
├── TaskComplete    → 标记任务完成
├── CreateNotebook  → 创建 Notebook
├── CreateWorkflow  → 创建工作流
├── CreateFile      → 创建文件
├── DeleteFile      → 删除文件
└── RunCode         → 运行代码 (特定语言)
```

### 4.2 AIAgentActionResultType

每个 Action 对应一个 Result 类型：

```
AIAgentActionResultType:
├── ShellResult        → stdout/stderr/exit_code
├── FileReadResult     → content + metadata
├── FileWriteResult    → path + size
├── EditFileResult     → diff + applied_lines
├── SearchResult       → matches[]
├── GrepResult         → matches with line numbers
├── WebResult          → content (markdown)
├── MemoryResult       → fact_key + value
├── AskUserResult      → user_response
├── ComputerUseResult  → screenshot/success
└── Error              → error message + details
```

### 4.3 BYOP Tool Adapters — OpenAiTool Registry 模式

对于 BYOP 调用，工具必须适配为 LLM 提供商理解的格式。Warp 使用 **OpenAiTool Registry 模式**：

```
                         LLM Provider
                             ▲
                             │  tool_calls format
                             │  (JSON Schema)
                             │
                ┌────────────┴────────────┐
                │  ToolAdapter<T: Tool>    │
                │  ├── to_openai_tool()   │
                │  │   → name, desc,      │
                │  │     parameters (JSON  │
                │  │     Schema)          │
                │  └── from_openai_call() │
                │      → parsed_args       │
                └────────────┬────────────┘
                             │
                ┌────────────┴────────────┐
                │  ToolRegistry           │
                │  ├── register::<T>()    │
                │  ├── all_tools()        │
                │  │   → Vec<OpenAiTool>  │
                │  ├── execute(name, args)│
                │  └── resolve(name)      │
                └─────────────────────────┘
```

关键设计决策：**所有工具共用同一个 OpenAI `function` schema 格式**，无论底层 LLM 是 Anthropic (tools) 还是 Gemini (functionDeclarations)。注册时每个工具只定义一个 OpenAiTool，适配层在各协议里做转换。

### 4.4 工具注册/发现/执行流程

```
┌──────────────┐
│ 注册阶段      │
│ (启动时)     │
│ ToolRegistry  │
│ .register()  │ ◄── 每个工具模块自注册
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ 发现阶段      │
│ (请求组装时) │
│ registry     │
│ .all_tools() │──▶ 根据 RequestParams 过滤
│ .filter()    │    (用户权限/功能 flag/模型能力)
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ 序列化阶段    │
│ (发送给 LLM) │
│ tool_adapter │
│ .to_openai() │──▶ JSON Schema 数组
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ 反序列化阶段  │
│ (LLM 返回    │
│  tool_call)  │
│ tool_adapter │
│ .from_openai│──▶ typed_args
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ 执行阶段      │
│ (调用系统)   │
│ executor     │
│ .execute()   │──▶ 异步执行 + 流式输出
└──────────────┘
```

---

## 5. MCP 集成

### 5.1 MCP 架构总览

Warp 通过 Model Context Protocol (MCP) 连接外部工具/数据源。MCP 集成涉及三个管理层 + 一个协议层：

```
┌───────────────────────────────────────────────────────────┐
│                    app/src/ai/ (管理层)                    │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ TemplatableMCPServerManager                         │  │
│  │ ● 管理模板化的 MCP Server 配置                       │  │
│  │ ● 支持变量替换（如 {{workspace_dir}}）               │  │
│  │ ● 状态：Stopped / Starting / Running / Error        │  │
│  └──────────────────────┬──────────────────────────────┘  │
│                         │                                   │
│  ┌──────────────────────▼──────────────────────────────┐  │
│  │ FileBasedMCPManager                                │  │
│  │ ● 从配置文件/目录发现 MCP Server                    │  │
│  │ ● 管理 Zap/Claude/Codex/Agents 各 Provider          │  │
│  │ ● 合并去重                                        │  │
│  └──────────────────────┬──────────────────────────────┘  │
│                         │                                   │
│  ┌──────────────────────▼──────────────────────────────┐  │
│  │ MCPProvider 枚举                                    │  │
│  │ ├── Zap     → 本应用创建/MCP Server                  │  │
│  │ ├── Claude  → Claude Desktop 配置文件               │  │
│  │ ├── Codex   → GitHub Codex MCP                     │  │
│  │ └── Agents  → 自定义 MCP Server                     │  │
│  └─────────────────────────────────────────────────────┘  │
└──────────────────────────┬────────────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────────────┐
│                    crates/ai (协议层)                      │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ ReconnectingPeer                                   │  │
│  │ ● 基于 rmcp (rust MCP client) 的连接管理           │  │
│  │ ● 自动重连 + 指数退避                               │  │
│  │ ● 健康检查 heartbeat                               │  │
│  └──────────────────────┬──────────────────────────────┘  │
│                         │                                   │
│  ┌──────────────────────▼──────────────────────────────┐  │
│  │ MCP Tool Discovery → BYOP Injection                 │  │
│  │                                                    │  │
│  │  MCP Server ──list_tools──▶ Vec<MCPTool>           │  │
│  │                              │                     │  │
│  │                              ▼                     │  │
│  │                    OpenAiTool (注册到 ToolRegistry)  │  │
│  │                              │                     │  │
│  │                    LLM 调用 ──call_tool──▶ MCP 执行 │  │
│  └─────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────┘
```

### 5.2 MCP 连接生命周期

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Stopped  │───▶│ Starting │───▶│ Running  │───▶│ Stopped  │
└──────────┘    └──────────┘    └────┬─────┘    └──────────┘
       ▲                             │
       │                    ┌────────▼────────┐
       │                    │  Reconnecting   │◄── 网络中断/异常
       │                    │  (指数退避)     │
       │                    └────────┬────────┘
       │                             │
       └─────────────────────────────┘
                ┌──────────┐
                │  Error   │──▶ 超过重试次数
                └──────────┘
```

### 5.3 MCP Tool → BYOP 注入

MCP 工具不能直接被 LLM 调用，必须注入到 ToolRegistry 中：

```
MCP Server
    │
    ▼  list_tools()
┌──────────────────────┐
│ MCPTool {            │
│   name: "weather"    │
│   description: "..." │
│   inputSchema: {...} │
│ }                    │
└──────────┬───────────┘
           │
           ▼  transform
┌──────────────────────┐
│ OpenAiTool {         │
│   function: {        │
│     name: "mcp__weather"  ← 命名空间前缀避免冲突
│     description: "..."    ← 来自 MCPTool
│     parameters: {...}     ← 来自 inputSchema
│   }                   │
│ }                     │
└──────────┬───────────┘
           │
           ▼  register
┌──────────────────────┐
│ ToolRegistry {       │
│   mcp_tools:         │
│   HashMap<name, fn>  │
│ }                    │
└──────────┬───────────┘
           │
           ▼  LLM 调用时
┌──────────────────────┐
│ execute("mcp__weather",│
│         { city: "..." })│
└──────────┬───────────┘
           │
           ▼  route
┌──────────────────────┐
│ MCPAdaptor.call_tool │
│ (name, args)         │
└──────────────────────┘
```

### 5.4 rmcp（Rust MCP Client）

`crates/ai` 封装了 Rust MCP 协议实现：

- 传输层：支持 stdio（子进程）和 SSE（远程）两种传输
- 协议层：基于 rmcp crate 的 `ClientHandler` trait
- 管理层：`ReconnectingPeer` 提供自动重连、心跳、状态上报

---

## 6. 对话与上下文管理

### 6.1 消息类型体系

Warp Agent 的对话消息包含丰富的类型体系，在 `crates/ai` 中定义：

```
对话消息类型:

输入消息 (用户 → Agent):
├── UserQuery        → 用户自然语言输入
├── UserCommand      → 用户直接命令 (/tool)
├── SystemMessage    → 系统注入的消息
├── ConversationResume → 继续历史对话

中间消息 (Agent 内部 / LLM ↔ 系统):
├── ToolCall         → LLM 请求调用工具
├── ToolCallResult   → 工具执行结果
├── UpdateTodos      → 更新 Task 树
├── StatusUpdate     → 状态变更通知
└── AgentThought     → Agent 内部推理过程

输出消息 (Agent → 用户):
├── TextResponse     → 文本回复
├── ToolResultView   → 工具结果渲染
├── PlanView         → 方案视图
├── DiffView         → 代码差异视图
├── ArtifactView     → 产物视图
├── ErrorMessage     → 错误信息
└── AskUserView      → 需要用户输入的提示
```

### 6.2 BlockContext — 终端上下文

终端集成上下文的核心类型是 `BlockContext`，它将已执行的终端 Block 结构化为 LLM 可理解的格式：

```
BlockContext 结构:
┌─────────────────────────────────────────────────────┐
│ BlockContext {                                       │
│   working_directory: Path,                           │
│   shell: String,  (e.g., "zsh", "bash", "fish")     │
│   prompt: String,                                    │
│   commands: Vec<CommandBlock>,                       │
│ }                                                    │
│                                                      │
│ CommandBlock {                                       │
│   index: usize,                                      │
│   input: String,      (用户输入的命令)              │
│   output: String,     (stdout + stderr 混合)        │
│   exit_code: i32,                                   │
│   duration: Duration,                                │
│   timestamp: DateTime,                               │
│   working_dir: Path,                                 │
│ }                                                    │
└─────────────────────────────────────────────────────┘
```

创建路径：`BlockContext::from_completed_block()` — 从一个已完成的终端 Block 构建上下文。

### 6.3 Session Context

Session Context 是跨 Conversation 的会话级上下文，包含：

```
SessionContext:
├── current_working_directory
├── recent_blocks (最近 N 个终端 Block)
├── environment_variables (选中的 env vars)
├── git_state (当前分支/未提交变更摘要)
├── opened_files (编辑器打开的文件的路径列表)
├── terminal_selection (当前选中的终端文本)
└── recent_errors (最近的错误输出摘要)
```

### 6.4 User Rules（AIFact 记忆）

User Rules 是基于键值对的持久化 AI 记忆系统，通过 `MemorySet`/`MemoryGet` 工具操作：

```
┌────────────────────────────────────┐
│ AIFact {                           │
│   key: String,                     │
│   value: String,                   │
│   category: Enum,                  │
│   ├── Preference   (用户偏好)       │
│   ├── Constraint   (约束/规则)      │
│   ├── Context      (上下文事实)     │
│   └── Custom       (自定义)        │
│   source:           (来源)          │
│   created_at: DateTime,            │
│   updated_at: DateTime,            │
│ }                                  │
└────────────────────────────────────┘

在 RequestParams 构建时注入:
  UserRules → 格式化为 System Prompt 的 "User Rules" 段落
```

### 6.5 BYOP Compaction（上下文窗口压缩）

当 BYOP 对话超过模型的上下文窗口时，执行压缩策略：

```
Compaction 策略（可配置）:
├── TruncateOldest
│   丢弃最早的消息对（UserQuery + Response），
│   保留 System Prompt 和最近的对话
│
├── SummarizeMid
│   对中间部分对话使用 LLM 进行一次凝练摘要，
│   用摘要消息替换原始消息序列
│
├── DropToolCallDetails
│   保留 ToolCall 记录但丢弃完整的 ToolCallResult body，
│   仅保留摘要（exit_code + output 前 N 字符）
│
└── KeepSystemOnly
    极端情况下仅保留 System Prompt + 最近一次 Exchange
```

---

## 7. 终端集成

### 7.1 BlockContext 构建时序

```
终端 Block 完成
      │
      ▼
┌───────────────────────────────────┐
│ Block 状态: Finished/Running      │
│ 在 ShellIntegration 中注册        │
└───────────────┬───────────────────┘
                │
                ▼ (用户触发 AI / Proactive)
┌───────────────────────────────────┐
│ BlockContext::from_completed_block│
│  ├── 提取 command + output        │
│  ├── 添加 exit_code / duration    │
│  ├── 注入环境元数据               │
│  └── 返回 BlockContext            │
└───────────────┬───────────────────┘
                │
                ▼
┌───────────────────────────────────┐
│ 组装到 RequestParams.system_prompt│
│ 格式:                             │
│ "以下是终端上下文中刚刚发生的事情: │
│  $ cd /project/src                │
│  $ cargo build                    │
│  [exit: 0, 2.3s]                  │
│  输出: Compiling...               │
│        error[E0308]: ..."         │
└───────────────────────────────────┘
```

### 7.2 Running Command Context

当有命令正在运行时，Agent 可以感知并避免冲突：

```
RunningCommandContext:
├── command: String
├── pid: u32
├── started_at: DateTime
├── working_dir: Path
└── shell: String

在工具执行时检查:
  1. 如果 Agent 要执行的 Shell 命令与当前运行命令
     在同一终端，发出警告
  2. 如果 Agent 要写入当前运行命令正在使用的文件，
     发出警告
  3. 如果用户确认，强制终止运行命令并执行
```

### 7.3 三类终端工具

```
┌──────────────────────────────────────────────────────────┐
│ Shell 工具（短命令）                                      │
│ ├── 适用: ls, git status, grep, cat, cargo check          │
│ ├── 执行方式: 同步，捕获完整 stdout/stderr                │
│ ├── 超时: 30s                                            │
│ └── 展示: 在 Agent 回复内联显示                           │
├──────────────────────────────────────────────────────────┤
│ Long Shell 工具（长命令）                                  │
│ ├── 适用: cargo build, npm test, python script.py         │
│ ├── 执行方式: 异步 PTY，流式输出回 UI                     │
│ ├── 超时: 600s（可配置）                                  │
│ └── 展示: 在 Agent 面板内嵌入终端视图                     │
├──────────────────────────────────────────────────────────┤
│ 文件编辑工具 (FileEdit / WriteFile)                       │
│ ├── 适用: 代码修改/创建                                   │
│ ├── 执行方式: 直接写文件系统，产生 Diff                    │
│ ├── 特殊: 需要用户批准（可配置）                          │
│ └── 展示: 在 Agent 面板内嵌入 Diff 视图                   │
└──────────────────────────────────────────────────────────┘
```

### 7.4 Blocklist 控制器

Agent 操作会产出一系列关联的终端 Block，由 Blocklist 控制器统一管理：

```
Agent 发起的操作:
  Shell("cargo build")
    │  ┌──────────────────────────┐
    ├─▶│ Block { type: Agent,     │
    │  │   status: Running,       │
    │  │   agent_exchange_id,     │
    │  │   command: "cargo build" }│
    │  └──────────────────────────┘
    │
    ▼ (完成后)
  FileEdit("src/main.rs")
    │  ┌──────────────────────────┐
    └─▶│ Block { type: Agent,     │
       │   status: Done,          │
       │   action_type: FileEdit, │
       │   diff: "...@@ ... @@"}   │
       └──────────────────────────┘

Blocklist 提供的能力:
  ├── 批量撤销：撤销一次 Agent Exchange 的所有变更
  ├── 状态追踪：显示哪些 Block 是 Agent 创建的
  ├── 差异高亮：Agent 修改的文件显示 Diff Marker
  └── 冲突检测：Agent 创建的 Block 与手动编辑冲突时告警
```

---

## 8. 关键模式

### 8.1 Tool Registry Pattern

工具注册模式是 `crates/ai` 的核心设计模式：

```
特征:
  1. 注册时提供 OpenAiTool 格式的 schema（唯一格式）
  2. 执行时接收 JSON Value，返回 JSON Value（序列化无关）
  3. 工具通过 trait 对象擦除具体类型

impl ToolRegistry {
    pub fn register<T: Tool>(&mut self, tool: T);
    pub fn all_tools(&self) -> Vec<OpenAiTool>;
    pub fn execute(&self, name: &str, args: Value) -> Result<Value>;
}

trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn schema(&self) -> OpenAiTool;
    async fn call(&self, args: Value) -> Result<Value>;
}
```

优点：
- **单一注册点**：新增工具只需 `registry.register(MyTool)`
- **协议无关**：`OpenAiTool` 是规范格式，Anthropic/Gemini 适配器负责转换
- **MCP 注入兼容**：MCP 工具适配为相同的 `Tool` trait 后注册

### 8.2 Optimistic → Server 转换

Task 的乐观更新模式：

```
1. 用户发出 "修改 src/main.rs 中的函数签名"
2. 本地立即创建 Optimistic Task:
     Task { id: "tmp-001", type: FileEdit, optimistic: true, status: Pending }
3. UI 立即显示一个待处理的编辑任务
4. Agent 经过 LLM 往返后，返回确认的 Task:
     Task { id: "sv-abc", type: FileEdit, optimistic: false, content: { diff: ... } }
5. 本地查找 "tmp-001"，替换为服务端版本
6. 若 LLM 拒绝了该操作，删除 Optimistic Task 并显示原因
```

设计考量：
- Optimistic ID 使用 UUID v4 + 客户端前缀，不可能冲突
- 本地在 Optimistic 状态下禁用重复提交
- 服务端返回的 Task 可能合并/拆分多个 Optimistic Task

### 8.3 双向 Protobuf 转换

AIConversation/AIAgentExchange/Task 等核心类型在本地（Rust struct）和服务端（Proto）之间存在双向转换：

```
Rust Struct                  Protobuf Message
┌──────────────┐          ┌──────────────────┐
│ AIConversation│ ──to──▶│ Conversation     │
│ .id          │  proto  │ .id              │
│ .exchanges   │         │ .exchange_ids[]  │
│ .status      │         │ .status          │
│ .metadata    │         │ .metadata (json) │
└──────────────┘          └──────────────────┘
       ◀──from──
        proto

转换要点:
  - 保证前向/后向兼容（Protobuf 的字段编号）
  - 枚举映射：Rust enum → Proto enum，处理 unknown variant
  - 时间戳：DateTime<Utc> ↔ google.protobuf.Timestamp
  - 嵌套类型：递归转换 Exchange → ExchangeProto
  - 大型字段（如 ToolCallResult body）：使用 field_mask 控制传输量
```

### 8.4 Jinja2 模板系统提示

系统提示（System Prompt）使用 Jinja2 模板引擎渲染，而非字符串拼接：

```
模板文件结构:
  crates/ai/src/prompt/templates/
  ├── base_system.jinja2       ← 基础系统提示
  ├── tools.jinja2             ← 工具定义注入
  ├── block_context.jinja2     ← 终端上下文注入
  ├── user_rules.jinja2        ← User Rules 注入
  ├── session_context.jinja2   ← 会话上下文注入
  ├── agent_role.jinja2        ← Agent 角色定义
  └── compaction.jinja2        ← 压缩摘要模板

渲染流程:
  1. Render base_system.jinja2
  2. Render agent_role.jinja2 → 嵌入 base
  3. Render tools.jinja2 → 嵌入 base
  4. Render block_context.jinja2 + user_rules.jinja2
     + session_context.jinja2 → 嵌入 base
  5. 最终拼装为完整的 system_prompt 字符串

示例 (block_context.jinja2):
---
{% if block_context %}
## 终端上下文

用户当前在 `{{ block_context.working_directory }}` ({{ block_context.shell }})。

最近的命令执行:
{% for cmd in block_context.commands %}
  $ {{ cmd.input }}
  [退出码: {{ cmd.exit_code }}, 耗时: {{ cmd.duration }}]
  {% if cmd.output %}
  ```
  {{ cmd.output | truncate(max_length=1000) }}
  ```
  {% endif %}
{% endfor %}
{% endif %}
---
```

设计理由：
- **关注点分离**：每个上下文模块有独立的模板文件
- **条件注入**：只有在有 BlockContext 时才渲染对应的段落
- **格式控制**：Jinja2 filter 可以控制输出长度、转义格式
- **可测试性**：可以为每个模板写独立的单元测试

### 8.5 BYOP-Only vs Cloud 共存

Warp Agent 同时支持 BYOP（用户自备模型）和 Cloud（服务端托管模型）两种模式，它们的核心差异：

| 维度 | BYOP-Only | Cloud |
|------|-----------|-------|
| 模型来源 | 用户提供的 API Key + endpoint | Warp 后端托管的模型 |
| 工具执行 | 本地执行全部工具 | 本地执行 + 部分服务端执行 |
| Task 树 | 全本地管理 | 本地 Optimistic + 服务端确认 |
| MCP | 本地 MCP Server | 本地 MCP + 服务端 MCP |
| 上下文 | 本地构建 + BYOP Compaction | 本地构建 + 服务端管理 |
| 持久化 | 本地 SQLite | 本地 + 云端 Drive 同步 |
| 网络 | 仅需要 LLM API 可达 | 需要连接 Warp 后端 |

```
┌──────────────────────────────────────────────────┐
│  App::assemble_ai()                              │
│                                                  │
│  根据用户配置和运行时状态决定 Agent 类型:          │
│                                                  │
│  if has_cloud_access() {                         │
│      → CloudAgent::new(warp_backend_client)      │
│  } else if has_byop_config() {                   │
│      → BYOPAgent::new(llm_provider_config)       │
│  } else {                                        │
│      → 不显示 Agent 入口                          │
│         (或引导用户配置 BYOP)                     │
│  }                                               │
└──────────────────────────────────────────────────┘

运行时行为:
  CloudAgent 内部也使用 BYOP 作为 fallback:
    当云端不可达时 → 自动降级为 BYOP
    当云端不支持某模型时 → 使用 BYOP 调用该模型
```

两种模式的上层接口（`AIAgent` trait）是统一的：

```
trait AIAgent {
    async fn create_conversation(&self) -> AIConversation;
    async fn send_message(&self, params: RequestParams)
        -> ResponseStream;
    async fn execute_tool(&self, action: AIAgentActionType)
        -> AIAgentActionResultType;
    async fn cancel(&self, exchange_id: &str);
    async fn get_status(&self) -> AgentStatus;
}
```

---

## 附录：关键索引

| 查找目标 | 路径 |
|---------|------|
| Agent 核心类型定义 | `crates/ai/src/` (protocol, types) |
| Tool Registry | `crates/ai/src/tool/` |
| MCP Client | `crates/ai/src/mcp/` |
| GenAI 适配器 | `crates/ai/src/genai/` |
| Proactive AI | `crates/ai/src/oneshot.rs` |
| Jinja2 模板 | `crates/ai/src/prompt/templates/` |
| Conversation 管理 | `app/src/ai/conversation_*.rs` |
| Agent 生命周期 | `app/src/ai/agent_*.rs` |
| BlockContext | `app/src/ai/block_context.rs` |
| Blocklist 控制 | `app/src/terminal/blocklist*.rs` |
| MCP 管理器 | `app/src/ai/mcp/` |
| Task 树 | `app/src/ai/task_*.rs` |
| Context 组装 | `app/src/ai/context_chips/` |
| Cloud Agent 客户端 | `app/src/ai/cloud_agent_*.rs` |
| 工具执行器 | `app/src/ai/tool_*.rs` |
| BYOP Compaction | `app/src/ai/compaction*.rs` |
| 编码入口 | `app/src/ai/coding_entrypoints/` |
