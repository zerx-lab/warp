# 持久化架构设计

> 本文档描述 Warp / Zap 客户端的持久化体系,覆盖 SQLite 数据库、设置系统、Feature Flag 三个层面。

---

## 1. 数据库架构

### 1.1 技术选型

```
┌──────────────────────────────────────────────────┐
│                    App Context                     │
├──────────────────────────────────────────────────┤
│  PersistenceWriter (SingletonEntity)              │
│  ┌────────────────────────────────────────────┐  │
│  │  SyncSender<ModelEvent>   (CHANNEL_SIZE=1024)│  │
│  └──────────┬─────────────────────────────────┘  │
│             │ send                               │
│  ┌──────────▼─────────────────────────────────┐  │
│  │  SQLite Writer Thread (专用后台线程)         │  │
│  │  • rx.recv() 阻塞等待事件                    │  │
│  │  • try_iter() 批量拉取 → deduplicate_events │  │
│  │  • handle_model_event() 写入 SQLite          │  │
│  │  • 退出时 PRAGMA wal_checkpoint(TRUNCATE)    │  │
│  └─────────────────────────────────────────────┘  │
│                                                    │
│  Reader Connections: establish_ro_connection()      │
│  (各模块按需建立只读连接,通过 file:path?mode=ro)    │
└──────────────────────────────────────────────────┘
```

- **ORM**: Diesel (`diesel_migrations::EmbeddedMigrations`)
- **数据库引擎**: SQLite3 (`libsqlite3-sys`)
- **连接模式**: 单写入器 + N 只读,写入通过 `SyncSender<ModelEvent>` 通道异步
- **数据库路径**: `secure_state_dir() / warp.sqlite`（macOS 下为安全容器区域）

### 1.2 连接 PRAGMA

```rust
// 建立连接时执行
PRAGMA foreign_keys = ON;           // 外键约束
PRAGMA busy_timeout = 1000;         // 忙等待 1 秒
PRAGMA journal_mode = WAL;          // WAL 模式
PRAGMA wal_autocheckpoint = 500;    // WAL 日志 ~2MB 时自动 checkpoint
```

- WAL 模式允许读写并发,读取不阻塞写入
- `wal_autocheckpoint=500` 比默认值（1000）更低,因为写入已集中在后台线程,更频繁 checkpoint 可控制 WAL 体积

### 1.3 Schema 管理

```
┌──────────────────────────────┐
│  crates/persistence/         │
│  ├── src/lib.rs              │  ← embed_migrations!("migrations")
│  ├── src/schema.rs           │  ← Diesel 自动生成(不做手改)
│  ├── src/model.rs            │  ← 表结构定义
│  └── migrations/             │  ← 142 个迁移(截至 2026-06)
│      ├── 2021-10-14-.../
│      ├── ...
│      └── 2026-06-24-.../
└──────────────────────────────┘
```

迁移执行:

```rust
// crates/persistence/src/lib.rs
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// app/src/persistence/sqlite.rs:setup_database()
conn.run_pending_migrations(persistence::MIGRATIONS)
```

- **schema.patch 机制**: 对于无法通过标准 migration 表达的变更（如 SQLite 不支持 `ALTER COLUMN`）,使用 `schema.patch` 文件记录,在 `embed_migrations!` 之后额外执行。
- **幂等迁移**: 每个 `up.sql` 中的 DDL 以 `IF NOT EXISTS` / `IF EXISTS` 包裹;down.sql 同理。
- **macOS 遗留迁移**: `migrate_zap_app_group_sqlite_if_needed()` 处理从旧 App Group 容器到新安全容器的 SQLite 文件迁移。

---

## 2. 表分组（142 个迁移,40+ 表）

### 2.1 窗口 / 标签 / 窗格布局树

```
windows
├── tabs
│   ├── pane_leaves             ← 终端/编辑器/Notebook 等叶节点
│   │   ├── terminal_panes
│   │   ├── setting_panes
│   │   ├── notebook_panes
│   │   ├── code_panes
│   │   ├── env_var_collection_panes
│   │   ├── mcp_panes
│   │   ├── ai_exchanges / ai_queries
│   │   ├──...
│   │   └── ai_document_panes
│   ├── code_pane_tabs
│   └── panels                   ← 左右面板
├── workspaces                  ← 工作区(多租户)
├── projects                    ← 项目定义
├── project_rules               ← 项目级 AI 规则
└── current_workspace           ← 激活工作区

窗口属性:全屏状态、quake 模式、主题覆盖、左侧面板开关、垂直面板开关
```

