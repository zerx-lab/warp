# Drive / 同步架构设计

> 本文档描述 Warp / Zap 客户端的本地对象存储与 Drive 子系统,覆盖对象模型、ID 体系、持久化、同步架构、空间模型、元数据/权限、跨模块集成、关键模式。

---

## 1. 对象模型

### 1.1 三层抽象

```
┌──────────────────────────────────────────────────────────────┐
│  StoredObject trait (object-safe, 动态分发基类)              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  uid() → ObjectUid                                     │  │
│  │  sync_id() → SyncId                                    │  │
│  │  hashed_sqlite_id() → HashedSqliteId                   │  │
│  │  metadata() → &StoredObjectMetadata                    │  │
│  │  permissions() → &StoredObjectPermissions               │  │
│  │  object_type() → ObjectType                            │  │
│  │  upsert_event() → ModelEvent                           │  │
│  │  to_warp_drive_item() → Option<Box<dyn WarpDriveItem>> │  │
│  │  as_any() → &dyn Any                                   │  │
│  │  clone_box() → Box<dyn StoredObject>                   │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
│  ▲ 由 GenericStoredObject<K, M> 实现                          │
│                                                               │
│  GenericStoredObject<K, M> (泛型承载)                         │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  id: SyncId                                            │  │
│  │  metadata: StoredObjectMetadata                        │  │
│  │  permissions: StoredObjectPermissions                   │  │
│  │  model: Arc<M>          ← 领域模型(Clone-on-Write)     │  │
│  │  conflict_status: ConflictStatus                       │  │
│  │                                                        │  │
│  │  fn model(&self) → &M                                  │  │
│  │  fn set_model(&mut self, model: M)                     │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
│  ▲ 类型别名,如:                                                │
│    pub type NotebookObject = GenericStoredObject<             │
│        NotebookId, NotebookObjectModel                        │
│    >;                                                          │
│                                                               │
│  StoredObjectModel trait (领域模型描述)                       │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  type StoredObjectType                                 │  │
│  │  type IdType                                           │  │
│  │  fn model_type_name() → &'static str                   │  │
│  │  fn object_type() → ObjectType                         │  │
│  │  fn upsert_event(object) → ModelEvent                  │  │
│  │  fn display_name() → String                            │  │
│  │  fn serialized() → SerializedModel                     │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### 1.2 对象类型

```rust
// 核心类型变体
pub enum ObjectType {
    Notebook,
    Workflow,
    Folder,
    GenericStringObject(GenericStringObjectFormat),
}

pub enum GenericStringObjectFormat {
    Json(JsonObjectType),
}

pub enum JsonObjectType {
    Preference,          // 云同步偏好
    EnvVarCollection,    // 环境变量集合
    WorkflowEnum,        // 工作流枚举
    AIFact,              // AI 规则/AI Fact
    MCPServer,           // MCP 服务器
    AIExecutionProfile,  // AI 执行档案
    TemplatableMCPServer,// 模板化 MCP 服务器
}
```

### 1.3 DriveObjectType（UI 展示分类）

```rust
pub enum DriveObjectType {
    Workflow,
    AgentModeWorkflow,      // Agent 模式工作流
    AIFact,
    AIFactCollection,
    Notebook { is_ai_document: bool },
    Folder,
    EnvVarCollection,
    MCPServer,
    MCPServerCollection,
}
```

每种类型关联对应图标（`Icon::Workflow` / `Icon::Prompt` / `Icon::Notebook` 等）。

---

## 2. ID 体系

```
                         ID 体系
                            │
            ┌───────────────┴───────────────┐
            ▼                               ▼
        SyncId                          HashedSqliteId
    (运时对象标识)                     (SQLite 索引键)
            │                               │
    ┌───────┴───────┐                       │
    ▼               ▼                       │
ClientId        ServerId                    │
(UUID v4)    (22 字符定长串)                  │
                                            │
            HashedSqliteId = ObjectIdType::sqlite_prefix()
                           + "-"
                           + SyncId.uid()
            // 例如: "Workflow-<uid>"
            // 例如: "Folder-<uid>"
            // 例如: "GenericStringObject-GENERIC_STRING_JSON_AIFACT<uid>"

