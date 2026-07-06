use std::ops::Deref;

use serde::{Serialize, Serializer};

use warpui::{ViewContext, platform::Cursor};

use crate::{
    send_telemetry_from_ctx,
    server::telemetry::{LinkOpenMethod, TelemetryEvent},
    terminal::{
        TerminalModel,
        model::{
            RespectObfuscatedSecrets,
            grid::grid_handler::Link,
            index::Point,
            terminal_model::{WithinBlock, WithinModel},
        },
    },
};

cfg_if::cfg_if! {
    if #[cfg(feature = "local_fs")] {
        use crate::{
            terminal::model::grid::grid_handler,
            terminal::ShellLaunchData,
            util::file::{FileLink, absolute_path_if_valid, ShellPathType},
            util::openable_file_type::FileTarget,
        };
        use std::path::PathBuf;
        use warp_util::path::CleanPathResult;
        use warp_util::path::LineAndColumnArg;
    }
}

use super::{FindLinkArg, TerminalEditor};

// "a/" and "b/" are prefixes specific to Git Diff
#[cfg(feature = "local_fs")]
const PREFIXES_TO_REMOVE: [&str; 2] = ["a/", "b/"];

/// "@" is a suffix that can be added to symlinks. It appears in Git Bash's default configuration
/// for `ls`.
#[cfg(feature = "local_fs")]
const SUFFIXES_TO_REMOVE: [&str; 1] = ["@"];

/// Highlighted link within a terminal model grid.
#[derive(Debug, Clone)]
pub enum GridHighlightedLink {
    Url(WithinModel<Link>),
    #[cfg(feature = "local_fs")]
    File(WithinModel<FileLink>),
}

impl GridHighlightedLink {
    pub fn contains(&self, position: &WithinModel<Point>) -> bool {
        match self {
            GridHighlightedLink::Url(url) => url.contains(position),
            #[cfg(feature = "local_fs")]
            GridHighlightedLink::File(file_link) => file_link.contains(position),
        }
    }

    pub fn tooltip_text(&self) -> String {
        match &self {
            #[cfg(feature = "local_fs")]
            GridHighlightedLink::File(file_link)
                if file_link
                    .get_inner()
                    .absolute_path()
                    .map(|path| path.is_dir())
                    .unwrap_or(false) =>
            {
                crate::t!("common-open-folder")
            }
            #[cfg(feature = "local_fs")]
            GridHighlightedLink::File(_) => crate::t!("common-open-file"),
            GridHighlightedLink::Url(_) => crate::t!("common-open-link"),
        }
    }
}

impl Serialize for GridHighlightedLink {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            GridHighlightedLink::Url(_) => {
                serializer.serialize_unit_variant("HighlightedLink", 0, "Url")
            }
            #[cfg(feature = "local_fs")]
            GridHighlightedLink::File(_) => {
                serializer.serialize_unit_variant("HighlightedLink", 1, "File")
            }
        }
    }
}

impl TryFrom<GridHighlightedLink> for Link {
    type Error = anyhow::Error;

    fn try_from(value: GridHighlightedLink) -> Result<Self, Self::Error> {
        match value {
            GridHighlightedLink::Url(WithinModel::AltScreen(url)) => Ok(url),
            #[cfg(feature = "local_fs")]
            GridHighlightedLink::File(WithinModel::AltScreen(file_link)) => Ok(file_link.link),
            _ => Err(anyhow::anyhow!(
                "HighlightedLink is not within the alt screen"
            )),
        }
    }
}

impl TryFrom<GridHighlightedLink> for WithinBlock<Link> {
    type Error = anyhow::Error;

    fn try_from(value: GridHighlightedLink) -> Result<Self, Self::Error> {
        match value {
            GridHighlightedLink::Url(WithinModel::BlockList(url)) => Ok(url),
            #[cfg(feature = "local_fs")]
            GridHighlightedLink::File(WithinModel::BlockList(file_link)) => {
                Ok(file_link.map(|file_link| file_link.link))
            }
            _ => Err(anyhow::anyhow!(
                "HighlightedLink is not within the block list"
            )),
        }
    }
}

/// The highlighted_link state is synced with both the BlockList and AltScreen so that they can
/// use the highlighted_link to override the normal smart-selection behavior. The
/// highlighted_link can, for example, verify that a file path actually exists on disk, and
/// include file paths with spaces. Smart-select can do neither of those things.
/// Since this value must be kept in sync, we need to prevent any mutation of the value outside
/// of this wrapper.
#[derive(Debug, Default)]
pub struct HighlightedLinkOption {
    inner: Option<GridHighlightedLink>,
    /// True if the underlying content has changed such that the link may no longer be valid.
    invalidated: bool,
}