### 2.2 云对象缓存

```
cloud_objects_metadata      ← stored object 元数据(类型/UID/修订/文件夹关系)
cloud_objects_refreshes     ← 刷新时间戳
object_permissions          ← 权限(Owner/Guests/LinkSharing)
object_actions              ← 操作历史(创建/更新/移动/删除)
folders                     ← 文件夹树
notebooks                   ← 笔记本数据
json_objects                ← 通用 JSON 对象(EnvVar/AIFact/MCP/ExecutionProfile)
```

### 2.3 命令历史

```
commands                    ← 执行的命令记录(内容/退出码/时间/工作目录/Git 分支)
commands_ai_metadata        ← AI 关联(agent_executed 标记)
```

### 2.4 终端块

```
blocks                      ← 终端输出块
blocks_content              ← 块二进制内容
ai_queries                  ← AI 查询记录
```

### 2.5 AI / Agent

```
agent_conversations         ← Agent 对话会话
conversation_tasks          ← 多 Agent 任务编排
ai_memory_panes / mcp_environment_variables
mcp_server_installations    ← MCP 服务器安装记录
running_mcp_servers         ← 运行时 MCP 服务器
active_mcp_servers
codebase_index_metadata     ← 代码索引元数据
```

### 2.6 多租户 / Workspace

```
teams                       ← 团队信息
team_settings               ← 团队配置
workspaces                  ← 工作区
workspace_teams             ← 工作区-团队关联
user_profiles               ← 用户档案
current_user_information    ← 当前登录用户信息
```

### 2.7 SSH 管理器

```
ssh_connections / ssh_hosts / ssh_groups / ssh_settings
ssh_onekey_credentials / ssh_onekey_key_type
ssh_nodes_is_collapsed      ← SSH 树折叠状态
```

### 2.8 MCP

```
mcp_servers                 ← MCP 服务器定义
mcp_environment_variables   ← MCP 环境变量
templatable_mcp_installations
mcp_server_installations
```

### 2.9 其他

```
app_table                   ← 应用级 KV 存储
server_experiments          ← 服务端实验配置
ignored_suggestions         ← 用户忽略的提示
user_profiles               ← 用户配置文件
sync_meta                   ← 同步元数据
billing_metadata (teams)    ← 计费信息
current_user_information    ← 用户登录 token 等
+ 工具命令/撤销历史等
```

---

## 3. 迁移策略

### 3.1 迁移目录

```
crates/persistence/migrations/
├── 2021-10-14-155402_empty_migration/
├── 2021-10-18-232826_create_windows_and_tabs/
│   ├── up.sql
│   └── down.sql
├── ...
└── 2026-06-24-000000_add_window_theme_override/
    ├── up.sql
    └── down.sql
```

命名规范: `${YYYY}-${MM}-${DD}-${HHMMSS}_${description}/`

### 3.2 启动时迁移

```rust
// 启动流程:
// 1. prewarm_db_in_background()        ← 后台线程预热(可选)
// 2. take_prewarmed_db() / init_db()   ← 取预热或同步初始化
//    3. setup_database()
//       a. establish_connection()
//       b. conn.run_pending_migrations(MIGRATIONS)
//       c. apply_schema_patches()       ← 额外 DDL patch
// 4. read_sqlite_data()                ← 读取持久化数据到内存
// 5. start_writer()                    ← 启动写入器线程
```

### 3.3 schema.patch 机制

当 Diesel migration 不能表达所需 DDL（例如 SQLite 不支持 `ALTER COLUMN`）时:

1. 在 `migrations/` 下创建一个正常的迁移
2. 在 `crates/persistence/src/schema.patch` 中记录额外变更
3. `run_pending_migrations` 后执行 patch

### 3.4 幂等原则

- 迁移的 `up.sql` 始终使用 `CREATE TABLE IF NOT EXISTS`
- 避免有状态逻辑（如数据迁移）在迁移中生硬嵌入,优先走应用层迁移
- 失败后允许重试

---

## 4. 设置持久化

Warp 设置系统采用**两层存储架构**:

```
┌──────────────────────────────────────────────────────────────┐
│                       App Context                             │
├──────────────────────────────────────────────────────────────┤
│  PublicPreferences (SingletonEntity)                          │
│  ┌────────────────────────────────────┐                       │
│  │  TOML 文件 (设置文件启用时)         │                       │
│  │  └ warp_settings.toml             │                       │
│  └────────────────────────────────────┘                       │
│                          OR                                    │
│  ┌────────────────────────────────────┐                       │
│  │  PrivatePreferences (fallback)    │                       │
│  │  └ NSUserDefaults / JSON / reg    │                       │
│  └────────────────────────────────────┘                       │
│                                                                │
│  PrivatePreferences (SingletonEntity)                          │
│  └ 平台原生存储(macOS UserDefaults / Linux Keyring /           │
│     Windows Registry)                                          │
│                                                                │
│  ┌────────────────────────────────────┐                       │
│  │  define_settings_group! 宏          │                       │
│  │  └ 类型安全的 Setting 结构体        │                       │
│  │  └ 每个字段 → 独立的 storage key    │                       │
│  └────────────────────────────────────┘                       │
└──────────────────────────────────────────────────────────────┘
```

### 4.1 UserPreferences trait

```rust
pub trait UserPreferences: Send + Sync {
    fn read_value(&self, key: &str) -> Result<Option<String>>;
    fn write_value(&self, key: &str, value: String) -> Result<()>;
    fn remove_value(&self, key: &str) -> Result<()>;
    fn read_value_with_hierarchy(&self, key: &str, hierarchy: Option<&str>) -> Result<Option<String>>;
    fn write_value_with_hierarchy(&self, key: &str, value: String, hierarchy: Option<&str>, max_table_depth: Option<u32>) -> Result<()>;
    fn is_settings_file(&self) -> bool;
    fn reload_from_disk(&self) -> Result<(), Error>;
    fn inhibit_writes_for_key(&self, key: &str, hierarchy: Option<&str>);
}
```

两个实现:

| 后端 | 用途 | 存储位置 |
|------|------|----------|
| TOML 文件（`PublicPreferences`） | 公共设置 | `~/.config/warp/warp_settings.toml` |
| 原生存储（`PrivatePreferences`） | 私有设置(密钥/窗口位置等) | 平台原生机制 |

### 4.2 Setting trait

```rust
pub trait Setting {
    type Group: Entity;
    type Value: Serialize + DeserializeOwned + PartialEq + Debug + SettingsValue;

    fn storage_key() -> &'static str;      // 唯一存储键
    fn toml_path() -> Option<&'static str>; // TOML 路径(如 "appearance.text.font_name")
    fn sync_to_cloud() -> SyncToCloud;      // 云端同步策略
    fn supported_platforms() -> SupportedPlatforms;
    fn is_private() -> bool;                // 是否私有

    fn value(&self) -> &Self::Value;
    fn set_value(&mut self, new_value: Self::Value, ctx: &mut ModelContext<Self::Group>) -> Result<()>;
    fn load_value(&mut self, new_value: Self::Value, explicitly_set: bool, ctx: &mut ModelContext<Self::Group>) -> Result<()>;
    fn clear_value(&mut self, ctx: &mut ModelContext<Self::Group>) -> Result<()>;
}
```

### 4.3 define_settings_group! 宏

```rust
define_settings_group!(WarpDriveSettings, settings: [
    sorting_choice: WarpDriveSortingChoice {
        type: DriveSortOrder,
        default: DriveSortOrder::ByObjectType,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "warp_drive.sorting_choice",
        description: "The sort order for items in Zap Drive.",
    },
    enable_warp_drive: EnableWarpDrive {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "warp_drive.enabled",
        description: "Whether Zap Drive is enabled.",
    },
]);
```

宏展开生成:
- `struct WarpDriveSettings`（包含 `WarpDriveSortingChoice`, `EnableWarpDrive` 字段）
- 每个字段是实现了 `Setting` trait 的类型
- 自动注册到全局 `SettingsManager`

---

## 5. SettingsValue — 绕过 serde 的自定义序列化

### 5.1 问题

TOML 设置文件需要更人性化的表示,而 serde 的输出可能不适合直接展示给用户。例如:
- `Duration` → serde 输出 `{"secs":30,"nanos":0}`,但人类期望 `30`
- 枚举 → serde 输出 `{"VariantName":...}`,但人类期望 `variant_name`
- `HashSet` → serde 输出顺序不确定的 JSON 数组

### 5.2 解决方案: SettingsValue trait

```rust
pub trait SettingsValue: Serialize + DeserializeOwned {
    fn to_file_value(&self) -> Value {
        serde_json::to_value(self).expect("...")
    }

    fn from_file_value(value: &Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok()
    }
}
```

三种实现策略:

```
                        SettingsValue 实现方式
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
      #[derive(SettingsValue)]   空 impl    手动覆写
        │                       (serde 透传)   │
        ▼                          │           ▼
    enum: snake_case               │       Duration→秒
    变体名转换                      │       AgentMode→regex
    struct: 递归调用                 │
    每个字段的 to_file_value        │
```