ObjectUid = String (来自 ClientId.to_string() 或 ServerId.uid())
```

### 2.1 SyncId

```rust
pub enum SyncId {
    ClientId(ClientId),   // 本地创建, UUID v4
    ServerId(ServerId),   // 遗留服务端 ID, 22 字符定长
}
```

双重 ID 设计支持:
- **离线创建**: 新对象先分配 `ClientId`,本地可见即可用
- **服务端对齐**: 遗留云端对象使用 `ServerId`,保持现有持久化数据兼容
- **转换**: `set_server_id()` 将本地对象的 `ClientId` 提升为 `ServerId`

### 2.2 ID Trait

```rust
pub trait HashableId: Sized + Send + Sync {
    fn to_hash(&self) -> String;
    fn from_hash(hash: &str) -> Option<Self>;
}
```

为 `ClientId`、`FolderId`、`NotebookId`、`WorkflowId` 等自动实现,用于从持久化字符串重建 ID。

---

## 3. 持久化

### 3.1 ObjectStoreModel

```
┌──────────────────────────────────────────────────────────────┐
│  ObjectStoreModel (SingletonEntity)                          │
│                                                               │
│  内存层:                                                      │
│    objects_by_id: HashMap<ObjectUid, Box<dyn StoredObject>>   │
│    initial_load_complete: Condition                          │
│                                                               │
│  持久化层:                                                    │
│    model_event_sender: Option<SyncSender<ModelEvent>>         │
│    ↓                                                          │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  ModelEvent 通道                                      │    │
│  │  ┌──────────────────────────────────────────────────┐│    │
│  │  │  UpsertWorkflow / UpsertNotebook / UpsertFolder  ││    │
│  │  │  UpsertGenericStringObject                        ││    │
│  │  │  DeleteObjects                                    ││    │
│  │  │  UpdateObjectMetadata                             ││    │
│  │  │  InsertCommand / SaveBlock / Snapshot ...         ││    │
│  │  └──────────────────────────────────────────────────┘│    │
│  └──────────────────────────────────────────────────────┘    │
│                                                               │
│  事件总线:                                                    │
│    ctx.emit(ObjectStoreEvent::ObjectUpdated { ... })          │
│    ctx.emit(ObjectStoreEvent::ObjectCreated { ... })          │
│    ctx.emit(ObjectStoreEvent::ObjectDeleted { ... })          │
│    ctx.emit(ObjectStoreEvent::ObjectMoved { ... })            │
│    ctx.emit(ObjectStoreEvent::ObjectTrashed { ... })          │
│    ctx.emit(ObjectStoreEvent::ObjectForceExpanded { ... })    │
│    ctx.emit(ObjectStoreEvent::InitialLoadCompleted)           │
└──────────────────────────────────────────────────────────────┘
```

- **内存 HashMap** 提供 O(1) 对象查找
- **SQLite 通道** 异步持久化变更
- **ObjectStoreEvent** 通知 UI 视图刷新

### 3.2 UpdateManager

```
┌──────────────────────────────────────────────────────────────┐
│  UpdateManager (SingletonEntity, 编排器)                     │
│                                                               │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────────┐    │
│  │  user edit   │───→│  更新 Model   │───→│  save_to_db() │    │
│  │  (UI 交互)  │    │  内存状态    │    │  写 SQLite  │    │
│  └─────────────┘    └──────────────┘    └──────────────┘    │
│                            │                                   │
│                            ▼                                   │
│                     emit Event                               │
│                     (通知 UI 刷新)                              │
│                                                               │
│  核心方法:                                                     │
│  - create_object(model, owner, client_id, ...)                │
│  - update_object(model, id, revision_ts, ctx)                 │
│  - update_workflow / update_notebook / update_ai_fact         │
│  - update_env_var_collection / update_templatable_mcp_server  │
│  - delete_object_by_user(object_type_and_id, ctx)             │
│  - trash_object(object_type_and_id, ctx)                      │
│  - duplicate_object(object_type_and_id, ctx)                  │
│  - move_object_to_location(object_id, new_location, ctx)      │
└──────────────────────────────────────────────────────────────┘
```

UpdateManager 三合一职责:
1. 更新 `ObjectStoreModel` 内存状态
2. 通过 `model_event_sender` 写 SQLite
3. 发出 `ObjectStoreEvent` 通知 UI

### 3.3 SQLite 表映射

```
内存对象 → ModelEvent → SQLite 写入