#[derive(Clone, Debug)]
pub enum RichContentLink {
    Url(String),
    #[cfg(feature = "local_fs")]
    FilePath {
        absolute_path: PathBuf,
        line_and_column_num: Option<LineAndColumnArg>,
        target_override: Option<FileTarget>,
    },
}

impl RichContentLink {
    pub fn tooltip_text(&self) -> String {
        match &self {
            #[cfg(feature = "local_fs")]
            RichContentLink::FilePath { absolute_path, .. } if absolute_path.is_dir() => {
                crate::t!("common-open-folder")
            }
            #[cfg(feature = "local_fs")]
            RichContentLink::FilePath { .. } => crate::t!("common-open-file"),
            RichContentLink::Url(_) => crate::t!("common-open-link"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RichContentLinkTooltipInfo {
    pub link: RichContentLink,
    pub position_id: String,
}

impl HighlightedLinkOption {
    /// Assigns the inner value and syncs it with the BlockList and AltScreen
    pub fn set(&mut self, link: GridHighlightedLink, model: &mut TerminalModel) {
        match &link {
            GridHighlightedLink::Url(within_model) => match within_model {
                WithinModel::BlockList(within_block) => {
                    let point_range = WithinBlock::new(
                        within_block.inner.range.clone(),
                        within_block.block_index,
                        within_block.grid,
                    );
                    model
                        .block_list_mut()
                        .set_smart_select_override(point_range);
                }
                WithinModel::AltScreen(link) => {
                    model
                        .alt_screen_mut()
                        .set_smart_select_override(link.range.clone());
                }
            },
            #[cfg(feature = "local_fs")]
            GridHighlightedLink::File(within_model) => match within_model {
                WithinModel::BlockList(within_block) => {
                    let point_range = WithinBlock::new(
                        within_block.inner.link.range.clone(),
                        within_block.block_index,
                        within_block.grid,
                    );
                    model
                        .block_list_mut()
                        .set_smart_select_override(point_range);
                }
                WithinModel::AltScreen(file_link) => {
                    model
                        .alt_screen_mut()
                        .set_smart_select_override(file_link.link.range.clone());
                }
            },
        }
        self.inner = Some(link);
    }

    /// Wrapper method for Option::take that also keeps the derived state in the BlockList and
    /// AltScreen in sync
    pub fn take(&mut self, model: &mut TerminalModel) -> Option<GridHighlightedLink> {
        model.block_list_mut().clear_smart_select_override();
        model.alt_screen_mut().clear_smart_select_override();
        self.invalidated = false;
        self.inner.take()
    }

    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    pub fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    pub fn clone_inner(&self) -> Option<GridHighlightedLink> {
        self.inner.clone()
    }
}

impl Deref for HighlightedLinkOption {
    type Target = Option<GridHighlightedLink>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl super::TerminalView {
    pub(super) fn maybe_link_hover(
        &mut self,
        position: &Option<WithinModel<Point>>,
        from_editor: TerminalEditor,
        ctx: &mut ViewContext<Self>,
    ) {
        // Do not highlight the url while selecting text or blocks, or if the window is not active.
        if self.terminal_is_selecting(&self.model.lock(), ctx)
            || self.is_navigated_away_from_window(ctx)
        {
            if self.highlighted_link.take(&mut self.model.lock()).is_some() {
                ctx.reset_cursor();
                ctx.notify();
            }
            return;
        }

        // If the mouse isn't in the terminal view, we're not hovering any link.
        let Some(position) = position else {
            if self.highlighted_link.take(&mut self.model.lock()).is_some() {
                ctx.reset_cursor();
                // Clear last_hover_fragment_boundary when mouse is out of block bounds.
                self.last_hover_fragment_boundary = None;
                ctx.notify();
            }
            return;
        };

        // If the mouse is still on top of the previous highlighted link and that link is
        // still valid, we can keep highlighting it.
        if let Some(link) = self.highlighted_link.as_ref() {
            if link.contains(position) && !self.highlighted_link.is_invalidated() {
                // If already hovering on a highlighted link, return.
                return;
            }
        }

        // Updating the cursor shape repeatedly can cause flashing, so we only set it once, and only
        // when necessary.
        let mut new_cursor_shape = None;

        // If a link is highlighted and it's invalidated or we're not hovering it, remove that
        // hover and look for a new one.
        if self.highlighted_link.is_some() {
            // Remove the current highlighted link because we are no longer
            // hovering over it.
            self.highlighted_link.take(&mut self.model.lock());
            new_cursor_shape = Some(Cursor::Arrow);
        }

        let (url_at_point, new_fragment_boundary) = {
            let model = self.model.lock();
            (
                model.url_at_point(position),
                model.fragment_boundary_at_point(position),
            )
        };

        match (url_at_point, &self.last_hover_fragment_boundary) {
            (Some(url), _) => {
                self.highlighted_link
                    .set(GridHighlightedLink::Url(url), &mut self.model.lock());
                new_cursor_shape = Some(Cursor::PointingHand);
            }
            // Only scan for links if the mouse hovered on a new word.
            (_, Some(last_hover_fragment_boundary))
                if !last_hover_fragment_boundary.contains(position) =>
            {
                // Use try_send to return an error directly when the channel is full
                // instead of blocking main thread.
                let _ = self.find_link_tx.try_send(FindLinkArg {
                    position: *position,
                    from_editor,
                });
            }
            // If there's no last hover fragment boundary, we scan for links.
            (_, None) => {
                let _ = self.find_link_tx.try_send(FindLinkArg {
                    position: *position,
                    from_editor,
                });
            }
            _ => (),
        };

        if let Some(new_cursor_shape) = new_cursor_shape {
            ctx.set_cursor_shape(new_cursor_shape);
            ctx.notify();
        }

        self.last_hover_fragment_boundary = Some(new_fragment_boundary);
    }

    #[cfg_attr(not(feature = "local_fs"), allow(unused_variables))]
    pub(super) fn handle_find_link(
        &mut self,
        find_link_arg: FindLinkArg,
        ctx: &mut ViewContext<Self>,
    ) {
        let FindLinkArg {
            position,
            from_editor,
        } = find_link_arg;

        // Already highlighted the hovered link, returning.
        if self
            .highlighted_link
            .as_ref()
            .is_some_and(|url| url.contains(&position))
        {
            #[cfg_attr(not(feature = "local_fs"), allow(clippy::needless_return))]
            return;
        }

        #[cfg(feature = "local_fs")]
        self.scan_for_file_path(position, from_editor, ctx);
    }

    pub(super) fn open_highlighted_link(
        &mut self,
        link: &GridHighlightedLink,
        ctx: &mut ViewContext<Self>,
    ) {
        self.dismiss_tooltips(ctx);
        ctx.focus(&self.input);
        ctx.notify();

        send_telemetry_from_ctx!(
            TelemetryEvent::OpenLink {
                link: link.clone(),
                open_with: LinkOpenMethod::ToolTip
            },
            ctx
        );
        match link {
            #[cfg(feature = "local_fs")]
            GridHighlightedLink::File(link) => {
                let link = link.get_inner();
                if let Some(path) = link.absolute_path() {
                    self.open_file_path(path.clone(), link.line_and_column_num, ctx);
                }
            }
            GridHighlightedLink::Url(url) => {
                let model = self.model.lock();
                ctx.open_url(&model.link_at_range(url, RespectObfuscatedSecrets::No));
            }
        };
    }

    pub(super) fn open_rich_content_link(
        &mut self,
        link: &RichContentLink,
        ctx: &mut ViewContext<Self>,
    ) {
        self.dismiss_tooltips(ctx);
        ctx.focus(&self.input);
        ctx.notify();

        match link {
            #[cfg(feature = "local_fs")]
            RichContentLink::FilePath {
                absolute_path,
                line_and_column_num,
                target_override,
            } => {
                if let Some(target_override) = target_override {
                    self.open_file_path_with_target(
                        absolute_path.clone(),
                        target_override.clone(),
                        *line_and_column_num,
                        ctx,
                    );
                } else {
                    self.open_file_path(absolute_path.clone(), *line_and_column_num, ctx);
                }
            }
            RichContentLink::Url(url) => {
                ctx.open_url(url);
            }
        };
    }
}

// A collection of link detection functions that are only valid on platforms
// where we can spawn a local tty.
#[cfg(feature = "local_fs")]
impl super::TerminalView {
    /// Zap:判断给定会话是否是 remote-server(SSH)会话。
    ///
    /// 当 `local_tty` 未启用 / 在 wasm 上 / `SshRemoteServer` feature flag 关闭时,
    /// 一律返回 `false`,即保持本地行为完全不变。
    fn session_is_remote(
        &self,
        session_id: Option<crate::terminal::model::session::SessionId>,
        ctx: &warpui::AppContext,
    ) -> bool {
        #[cfg(all(feature = "local_tty", not(target_family = "wasm")))]
        {
            use warpui::SingletonEntity as _;

            use crate::features::FeatureFlag;
            use crate::remote_server::manager::RemoteServerManager;

            if FeatureFlag::SshRemoteServer.is_enabled() {
                if let Some(session_id) = session_id {
                    return RemoteServerManager::handle(ctx)
                        .as_ref(ctx)
                        .host_id_for_session(session_id)
                        .is_some();
                }
            }
        }

        let _ = (session_id, ctx);
        false
    }

    /// Zap:取得远端会话某个 cwd 的目录列表校验上下文。
    ///
    /// 命中缓存则直接返回 `Remote(Some(..))`;未命中则异步发起 daemon
    /// `ListDirectory` RPC 拉取该目录列表,本轮返回 `Remote(None)`(不高亮),
    /// 拉取完成后写入缓存并 `ctx.notify()` 触发 re-render 把链接点亮。
    ///
    /// 缓存保持有界:拉取新 cwd 时清掉所有旧条目,只保留当前 cwd。
    #[cfg(all(
        feature = "local_tty",
        feature = "local_fs",
        not(target_family = "wasm")
    ))]
    fn remote_dir_listing_context(
        &mut self,
        session_id: crate::terminal::model::session::SessionId,
        cwd: &str,
        ctx: &mut ViewContext<Self>,
    ) -> crate::util::file::LinkValidationContext {
        use std::path::PathBuf;
        use std::sync::Arc;

        use warpui::SingletonEntity as _;

        use crate::remote_server::manager::RemoteServerManager;
        use crate::util::file::{LinkValidationContext, RemoteDirListing};

        let cwd_path = PathBuf::from(cwd);
        // 缓存按 (会话, cwd) 复合键索引,避免不同 host 的相同路径互相串扰。
        let cache_key = (session_id, cwd_path.clone());

        // 命中缓存(已就绪或拉取中)直接返回。
        if let Some(entry) = self.remote_dir_listing_cache.get(&cache_key) {
            return LinkValidationContext::Remote(entry.clone());
        }

        // 取该会话的 daemon 客户端。
        let Some(client) = RemoteServerManager::handle(ctx)
            .as_ref(ctx)
            .client_for_session(session_id)
            .cloned()
        else {
            return LinkValidationContext::Remote(None);
        };

        // 拉取新 cwd:超出容量上限时按插入顺序 FIFO 淘汰最旧条目,
        // 然后插入 `None` 占位(标记拉取中)。`MAX_ENTRIES` 选 8 足够覆盖
        // 用户在终端里常切的几个工作目录,避免每次切回都要 RPC。
        const MAX_ENTRIES: usize = 8;
        while self.remote_dir_listing_cache.len() >= MAX_ENTRIES {
            // shift_remove_index 保持插入顺序;FIFO 头部是最旧的。
            self.remote_dir_listing_cache.shift_remove_index(0);
        }
        self.remote_dir_listing_cache
            .insert(cache_key.clone(), None);

        let cwd_for_request = cwd.to_string();
        let cwd_for_store = cwd_path.clone();
        let key_for_store = cache_key.clone();
        ctx.spawn(
            async move { client.list_directory(cwd_for_request).await },
            move |me, result, ctx| {
                use crate::remote_server::proto::list_directory_response;

                // 拉取期间用户可能已经切换 cwd / 清空缓存,只有占位还在才写入。
                if !me.remote_dir_listing_cache.contains_key(&key_for_store) {
                    return;
                }
                match result {
                    Ok(resp) => match resp.result {
                        Some(list_directory_response::Result::Success(success)) => {
                            let entries = success
                                .entries
                                .into_iter()
                                .map(|e| (e.name, e.is_dir))
                                .collect();
                            let listing =
                                Arc::new(RemoteDirListing::new(cwd_for_store.clone(), entries));
                            me.remote_dir_listing_cache
                                .insert(key_for_store.clone(), Some(listing));
                            // 列表到达,触发 re-render 让链接重新扫描并点亮。
                            ctx.notify();
                        }
                        Some(list_directory_response::Result::Error(err)) => {
                            log::warn!(
                                "远端 ListDirectory 失败 {cwd_for_store:?}: {}",
                                err.message
                            );
                            // 拉取失败:移除占位,下次悬停时会重试。
                            me.remote_dir_listing_cache.shift_remove(&key_for_store);
                        }
                        None => {
                            me.remote_dir_listing_cache.shift_remove(&key_for_store);
                        }
                    },
                    Err(err) => {
                        log::warn!("远端 ListDirectory RPC 出错 {cwd_for_store:?}: {err}");
                        me.remote_dir_listing_cache.shift_remove(&key_for_store);
                    }
                }
            },
        );

        LinkValidationContext::Remote(None)
    }

    /// Scans the terminal model at the given position to see if it is
    /// contained within a path that should be linkified.
    fn scan_for_file_path(
        &mut self,
        position: WithinModel<Point>,
        from_editor: TerminalEditor,
        ctx: &mut ViewContext<Self>,
    ) {
        use crate::util::file::LinkValidationContext;

        // Zap:判断被悬停 block 所属会话是否是 remote-server 会话。
        // 远端会话的文件不在本地磁盘上,需要用 `LinkValidationContext::Remote`
        // 携带 daemon 拉取来的真实目录列表做精确校验。
        let block_session_id = match position {
            WithinModel::AltScreen(_) => self.active_block_session_id(),
            WithinModel::BlockList(inner) => self
                .model
                .lock()
                .block_list()
                .block_at(inner.block_index)
                .and_then(|block| block.session_id()),
        };
        let is_remote = self.session_is_remote(block_session_id, ctx);

        // For AltScreen we scan for relative path with the current working directory.
        // For BlockList we scan for relative path with the pwd of the hovered block.
        //
        // Zap:远端会话的 block `pwd()` 是 shell-integration 上报的远端 cwd,
        // 拼接后即得到正确的远端绝对路径,因此远端 block 也参与扫描(不再跳过)。
        let pwd_to_scan_for = match position {
            WithinModel::AltScreen(_) => {
                if is_remote {
                    // 远端会话:`pwd()` 返回的是 shell-integration 上报的远端活动 cwd。
                    self.pwd()
                } else {
                    self.pwd_if_local(ctx)
                }
            }
            WithinModel::BlockList(inner) => self
                .model
                .lock()
                .block_list()
                .block_at(inner.block_index)
                .and_then(|block| block.pwd().map(String::from)),
        };

        // Zap:远端会话用缓存的 cwd 目录列表精确校验;本地会话保持 `Local`。
        let validation_ctx = match (&pwd_to_scan_for, block_session_id) {
            #[cfg(all(feature = "local_tty", not(target_family = "wasm")))]
            (Some(cwd), Some(session_id)) if is_remote => {
                self.remote_dir_listing_context(session_id, cwd, ctx)
            }
            _ => LinkValidationContext::Local,
        };

        match pwd_to_scan_for {
            // Check if we are hovering on any file path. Don't scan for file path
            // if user is hovering from an editor like vim or nano.
            Some(path) if matches!(from_editor, TerminalEditor::No) => {
                let possible_paths = self.model.lock().possible_file_paths_at_point(position);
                let max_columns = self.size_info.columns;
                // 用被悬停 block 自己的 launch data,避免跨会话/host/WSL 时
                // 用错 shell 规则解析路径。
                let shell_launch_data = block_session_id
                    .and_then(|session_id| self.sessions.as_ref(ctx).get(session_id))
                    .and_then(|session| session.launch_data().cloned());

                // Using the thread builder instead of ctx.spawn here so that the previous
                // scanning job will be dropped once there is a new scanning job created.
                let (tx, rx) = futures::channel::oneshot::channel();
                self.file_link_scanning_join_handle = std::thread::Builder::new()
                    .name("Compute file paths".into())
                    .spawn(move || {
                        let paths = Self::compute_valid_paths(
                            &path,
                            possible_paths,
                            max_columns,
                            shell_launch_data,
                            validation_ctx,
                        );
                        let _ = tx.send(paths);
                    })
                    .map_err(|e| {
                        log::error!("Unable to spawn thread {e:?}");
                    })
                    .ok();

                let _ = ctx.spawn(
                    async move { rx.await.ok().flatten() },
                    Self::handle_file_link_completed,
                );
            }
            _ if self.highlighted_link.take(&mut self.model.lock()).is_some() => {
                ctx.reset_cursor();
                ctx.notify();
            }
            _ => (),
        };
    }

    fn compute_valid_paths(
        working_directory: &str,
        possible_paths: impl Iterator<Item = WithinModel<grid_handler::PossiblePath>>,
        max_columns: usize,
        shell_launch_data: Option<ShellLaunchData>,
        validation_ctx: crate::util::file::LinkValidationContext,
    ) -> Option<GridHighlightedLink> {
        let mut link = None;
        'path_loop: for within_model_possible_path in possible_paths {
            let possible_path = within_model_possible_path.get_inner();
            // We want to check if the clean path result is a valid path and get the canonical
            // absolute path back.
            let absolute_path = absolute_path_if_valid(
                &possible_path.path,
                ShellPathType::ShellNative(working_directory.to_string()),
                shell_launch_data.as_ref(),
                &validation_ctx,
            );

            if let Some(absolute_path) = absolute_path {
                link = Some(Self::create_valid_link(
                    absolute_path,
                    possible_path.path.line_and_column_num,
                    possible_path.range.clone(),
                    &within_model_possible_path,
                ));
                break;
            }

            for prefix in PREFIXES_TO_REMOVE {
                if let Some(new_possible_path) = possible_path.path.path.strip_prefix(prefix) {
                    let new_possible_cleaned_path = CleanPathResult {
                        path: new_possible_path.into(),
                        line_and_column_num: possible_path.path.line_and_column_num,
                    };
                    let absolute_path = absolute_path_if_valid(
                        &new_possible_cleaned_path,
                        ShellPathType::ShellNative(working_directory.to_string()),
                        shell_launch_data.as_ref(),
                        &validation_ctx,
                    );

                    // check if new_possible_path is valid
                    if let Some(absolute_path) = absolute_path {
                        let new_start_point = possible_path
                            .range
                            .start()
                            .wrapping_add(max_columns, prefix.len());

                        link = Some(Self::create_valid_link(
                            absolute_path,
                            new_possible_cleaned_path.line_and_column_num,
                            new_start_point..=*possible_path.range.end(),
                            &within_model_possible_path,
                        ));

                        // break outer_loop
                        break 'path_loop;
                    }
                }
            }

            for suffix in SUFFIXES_TO_REMOVE {
                if let Some(new_possible_path) = possible_path.path.path.strip_suffix(suffix) {
                    let new_possible_cleaned_path = CleanPathResult {
                        path: new_possible_path.into(),
                        line_and_column_num: possible_path.path.line_and_column_num,
                    };
                    let absolute_path = absolute_path_if_valid(
                        &new_possible_cleaned_path,
                        ShellPathType::ShellNative(working_directory.to_string()),
                        shell_launch_data.as_ref(),
                        &validation_ctx,
                    );

                    // check if new_possible_path is valid
                    if let Some(absolute_path) = absolute_path {
                        let new_end_point = possible_path
                            .range
                            .end()
                            .wrapping_sub(max_columns, suffix.len());

                        link = Some(Self::create_valid_link(
                            absolute_path,
                            new_possible_cleaned_path.line_and_column_num,
                            *possible_path.range.start()..=new_end_point,
                            &within_model_possible_path,
                        ));

                        // break outer_loop
                        break 'path_loop;
                    }
                }
            }
        }

        link.map(GridHighlightedLink::File)
    }

    fn create_valid_link(
        absolute_path: PathBuf,
        line_and_column_num: Option<LineAndColumnArg>,
        path_range: std::ops::RangeInclusive<Point>,
        possible_path: &WithinModel<grid_handler::PossiblePath>,
    ) -> WithinModel<FileLink> {
        let inner_link = FileLink {
            link: Link {
                range: path_range,
                is_empty: false,
            },
            absolute_path,
            line_and_column_num,
        };

        match possible_path {
            WithinModel::AltScreen(_) => WithinModel::AltScreen(inner_link),
            WithinModel::BlockList(inner) => {
                WithinModel::BlockList(WithinBlock::new(inner_link, inner.block_index, inner.grid))
            }
        }
    }

    fn handle_file_link_completed(
        &mut self,
        link_result: Option<GridHighlightedLink>,
        ctx: &mut ViewContext<Self>,
    ) {
        let mut model = self.model.lock();
        if self.highlighted_link.take(&mut model).is_some() {
            ctx.reset_cursor();
            ctx.notify();
        }

        if let Some(new_link) = link_result {
            self.highlighted_link.set(new_link, &mut model);
            ctx.set_cursor_shape(Cursor::PointingHand);
            ctx.notify();
        }
    }
}