### 5.3 枚举 snake_case 转换

```rust
// #[derive(SettingsValue)] 对枚举的处理:

// 输入:
enum DriveSortOrder {
    ByTimestamp,            // → "by_timestamp"
    AlphabeticalDescending, // → "alphabetical_descending"
    AlphabeticalAscending,  // → "alphabetical_ascending"
    ByObjectType,           // → "by_object_type"
}

// 生成:
// to_file_value:
//   DriveSortOrder::ByTimestamp → Value::String("by_timestamp")
// from_file_value:
//   Value::String("by_timestamp") → Some(DriveSortOrder::ByTimestamp)
```

### 5.4 读写路径

```
读路径:
  TOML 文件 → serde_json::from_str<Value> → M::from_file_value → 最终类型

写路径:
  最终类型 → M::to_file_value → serde_json::to_string → TOML 写入
```

云同步和原生存储路径仍使用标准 serde,只有 TOML 设置文件走 SettingsValue。

---

## 6. 变更传播

### 6.1 SettingsManager 架构

```
┌──────────────────────────────────────────────────┐
│  SettingsManager (SingletonEntity)                │
│                                                    │
│  settings: HashMap<storage_key, SettingsInfo>      │
│  update_fns: HashMap<storage_key, UpdateFn>       │
│  load_fns:   HashMap<storage_key, LoadFn>          │  ← 热重载专用
│  clear_fns:  HashMap<storage_key, ClearFn>         │
│  equals_fns: HashMap<storage_key, EqualsFn>       │
│  is_syncable_fns: HashMap<storage_key, IsSyncableFn>│
└──────────────────────────────────────────────────┘
```

### 6.2 事件流

```
用户交互 (UI Toggle/Slider/Input)
    │
    ▼
Setting::set_value(new_value, ctx)
    │
    ├── 1. validate(new_value)
    ├── 2. write_to_preferences(new_value, prefs)
    │       ├── toml_key / hierarchy / max_table_depth
    │       ├── SettingsValue::to_file_value → serde_json → 写入
    │       └── 语义比较:反序列化旧值对比,避免无意义写入
    ├── 3. emit(SettingsEvent::LocalPreferencesUpdated { storage_key, sync_to_cloud })
    │
    ▼
SettingsManager::update_setting_with_storage_key()
    │
    ├── 获取已注册的 update_fn
    └── 调用 update_fn(new_value, from_cloud_sync, ctx)
```

### 6.3 热重载（防写回循环）

```
文件变化 (FS watcher)
    │
    ▼
PublicPreferences::reload_from_disk()
    │
    ▼
SettingsManager::reload_all_public_settings()
    │
    ├── 遍历所有非私有设置
    ├── 从最新文件读取值
    └── 对每个设置调用 load_fn(value, explicitly_set, ctx)
        └── 内部调用 Setting::load_value() ← 只更新内存,不写回存储

关键: load_value 与 set_value 的区别
  - set_value:   更新内存 + 写回存储
  - load_value:  更新内存,不写回 ← 防止文件 watcher 写回循环
```

### 6.4 SettingsFile 切换

通过 `SETTINGS_FILE_ENABLED` 静态原子变量控制:

```rust
set_settings_file_enabled(enabled);  // 启动时设置一次

fn preferences_for_setting(ctx) -> &dyn UserPreferences {
    if Self::is_private() {
        <PrivatePreferences>          // 私有:永远走原生存储
    } else if is_settings_file_enabled() {
        <PublicPreferences>           // 公共+启用:走 TOML 文件
    } else {
        <PrivatePreferences>          // 公共+未启用:fallback 到原生
    }
}
```

---

## 7. Feature Flag 系统

### 7.1 枚举定义

```rust
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Sequence)]
pub enum FeatureFlag {
    Changelog,
    CrashReporting,
    AgentMode,
    SettingsFile,
    // ... 200+ 个变体 (截至 2026-06)
}
```

### 7.2 四层发布通道

```
DEBUG_FLAGS ──→ 仅 debug build,永不发布
    │  [DebugMode, RuntimeFeatureFlags]

DOGFOOD_FLAGS ──→ 内部团队抢先体验
    │  [AgentModeWorkflows, MultiWorkspace, Projects, ...]

PREVIEW_FLAGS ──→ 预览版用户(Friends of Zap)
    │  [MarkdownTables, GitOperationsInCodeReview]

RELEASE_FLAGS ──→ 全体发布用户
    │  [Autoupdate, Changelog, CrashReporting, ...]
```