┌───────────────┐       ┌───────────────────────┐
│ WorkflowObject│──────→│ cloud_objects_metadata│
│               │       │ (类型/ID/修订/文件夹)  │
│               │──────→│ object_permissions    │
│               │       │ (Owner/Guests/Share)  │
│               │──────→│ workflows (JSON body) │
└───────────────┘       └───────────────────────┘

┌───────────────┐       ┌───────────────────────┐
│ NotebookObject│──────→│ cloud_objects_metadata│
│               │       ├───────────────────────┤
│               │──────→│ notebooks (JSON body) │
└───────────────┘       └───────────────────────┘

┌──────────────────────┐       ┌──────────────────────────┐
│ EnvVarCollectionObject│──────→│ cloud_objects_metadata   │
│                      │       ├──────────────────────────┤
│                      │──────→│ json_objects (JSON body)│
└──────────────────────┘       └──────────────────────────┘
```

---

## 4. 同步架构

### 4.1 架构总览

```
┌──────────────────×──────────────────────────────────────┐
│                  ×   Zap 本地化 (云端同步已剥离)          │
│                  ×                                       │
│  ┌──────────┐   ×   ┌──────────────────┐               │
│  │ UI 视图  │   ×   │  UpdateManager   │               │
│  │ (Drive)  │──×──→│  (编排器)        │               │
│  └──────────┘   ×   └────────┬─────────┘               │
│                  ×            │                          │
│                  ×     ┌──────▼────────┐               │
│  ┌──────────┐   ×     │ ObjectStoreModel│              │
│  │ 终端     │   ×     │ (内存 HashMap)  │              │
│  │(Notebook)│──×──→  └──────┬────────┘               │
│  └──────────┘   ×            │                          │
│                  ×     ┌──────▼────────┐               │
│  ┌──────────┐   ×     │ ModelEvent     │               │
│  │ AI/Agent │   ×     │ (SyncChannel)  │               │
│  │ (AIFact) │──×──→  └──────┬────────┘               │
│  └──────────┘   ×            │                          │
│                  ×     ┌──────▼────────┐               │
│  ┌──────────┐   ×     │ SQLite 写入线程│              │
│  │ 设置     │   ×     └───────────────┘               │
│  │(Preference)─×──→                                  │
│  └──────────┘   ×                                       │
│                  ×                                       │
│  以下已剥离:     ×                                       │
│  ✗ SyncQueue    ×                                       │
│  ✗ ServerApiProvider ×                                   │
│  ✗ RTC 连接     ×                                       │
│  ✗ 服务端往返   ×                                       │
└──────────────────×──────────────────────────────────────┘
```

### 4.2 离线优先设计原则

1. **所有写入首先到达本地**: 无论数据来源（用户编辑 / 系统生成）,先写入 `ObjectStoreModel` 内存 + SQLite
2. **冲突检测遗留**: `ConflictStatus` 枚举和 `has_conflicting_changes()` 方法保留,但对云端冲突的实际处理逻辑已剥离
3. **乐观更新**: 操作立即反映在 UI 上,不需要等待任何网络往返
4. **SyncQueue 为空实现**: `SyncQueue` 和 `ServerApiProvider` 的所有方法已被清空或直接返回 `Ok(())`

### 4.3 与上游的差异化

| 功能 | 上游 Warp | Zap 本地化 |
|------|-----------|-----------|
| 对象创建 | 本地创建 + SyncQueue 入队 → 服务端同步 | 纯本地创建,写 SQLite |
| 对象移动 | 本地乐观更新 + 服务端 RPC 确认 | 纯本地内存 + SQLite |
| 冲突解决 | 服务端冲突检测 + UI 解决提示 | 保留枚举结构,无实际逻辑 |
| 刷新 | 轮询服务端获取更新 | `refresh_updated_objects()` 为空实现 |
| 链接 | 生成服务端可共享 URL | `object_link()` 仍可生成,但不会上传同步 |

---

## 5. 空间模型

```rust
pub enum Space {
    Personal,                    // 当前用户个人空间
    Team { team_uid: ServerId }, // 团队空间
    Shared,                      // 与当前用户共享的对象
}