通道优先级:

```
Channel 层级关系:
  Local / Dev (dogfood=true)
    └─ 启用: DEBUG + DOGFOOD + PREVIEW + RELEASE
  Preview (dogfood=false)
    └─ 启用: PREVIEW + RELEASE
  Stable / Oss (dogfood=false)
    └─ 启用: RELEASE
```

### 7.3 运行时 is_enabled()

```rust
impl FeatureFlag {
    pub fn is_enabled(&self) -> bool {
        // 1. 线程本地覆盖(测试用)
        overrides::get_override(*self)
            .or(
        // 2. 用户偏好覆盖(设置页面手动开关)
        USER_PREFERENCE_MAP[*self as usize].get()
            )
            .or(
        // 3. 通道默认值
        Some(FLAG_STATES[*self as usize].load(Ordering::Relaxed))
            )
            .unwrap_or(false)
    }
}
```

优先级: 线程覆盖 > 用户偏好 > 通道默认

### 7.4 使用原则

```
// ✅ 优先: 运行时检查
if FeatureFlag::AgentModeWorkflows.is_enabled() {
    // 新功能代码
}

// ❌ 避免: 编译时 cfg,除非无法编译(平台/依赖)
// #[cfg(feature = "...")]
```

- 上线稳定后清理 flag 与死分支
- UI 入口与代码路径使用同一个 flag

---

## 8. 关键模式

### 8.1 SQLite 写入器线程（CQRS）

```
┌──────────┐   send     ┌─────────────────┐   batch_write   ┌──────────┐
│ UI/Model │ ────────→  │ SyncChannel(1024)│ ─────────────→ │ SQLite   │
│ 模块      │            │ [事件队列,FIFO]  │                │ 写入线程  │
└──────────┘            └─────────────────┘                └──────────┘
                                │
                                │ try_iter() 批量拉取
                                ▼
                         deduplicate_events()
                         │
                         ├── 相同 pane_id 的连续 SaveBlock 合并
                         ├── 相同 id 的 UpsertWorkflow 去重
                         └── UpsertNotebook + DeleteBlocks 等
```

写入器是一个独立线程（`start_writer`）:
1. `rx.recv()` 阻塞等待第一个事件
2. `rx.try_iter()` 批量拉取累积事件
3. `deduplicate_events()` 语义去重（相同 pane_id 的 SaveBlock 合并）
4. `handle_model_event()` 逐个写入 SQLite
5. 退出时 `PRAGMA wal_checkpoint(TRUNCATE)` 确保 WAL 回写

### 8.2 后台预热

```rust
// 启动流程(并行):
// [main thread]          [background thread]
// app_builder.run()      prewarm_db_in_background()
//     │                       │
//     │                   init_db() ← 70-90ms
//     │                       │
//     └─── 取预热结果 ──────┘
//           │
//       initialize(ctx)
```

`PrewarmState` 状态机:

```
Pending → (后台线程运行中)
    │
    ├── 后台完成 → Done
    └── 主线程取走 → Joining → (等 join) → Done
```

### 8.3 语义去重

```rust
fn deduplicate_events(events: Vec<ModelEvent>) -> Vec<ModelEvent> {
    // 对于相同 pane_id 的 SaveBlock,只保留最新
    // 对于相同 UID 的 UpsertWorkflow/UpsertNotebook,只保留最新
    // 对于冲突类型的操作(如 DeleteBlocks + SaveBlock),裁剪
}
```

### 8.4 设置热重载安全

```
┌─────────────────┐
│  FS Watcher     │  发现 settings.toml 变化
└────────┬────────┘
         │
         ▼
PublicPreferences::reload_from_disk()
         │
         ▼
SettingsManager::reload_all_public_settings()
         │
         ├── 收集所有设置的最新值(不可变收集)
         ├── 逐个调用 load_fn(value, explicitly_set, ctx)
         │        └── Setting::load_value() ← 只写内存
         └── 失败时: inhibit_writes_for_key() ← 禁止误写回无效值
```

### 8.5 冷启动优化顺序

```
1. prewarm_db_in_background()       ← 尽早调用,与 winit/wgpu 并行
2. 取预热 SQLite 连接
3. 执行迁移
4. read_sqlite_data() → PersistedData
5. start_writer() → WriterHandles
6. 注册所有设置组 → SettingsManager
7. 应用第一次全量热重载
8. 启动 FS watcher 监听设置文件
```