pub enum StoredObjectLocation {
    Space(Space),                // 空间顶层
    Folder(SyncId),              // 某文件夹内
    Trash,                       // 回收站
}
```

### 空间推导

```rust
// Owner → Space 的映射
fn owner_to_space(owner: Owner, app) -> Space {
    match owner {
        Owner::User { user_uid } if user_uid == current_user =>
            Space::Personal,
        Owner::User { .. } =>
            Space::Shared,
        Owner::Team { team_uid } if user_in_team(team_uid) =>
            Space::Team { team_uid },
        Owner::Team { .. } =>
            Space::Shared,
    }
}
```

- `ObjectTypeAndId` 是 Drive UI 中标识特定对象位置的判别式枚举
- `ObjectStoreModel` 提供 `active_cloud_objects_in_space()`, `directly_trashed_cloud_objects_in_space()` 等空间查询方法

---

## 6. 元数据与权限

### 6.1 Revision（微秒时间戳）

```rust
pub struct Revision(ServerTimestamp);

// ServerTimestamp = DateTime<Utc>
// timestamp_micros() → i64
```

- Revision 本质是 UTC 时间的微秒精度时间戳
- 用于对象的版本排序和冲突检测
- `Revision::from_unix_timestamp_micros(micros)` / `Revision::now()`

### 6.2 StoredObjectMetadata

```rust
pub struct StoredObjectMetadata {
    pub revision: Option<Revision>,                    // 修订号
    pub metadata_last_updated_ts: Option<ServerTimestamp>,
    pub current_editor_uid: Option<String>,            // 当前编辑者
    pub pending_changes_statuses: StoredObjectStatuses,// 同步状态
    pub trashed_ts: Option<ServerTimestamp>,            // 删除时间
    pub folder_id: Option<SyncId>,                     // 父文件夹
    pub is_welcome_object: bool,                       // 欢迎对象
    pub last_editor_uid: Option<String>,
    pub creator_uid: Option<String>,
    pub last_task_run_ts: Option<ServerTimestamp>,
}

pub struct StoredObjectStatuses {
    pub content_sync_status: StoredObjectSyncStatus,    // 内容同步状态
    pub has_pending_permissions_change: bool,           // 权限待变更
    pub has_pending_metadata_change: bool,              // 元数据待变更
    pub pending_untrash: bool,                         // 待恢复
    pub pending_delete: bool,                          // 待删除
}

pub enum StoredObjectSyncStatus {
    NoLocalChanges,          // 已同步
    InFlight(NumInFlightRequests), // 传输中
    InConflict,              // 冲突
    Errored,                 // 错误
}
```

### 6.3 StoredObjectPermissions

```rust
pub struct StoredObjectPermissions {
    pub owner: Owner,                              // 所有者
    pub permissions_last_updated_ts: Option<ServerTimestamp>,
    pub anyone_with_link: Option<LinkSharing>,     // 链接分享
    pub guests: Vec<StoredObjectGuest>,            // 协作成员
}

pub enum Owner {
    User { user_uid: UserUid },
    Team { team_uid: ServerId },
}

pub struct LinkSharing {
    pub access_level: SharingAccessLevel,
    pub source: Option<ServerObjectContainer>,
}

pub struct StoredObjectGuest {
    pub subject: Subject,
    pub access_level: SharingAccessLevel,
    pub source: Option<ServerObjectContainer>,
}
```

---

## 7. 与其他模块的集成

### 7.1 终端（Workflow / EnvVar / Notebook）

```
终端 / 命令执行
    │
    ├── 工作流触发: 通过 workflow_id → ObjectStoreModel.get_workflow()
    │                → 读取工作流定义 → 注入命令参数
    │
    ├── 环境变量注入: EnvVarCollection → 读取键值对
    │                → 注入到 shell 进程
    │
    └── 打开 Notebook: NotebookObject → NotebookPane
                      → 从 SQLite 恢复 notebook 数据
```

### 7.2 Drive 设置

```rust
define_settings_group!(WarpDriveSettings, settings: [
    sorting_choice: WarpDriveSortingChoice {
        type: DriveSortOrder,
        toml_path: "warp_drive.sorting_choice",
    },
    sharing_onboarding_block_shown: WarpDriveSharingOnboardingBlockShown {
        type: bool,
        private: true,  // 私有设置
    },
    enable_warp_drive: EnableWarpDrive {
        type: bool,
        toml_path: "warp_drive.enabled",
    },
]);
```

- `is_warp_drive_enabled()`: 检查设置 + 用户登录状态
- DriveSortOrder: `ByTimestamp` / `Alphabetical*` / `ByObjectType`

### 7.3 AI / Agent

```
AI / Agent 模块
    │
    ├── AIFact:     AI 规则同步 → ObjectStoreModel
    │                → AIFactObjectModel(内容/描述/适用目录)
    │                → UpdateManager::update_ai_fact()
    │
    ├── AIExecutionProfile: AI 执行档案同步
    │                → AIExecutionProfileObjectModel
    │                → UpdateManager::create_ai_execution_profile()
    │
    └── MCPServer:  MCP 服务器配置同步
                    → MCPServerObjectModel
                    → TemplatableMCPServerObjectModel
                    → UpdateManager::create_templatable_mcp_server()
```

### 7.4 ObjectTypeAndId 与 UI

```rust
pub enum ObjectTypeAndId {
    Notebook(SyncId),
    Workflow(SyncId),
    Folder(SyncId),
    GenericStringObject { object_type: GenericStringObjectFormat, id: SyncId },
}
```

被 60+ 处代码使用,用于:
- `DrivePanel` 中渲染组织树
- `UpdateManager` 中定位对象执行操作
- `search` 模块中跨类型搜索
- `notebooks` / `workflows` 模块打开特定对象

---

## 8. 关键模式

### 8.1 离线优先写入

```
用户编辑 → UpdateManager::update_object()
    │
    ├── 1. 更新 ObjectStoreModel 内存状态（立即）
    │       └── set_model(new_model)
    │
    ├── 2. 发送 ModelEvent 到 SQLite 通道（异步,不阻塞）
    │       └── model_event_sender.send(upsert_event)
    │
    └── 3. 发出 ObjectStoreEvent 通知 UI（同步）
            └── ctx.emit(ObjectStoreEvent::ObjectUpdated)
```

所有操作都遵循"先写内存、再写 SQLite"的顺序,确保 UI 始终响应。

### 8.2 ObjectStoreEvent 事件总线

```
┌─────────────────────────────────────────┐
│  ObjectStoreEvent 变体                   │
├─────────────────────────────────────────┤
│  ObjectCreated                          │
│  ObjectUpdated (带 UpdateSource)         │
│  ObjectDeleted (带 folder_id)            │
│  ObjectMoved (from_folder → to_folder)  │
│  ObjectTrashed / ObjectUntrashed        │
│  ObjectPermissionsUpdated               │
│  ObjectForceExpanded                    │
│  NotebookEditorChangedExternally        │
│  InitialLoadCompleted                   │
└─────────────────────────────────────────┘
```

- `UpdateSource::Local` vs `UpdateSource::External` 区分变更来源
- 订阅者：DrivePanel、Search、Notifications、Workspace 等
- 每个 `ObjectStoreEvent` 通过 `ctx.emit()` 在 ModelContext 中广播

### 8.3 UpdateManager 编排

```
UpdateManager 各方法调用流:

create_object(model, owner, client_id, ...):
    ObjectStoreModel::handle().update(|model, ctx| {
        model.create_object(id, generic_object, ctx)
    })
    save_to_db([upsert_event])

update_object(model, id, revision_ts, ctx):
    ObjectStoreModel::handle().update(|model, ctx| {
        model.update_object_from_edit(model, id, ctx)
    })
    save_to_db([upsert_event])

delete_object_by_user(object_type_and_id, ctx):
    ObjectStoreModel::handle().update(|model, ctx| {
        model.delete_object(id, ctx)
    })
    save_to_db([ModelEvent::DeleteObjects { ids }])

trash_object(object_type_and_id, ctx):
    ObjectStoreModel::handle().update(|model, ctx| {
        // 标记 trashed_ts
    })
    save_to_db([ModelEvent::UpdateObjectMetadata { ... }])
```

### 8.4 Clone-on-Write 模型

```rust
pub struct GenericStoredObject<K, M> {
    // ...
    model: Arc<M>,  // Arc 包装,实现克隆共享
}

// 克隆时只增加 Arc 引用计数,不做深拷贝
// 修改时必须调用 set_model() 替换整个 Arc
fn set_model(&mut self, model: M) {
    self.model = model.into();  // Arc::new
}
```

### 8.5 初始加载流程

```rust
// 1. SQLite 初始化 → read_sqlite_data()
//    ├── 读取 app_state (窗口/标签/窗格)
//    ├── 读取 cloud_objects (所有 StoredObject)
//    ├── 读取 workspaces / user_profiles / commands
//    └── 组装 PersistedData

// 2. ObjectStoreModel::new(cached_objects, ...)
//    ├── objects_by_id 从 Vec<Box<dyn StoredObject>> 构建
//    └── initial_load_complete 立即标记完成(纯本地无远端拉取)

// 3. initial_load_complete() → Future
//    └── UI 视图在第一次 render 前 await 此 condition
```

### 8.6 空的云端同步钩子

Zap 本地化保留了上游云端同步的接口签名以保证代码兼容,但所有实现均为空:

```rust
// UpdateManager 中:
fn fetch_single_cloud_object(...) -> Receiver<()> {
    let (tx, rx) = oneshot::channel();
    let _ = tx.send(());           // 立即完成
    rx
}

fn refresh_updated_objects(&mut self, ctx) {
    let _ = ctx;                   // 不做任何事
}

fn resync_object(&mut self, ...) {
    let _ = ...;                   // 不做任何事
}
```

这些钩子在未来如需加回云端同步时只需填充实现,不需要改动调用点。

---

## 附录: 关键文件索引

| 文件 | 用途 |
|------|------|
| `app/src/cloud_object/mod.rs` | `StoredObject` trait, `GenericStoredObject`, `StoredObjectModel` 定义 |
| `app/src/cloud_object/server_types.rs` | `ObjectType`, `ObjectIdType`, `JsonObjectType`, `Revision`, `Owner`, `Space`, `StoredObjectMetadata`, `StoredObjectPermissions` |
| `app/src/cloud_object/model/persistence.rs` | `ObjectStoreModel`（内存 HashMap + SQLite 通道 + 事件总线） |
| `app/src/cloud_object/update_manager.rs` | `UpdateManager`（创建/更新/删除/移动/复制编排） |
| `app/src/server/ids.rs` | `SyncId`, `ClientId`, `ServerId`, `HashedSqliteId`, `ObjectUid` |
| `app/src/drive/mod.rs` | `DriveObjectType`, `ObjectTypeAndId`, `DriveSortOrder` |
| `app/src/drive/settings.rs` | `WarpDriveSettings`（`define_settings_group!`） |
| `app/src/drive/folders/mod.rs` | `FolderObjectModel`, `FolderObject` |
| `app/src/persistence/mod.rs` | `ModelEvent` 枚举, `WriterHandles` |
| `app/src/persistence/sqlite.rs` | SQLite 读写实现, 写入器线程 |
