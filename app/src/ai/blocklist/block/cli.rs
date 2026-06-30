use parking_lot::{FairMutex, RwLock};
use pathfinder_color::ColorU;
use settings::Setting as _;
use std::time::Duration;
use std::{cell::RefCell, cmp::Ordering, rc::Rc, sync::Arc};
use warp_core::features::FeatureFlag;
use warp_core::report_error;
use warp_core::ui::theme::color::internal_colors;
use warpui::elements::new_scrollable::SingleAxisConfig;
use warpui::elements::{
    resizable_state_handle, ClippedScrollStateHandle, ConstrainedBox, DispatchEventResult,
    DragBarSide, Empty, EventHandler, Fill, FormattedTextElement, Highlight, HighlightedHyperlink,
    Hoverable, MainAxisAlignment, MainAxisSize, NewScrollable, Resizable, ResizableStateHandle,
    SavePosition, ScrollTarget, ScrollToPositionMode, SelectableArea, SizeConstraintCondition,
    SizeConstraintSwitch,
};
use warpui::fonts::Weight;
use warpui::platform::{Cursor, OperatingSystem};
use warpui::ui_components::components::{Coords, UiComponent, UiComponentStyles};
use warpui::units::IntoPixels;

use lazy_static::lazy_static;
use pathfinder_geometry::vector::vec2f;

use markdown_parser::{FormattedText, FormattedTextFragment, FormattedTextLine};
use warp_core::semantic_selection::SemanticSelection;
use warp_core::ui::appearance::Appearance;
use warp_editor::{
    content::buffer::InitialBufferState, render::element::VerticalExpansionBehavior,
};
use warpui::r#async::Timer;
use warpui::{
    clipboard::ClipboardContent,
    elements::{
        Border, ChildAnchor, ChildView, Container, CornerRadius, CrossAxisAlignment, DropShadow,
        Expanded, Flex, MouseStateHandle, OffsetPositioning, ParentElement,
        PositionedElementAnchor, PositionedElementOffsetBounds, Radius, SelectionHandle,
        Shrinkable, Stack, Text,
    },
    fonts::{Properties, Style},
    keymap::{EditableBinding, Keystroke},
    r#async::SpawnedFutureHandle,
    AppContext, Element, Entity, EntityId, ModelHandle, SingletonEntity, TypedActionView, View,
    ViewContext, ViewHandle,
};

use crate::ai::agent::{AIAgentPtyWriteMode, CancellationReason};
use crate::ai::blocklist::block::view_impl::common::{
    render_query_text, UserQueryProps, BLOCKED_ACTION_MESSAGE_FOR_GREP_OR_FILE_GLOB,
    BLOCKED_ACTION_MESSAGE_FOR_READING_FILES,
    BLOCKED_ACTION_MESSAGE_FOR_WRITE_TO_LONG_RUNNING_SHELL_COMMAND,
    LOAD_OUTPUT_MESSAGE_FOR_FILE_GLOB, LOAD_OUTPUT_MESSAGE_FOR_GREP,
    LOAD_OUTPUT_MESSAGE_FOR_READING_FILES, LOAD_OUTPUT_MESSAGE_FOR_WEB_SEARCH,
};
use crate::ai::blocklist::permissions::is_agent_mode_autonomy_allowed;
use crate::ai::control_code_parser::{parse_control_codes_from_bytes, ParsedControlCodeOutput};
use crate::code::editor::view::{CodeEditorEvent, CodeEditorRenderOptions};
use crate::menu::MenuItemFields;
use crate::send_telemetry_from_ctx;
use crate::server::telemetry::TelemetryEvent;
use crate::settings::{AISettings, SelectionSettings};
use crate::terminal::input::SET_INPUT_MODE_TERMINAL_ACTION_NAME;
use crate::terminal::model::block::BlockId;
use crate::terminal::{ShellLaunchData, TerminalModel};
use crate::view_components::DismissibleToast;
use crate::workspace::WorkspaceAction;
use crate::ToastStack;
use crate::{
    ai::{
        agent::{
            conversation::AIConversationId, task::TaskId, AIAgentActionType, AIAgentExchangeId,
            AIAgentOutput, AIAgentOutputMessageType, AIAgentText, AIAgentTextSection,
            ProgrammingLanguage, WebSearchStatus,
        },
        blocklist::{
            code_block::CodeSnippetButtonHandles, BlocklistAIActionModel, BlocklistAIHistoryEvent,
            BlocklistAIPermissions,
        },
        execution_profiles::profiles::{AIExecutionProfilesModel, AIExecutionProfilesModelEvent},
    },
    code::{editor::view::CodeEditorView, editor_management::CodeSource},
    editor::InteractionState,
    menu::{Event as MenuEvent, Menu, MenuVariant},
    settings_view::SettingsSection,
    terminal::safe_mode_settings::get_secret_obfuscation_mode,
    ui_components::{blended_colors, icons::Icon},
    view_components::{
        action_button::{ButtonSize, KeystrokeSource, NakedTheme, PrimaryTheme},
        compactible_action_button::{
            render_compact_and_regular_button_rows, CompactibleActionButton,
            RenderCompactibleActionButton,
        },
        compactible_split_action_button::CompactibleSplitActionButton,
    },
    BlocklistAIHistoryModel,
};

use crate::ai::agent::AIAgentInput;
use crate::ai::blocklist::block::TextLocation;
use crate::util::link_detection::{detect_links, DetectedLinksState};

use crate::ai::agent::icons::yellow_stop_icon;
use crate::ai::blocklist::inline_action::inline_action_icons::icon_size;

use super::cli_controller::{CLISubagentController, CLISubagentEvent, UserTakeOverReason};
use super::model::AIBlockModelHelper;
use super::TableSectionHandles;
use super::{
    model::{AIBlockModel, AIBlockModelImpl, AIBlockOutputStatus},
    view_impl::{
        common::{
            render_debug_footer, render_failed_output, render_informational_footer,
            render_text_sections, DebugFooterProps, FailedOutputProps, TextSectionsProps,
        },
        output::are_all_text_sections_empty,
    },
    EmbeddedCodeEditorView, SecretRedactionState,
};
const MENU_WIDTH: f32 = 200.0;
const MAX_HEIGHT: f32 = 320.0;
// CLI agent 浮窗的最小宽度，避免内容被拖到不可读；外层布局也复用该值保持约束一致。
pub(crate) const CLI_SUBAGENT_MIN_RESIZABLE_WIDTH: f32 = 360.0;
const MIN_RESIZABLE_WIDTH: f32 = CLI_SUBAGENT_MIN_RESIZABLE_WIDTH;
// CLI agent 浮窗的最小高度，保留一行以上内容和拖拽命中区域。
const MIN_RESIZABLE_HEIGHT: f32 = 40.0;
// 横向缩放时给窗口边缘保留的少量可见宽度。
const MIN_REMAINING_WINDOW_WIDTH: f32 = 16.0;
// 纵向缩放时给窗口边缘保留的少量可见高度。
const MIN_REMAINING_WINDOW_HEIGHT: f32 = 16.0;
const AVATAR_RIGHT_MARGIN: f32 = 8.;
const CONTENT_PADDING: f32 = 12.;
const ALLOW_ACTION_POSITION_ID: &str = "allow-action-position-id";
const USER_QUERY_POSITION_ID: &str = "cli-subagent-user-query-position-id";
const CONVERSATION_SCROLL_BOTTOM_POSITION_ID: &str = "cli-subagent-conversation-bottom-position-id";

fn cli_subagent_width_bounds(window_width: f32) -> (f32, f32) {
    // 最大宽度接近整窗，允许右下角浮窗向左覆盖大部分终端区域。
    let max = (window_width - MIN_REMAINING_WINDOW_WIDTH).max(MIN_RESIZABLE_WIDTH);
    (MIN_RESIZABLE_WIDTH, max)
}

fn cli_subagent_height_bounds(window_height: f32) -> (f32, f32) {
    // 最大高度接近整窗，允许右下角浮窗向上覆盖大部分终端区域。
    let max = (window_height - MIN_REMAINING_WINDOW_HEIGHT).max(MIN_RESIZABLE_HEIGHT);
    (MIN_RESIZABLE_HEIGHT, max)
}

/// 追加 CLI agent 浮窗历史 exchange id，并忽略重复事件。
fn cli_subagent_append_history_exchange_id(
    exchange_ids: &mut Vec<AIAgentExchangeId>,
    exchange_id: AIAgentExchangeId,
) -> bool {
    if exchange_ids.contains(&exchange_id) {
        return false;
    }

    exchange_ids.push(exchange_id);
    true
}

/// 用户滚轮查看历史后，不再强制把整体对话滚回底部。
fn cli_subagent_mark_conversation_scroll_manually_moved(is_pinned: &mut bool) {
    *is_pinned = false;
}

/// 根据滚轮方向更新贴底状态：已在底部继续向下滚动时保持 auto-scroll。
fn cli_subagent_update_conversation_scroll_pin_after_wheel(
    is_pinned: &mut bool,
    vertical_delta: f32,
) -> bool {
    if *is_pinned && vertical_delta <= 0. {
        return false;
    }

    if *is_pinned {
        cli_subagent_mark_conversation_scroll_manually_moved(is_pinned);
        return true;
    }

    false
}

/// 新一轮 exchange 追加时，默认恢复跟随最新内容。
fn cli_subagent_mark_conversation_scroll_should_follow_latest(is_pinned: &mut bool) {
    *is_pinned = true;
}

/// 切换响应可见性时保存或恢复对话滚动位置。
fn cli_subagent_response_visibility_scroll_offset(
    current_scroll_offset: f32,
    should_hide_responses: bool,
    saved_scroll_offset: &mut Option<f32>,
) -> Option<f32> {
    if should_hide_responses {
        *saved_scroll_offset = Some(current_scroll_offset);
        None
    } else {
        saved_scroll_offset.take()
    }
}

/// 浮窗内滚轮事件只消费在整体对话窗口中，避免继续带动外层终端滚动。
fn cli_subagent_conversation_scroll_wheel_dispatch_result() -> DispatchEventResult {
    DispatchEventResult::StopPropagation
}

/// 判断 CLI 浮窗中的用户输入气泡是否应该渲染。
fn cli_subagent_should_render_user_input(
    should_hide_responses: bool,
    is_input_dismissed: bool,
) -> bool {
    !should_hide_responses && !is_input_dismissed
}

/// 统计 CLI 浮窗实际渲染的 output sections，供脱敏索引与 render 累计保持一致。
fn cli_subagent_rendered_output_text_section_count(output: &AIAgentOutput) -> usize {
    output
        .messages
        .iter()
        .filter_map(|output_message| {
            if let AIAgentOutputMessageType::Text(AIAgentText { sections }) =
                &output_message.message
            {
                if !are_all_text_sections_empty(sections) {
                    return Some(sections.len());
                }
            }

            None
        })
        .sum()
}

/// 按 CLI 浮窗 render 顺序扫描可见 output，并返回参与索引累计的 section 数。
fn cli_subagent_run_redaction_on_rendered_output(
    secret_redaction_state: &mut SecretRedactionState,
    output: &AIAgentOutput,
    starting_section_index: usize,
    should_obfuscate: bool,
) -> usize {
    let mut scanned_section_count = 0;

    for output_message in output.messages.iter() {
        if let AIAgentOutputMessageType::Text(AIAgentText { sections }) = &output_message.message {
            if !are_all_text_sections_empty(sections) {
                scanned_section_count += secret_redaction_state
                    .run_redaction_on_text_sections_with_starting_section_index(
                        sections,
                        starting_section_index + scanned_section_count,
                        should_obfuscate,
                    );
            }
        }
    }

    debug_assert_eq!(
        scanned_section_count,
        cli_subagent_rendered_output_text_section_count(output)
    );

    scanned_section_count
}

lazy_static! {
    static ref ACCEPT_KEYSTROKE: Keystroke = Keystroke {
        key: "enter".to_owned(),
        ..Default::default()
    };
    static ref REJECT_KEYSTROKE: Keystroke =
        Keystroke::parse("ctrl-c").expect("Failed to parse take over keystroke");
    static ref AUTO_APPROVE_KEYSTROKE: Keystroke = {
        let binding = if OperatingSystem::get().is_mac() {
            "cmd-shift-I"
        } else {
            "ctrl-shift-I"
        };
        Keystroke::parse(binding).expect("Failed to parse auto approve keystroke")
    };
}

const HAS_PENDING_CLI_ACTION_CONTEXT_KEY: &str = "HasPendingCLIAgentAction";
const HAS_PENDING_NON_TRANSFER_CONTROL_ACTION_CONTEXT_KEY: &str =
    "HasPendingNonTransferControlCLIAgentAction";
const BLOCKED_ACTION_MESSAGE_FOR_TRANSFER_CONTROL: &str = "Agent is asking you to take control.";

pub fn init(app: &mut AppContext) {
    use warpui::keymap::{macros::*, FixedBinding};

    app.register_fixed_bindings([
        FixedBinding::new(
            ACCEPT_KEYSTROKE.normalized(),
            CLISubagentAction::ExecuteBlockedAction,
            id!(CLISubagentView::ui_name()) & id!(HAS_PENDING_CLI_ACTION_CONTEXT_KEY),
        ),
        FixedBinding::new(
            REJECT_KEYSTROKE.normalized(),
            CLISubagentAction::RejectBlockedAction {
                should_user_take_over: false,
            },
            id!(CLISubagentView::ui_name()) & id!(HAS_PENDING_CLI_ACTION_CONTEXT_KEY),
        ),
        FixedBinding::new(
            "escape",
            CLISubagentAction::RejectBlockedAction {
                should_user_take_over: true,
            },
            id!(CLISubagentView::ui_name())
                & id!(HAS_PENDING_NON_TRANSFER_CONTROL_ACTION_CONTEXT_KEY),
        ),
        FixedBinding::new(
            AUTO_APPROVE_KEYSTROKE.normalized(),
            CLISubagentAction::ExecuteAndAutoApprove,
            id!(CLISubagentView::ui_name())
                & id!(HAS_PENDING_NON_TRANSFER_CONTROL_ACTION_CONTEXT_KEY),
        ),
    ]);
    app.register_editable_bindings([EditableBinding::new(
        SET_INPUT_MODE_TERMINAL_ACTION_NAME,
        crate::t!("keybinding-desc-take-control-of-running-command"),
        CLISubagentAction::TakeControlOfRunningCommand,
    )
    .with_mac_key_binding("cmd-i")
    .with_linux_or_windows_key_binding("ctrl-i")
    .with_context_predicate(
        id!(CLISubagentView::ui_name()) & id!(HAS_PENDING_CLI_ACTION_CONTEXT_KEY),
    )]);
}

type SelectionHandleList = Rc<RefCell<Vec<SelectionHandle>>>;

#[derive(Clone, Copy)]
enum SelectionHandleGroup {
    Query,
    Output,
    Action,
}

/// 按渲染项索引获取独立的选择状态，避免多个气泡共用同一套划词范围。
fn selection_handle_for_index(handles: &SelectionHandleList, index: usize) -> SelectionHandle {
    let mut handles = handles.borrow_mut();
    while handles.len() <= index {
        handles.push(SelectionHandle::default());
    }
    handles[index].clone()
}

/// 清理同一浮窗内一组可选区域的所有划词状态。
fn clear_selection_handles(handles: &SelectionHandleList) {
    for handle in handles.borrow().iter() {
        handle.clear();
    }
}

/// 清理同组其它可选区域，保留当前正在产生选择的区域。
fn clear_selection_handles_except(handles: &SelectionHandleList, selected_index: usize) {
    for (index, handle) in handles.borrow().iter().enumerate() {
        if index != selected_index {
            handle.clear();
        }
    }
}

/// 开始在一个可选区域内划词时，清掉其它同级区域的旧选择状态。
fn clear_selection_handles_for_active_area(
    query_selection_handles: &SelectionHandleList,
    output_selection_handles: &SelectionHandleList,
    action_selection_handles: &SelectionHandleList,
    active_group: SelectionHandleGroup,
    active_index: usize,
) {
    match active_group {
        SelectionHandleGroup::Query => {
            clear_selection_handles_except(query_selection_handles, active_index);
            clear_selection_handles(output_selection_handles);
            clear_selection_handles(action_selection_handles);
        }
        SelectionHandleGroup::Output => {
            clear_selection_handles(query_selection_handles);
            clear_selection_handles_except(output_selection_handles, active_index);
            clear_selection_handles(action_selection_handles);
        }
        SelectionHandleGroup::Action => {
            clear_selection_handles(query_selection_handles);
            clear_selection_handles(output_selection_handles);
            clear_selection_handles_except(action_selection_handles, active_index);
        }
    }
}

#[derive(Default)]
struct StateHandles {
    invalid_api_key_button_handle: MouseStateHandle,
    debug_copy_button_handle: MouseStateHandle,
    submit_issue_button_handle: MouseStateHandle,
    query_selection_handles: SelectionHandleList,
    output_selection_handles: SelectionHandleList,
    action_selection_handles: SelectionHandleList,
    speedbump_checkbox_handle: MouseStateHandle,
    ai_settings_link: HighlightedHyperlink,
    conversation_scroll_state: ClippedScrollStateHandle,
    input_hover_state: MouseStateHandle,
    dismiss_input_mouse_state: MouseStateHandle,
}

pub struct CLISubagentView {
    block_id: BlockId,
    model: Rc<dyn AIBlockModel<View = CLISubagentView>>,
    // CLI agent follow-up 会为同一个 subtask 追加多个 exchange；这里保留历史用于渲染旧轮次。
    history_models: Vec<Rc<dyn AIBlockModel<View = CLISubagentView>>>,
    // 与 history_models 对齐，用于防止重复 AppendedExchange 事件把同一轮渲染两次。
    history_exchange_ids: Vec<AIAgentExchangeId>,
    subagent_controller: ModelHandle<CLISubagentController>,
    action_model: ModelHandle<BlocklistAIActionModel>,
    terminal_model: Arc<FairMutex<TerminalModel>>,
    conversation_id: AIConversationId,
    terminal_view_id: EntityId,

    state_handles: StateHandles,
    code_editor_views: Vec<EmbeddedCodeEditorView>,
    code_editor_buttons: Vec<CodeSnippetButtonHandles>,
    table_section_handles: Vec<TableSectionHandles>,

    secret_redaction_state: SecretRedactionState,
    link_detection_state: DetectedLinksState,
    selected_text: Arc<RwLock<Option<String>>>,

    allow_button: CompactibleSplitActionButton,
    reject_button: CompactibleActionButton,
    take_over_button: CompactibleActionButton,
    transfer_control_button: CompactibleActionButton,
    allow_menu: ViewHandle<Menu<CLISubagentAction>>,
    is_allow_menu_open: bool,
    always_allow_write_to_pty_checked: bool,
    always_allow_read_files_checked: bool,
    // 整体对话滚动是否仍跟随最新输出；用户手动滚轮查看历史后会关闭。
    is_conversation_scroll_pinned_to_bottom: bool,
    // Hide responses 会让滚动内容变短并被 scrollable 裁剪到顶部；这里记录隐藏前的位置用于显示时恢复。
    hidden_response_scroll_offset: Option<f32>,

    is_input_dismissed: bool,
    input_dismiss_timer_handle: Option<SpawnedFutureHandle>,
    // 用户拖拽后的浮窗宽度，交给 Resizable 在多次 render 间保持。
    resizable_width: ResizableStateHandle,
    // 用户拖拽后的浮窗高度，同时作为内部滚动区域的 max height。
    resizable_height: ResizableStateHandle,

    current_working_directory: Option<String>,
    shell_launch_data: Option<ShellLaunchData>,
}

impl CLISubagentView {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        block_id: BlockId,
        action_model: ModelHandle<BlocklistAIActionModel>,
        subagent_controller: ModelHandle<CLISubagentController>,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        conversation_id: AIConversationId,
        task_id: TaskId,
        current_working_directory: Option<String>,
        shell_launch_data: Option<ShellLaunchData>,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        let allow_button = CompactibleSplitActionButton::new(
            "Allow".to_string(),
            Some(KeystrokeSource::Fixed(ACCEPT_KEYSTROKE.clone())),
            ButtonSize::Small,
            CLISubagentAction::ExecuteBlockedAction,
            CLISubagentAction::ToggleAllowMenu,
            Icon::Check,
            true,
            Some(ALLOW_ACTION_POSITION_ID.to_string()),
            ctx,
        );

        let reject_button = CompactibleActionButton::new(
            "Refine".to_string(),
            Some(KeystrokeSource::Fixed(REJECT_KEYSTROKE.clone())),
            ButtonSize::Small,
            CLISubagentAction::RejectBlockedAction {
                should_user_take_over: false,
            },
            Icon::X,
            Arc::new(NakedTheme),
            ctx,
        );

        let take_over_button = CompactibleActionButton::new(
            "Take over".to_string(),
            Some(KeystrokeSource::Binding(
                SET_INPUT_MODE_TERMINAL_ACTION_NAME,
            )),
            ButtonSize::Small,
            CLISubagentAction::RejectBlockedAction {
                should_user_take_over: true,
            },
            Icon::Hand,
            Arc::new(NakedTheme),
            ctx,
        );
        let transfer_control_button = CompactibleActionButton::new(
            "Take control".to_string(),
            Some(KeystrokeSource::Binding(
                SET_INPUT_MODE_TERMINAL_ACTION_NAME,
            )),
            ButtonSize::Small,
            CLISubagentAction::ExecuteBlockedAction,
            Icon::Hand,
            Arc::new(PrimaryTheme),
            ctx,
        );

        let allow_menu = ctx.add_typed_action_view(|ctx| {
            let theme = Appearance::as_ref(ctx).theme();
            Menu::new()
                .with_width(MENU_WIDTH)
                .with_menu_variant(MenuVariant::Fixed)
                .with_border(Border::all(1.).with_border_fill(theme.outline()))
                .prevent_interaction_with_other_elements()
        });
        allow_menu.update(ctx, |menu, ctx| {
            menu.set_items(
                vec![
                    MenuItemFields::new(crate::t!("ai-block-accept"))
                        .with_key_shortcut_label(Some(ACCEPT_KEYSTROKE.displayed()))
                        .with_on_select_action(CLISubagentAction::ExecuteBlockedAction)
                        .into_item(),
                    MenuItemFields::new(crate::t!("ai-block-auto-approve"))
                        .with_key_shortcut_label(Some(AUTO_APPROVE_KEYSTROKE.displayed()))
                        .with_on_select_action(CLISubagentAction::ExecuteAndAutoApprove)
                        .into_item(),
                ],
                ctx,
            );
        });
        ctx.subscribe_to_view(&allow_menu, |me, _menu, event, ctx| match event {
            MenuEvent::Close { .. } => {
                me.is_allow_menu_open = false;
                ctx.notify();
            }
            MenuEvent::ItemSelected | MenuEvent::ItemHovered => {}
        });

        // We want to default the checkbox to true when rendering the speedbump for the first time.
        // Otherwise, update it when the permission changes.
        let always_allow_write_to_pty_checked = if should_show_write_to_pty_speedbump(ctx) {
            true
        } else {
            BlocklistAIPermissions::as_ref(ctx)
                .can_write_to_pty(&conversation_id, Some(ctx.view_id()), ctx)
                .is_always_allow()
        };

        let always_allow_read_files_checked = if should_show_read_files_speedbump(ctx) {
            true
        } else {
            BlocklistAIPermissions::as_ref(ctx)
                .can_read_files(Some(&conversation_id), Vec::new(), Some(ctx.view_id()), ctx)
                .is_allowed()
        };

        let history_model = BlocklistAIHistoryModel::handle(ctx);
        let mut task_id_clone = task_id.clone();
        ctx.subscribe_to_model(
            &history_model,
            move |me, _history_model, event, ctx| match event {
                BlocklistAIHistoryEvent::UpgradedTask {
                    optimistic_id: old_id,
                    confirmed_task_id: new_id,
                    ..
                } if *old_id == task_id_clone => {
                    task_id_clone = new_id.clone();
                }
                BlocklistAIHistoryEvent::AppendedExchange {
                    exchange_id,
                    task_id,
                    conversation_id,
                    ..
                } => {
                    if task_id == &task_id_clone {
                        let appended_exchange_id = *exchange_id;
                        if let Ok(model) = AIBlockModelImpl::<CLISubagentView>::new(
                            appended_exchange_id,
                            *conversation_id,
                            false,
                            false,
                            ctx,
                        ) {
                            model.on_updated_output(
                                Box::new(move |me, ctx| {
                                    me.handle_updated_exchange_output(appended_exchange_id, ctx);
                                }),
                                ctx,
                            );
                            let model: Rc<dyn AIBlockModel<View = CLISubagentView>> =
                                Rc::new(model);
                            me.append_history_model(appended_exchange_id, model.clone());
                            me.model = model;
                            cli_subagent_mark_conversation_scroll_should_follow_latest(
                                &mut me.is_conversation_scroll_pinned_to_bottom,
                            );
                            me.code_editor_views = Default::default();
                            me.code_editor_buttons = Default::default();
                            me.table_section_handles = Default::default();
                            me.secret_redaction_state.reset();
                            me.set_state_from_updated_inputs(ctx);
                            me.refresh_output_secret_redaction_state(ctx);
                        }
                        ctx.notify();
                    }
                }
                _ => {
                    ctx.notify();
                }
            },
        );

        ctx.subscribe_to_model(
            &AIExecutionProfilesModel::handle(ctx),
            move |me, _, event, ctx| {
                let should_update_permissions = match event {
                    AIExecutionProfilesModelEvent::UpdatedActiveProfile { terminal_view_id } => {
                        *terminal_view_id == me.terminal_view_id
                    }
                    AIExecutionProfilesModelEvent::ProfileUpdated(profile_id) => {
                        let active_profile = AIExecutionProfilesModel::as_ref(ctx)
                            .active_profile(Some(me.terminal_view_id), ctx);
                        *profile_id == *active_profile.id()
                    }
                    _ => false,
                };
                if should_update_permissions {
                    let ai_permission = BlocklistAIPermissions::as_ref(ctx);
                    if should_show_write_to_pty_speedbump(ctx) {
                        me.always_allow_write_to_pty_checked = ai_permission
                            .can_write_to_pty(&me.conversation_id, Some(me.terminal_view_id), ctx)
                            .is_always_allow();
                    }
                    if should_show_read_files_speedbump(ctx) {
                        me.always_allow_read_files_checked = ai_permission
                            .get_read_files_setting(ctx, Some(me.terminal_view_id))
                            .is_always_allow();
                    }
                    ctx.notify();
                }
            },
        );
        let (exchange_id, initial_history_exchange_ids) = history_model
            .as_ref(ctx)
            .conversation(&conversation_id)
            .and_then(|c| {
                c.get_task(&task_id)
                    .and_then(|t| {
                        t.last_exchange().map(|last_exchange| {
                            (
                                last_exchange.id,
                                t.exchanges()
                                    .map(|exchange| exchange.id)
                                    .collect::<Vec<_>>(),
                            )
                        })
                    })
                    .or_else(|| {
                        // Zap BYOP fallback:agent 自起 LRC 时
                        // `cli_controller::FinishedAction` 通过
                        // `create_silent_cli_subagent_task_for_conversation` 真实创建
                        // subtask 但暂未给它 append exchange(没新 query 触发
                        // `update_for_new_request_input`),用 root task 的 last
                        // exchange 占位。后续用户 follow-up query 路由到此 task →
                        // `AppendedExchange` → 上面的订阅(line 365-394)会自动
                        // 切到真实 exchange。占位不加入 history,避免显示不属于此
                        // subtask 的 root exchange。
                        let fallback = c.root_task_exchanges().last().map(|e| (e.id, Vec::new()));
                        if fallback.is_some() {
                            log::warn!(
                                "[byop] CLISubagentView::new task={task_id:?} 暂无 \
                                 exchange,fallback 到 root_task last_exchange;\
                                 等待 AppendedExchange 触发切换。"
                            );
                        }
                        fallback
                    })
            })
            .expect("Exchange exists.");
        let model = AIBlockModelImpl::<CLISubagentView>::new(
            exchange_id,
            conversation_id,
            false,
            false,
            ctx,
        )
        .expect("Exchange exists.");
        model.on_updated_output(
            Box::new(move |me, ctx| {
                me.handle_updated_exchange_output(exchange_id, ctx);
            }),
            ctx,
        );
        let model: Rc<dyn AIBlockModel<View = CLISubagentView>> = Rc::new(model);
        let mut history_models: Vec<Rc<dyn AIBlockModel<View = CLISubagentView>>> = Vec::new();
        let mut history_exchange_ids = Vec::new();
        for history_exchange_id in initial_history_exchange_ids {
            if history_exchange_id == exchange_id {
                if cli_subagent_append_history_exchange_id(
                    &mut history_exchange_ids,
                    history_exchange_id,
                ) {
                    history_models.push(model.clone());
                }
                continue;
            }

            if let Ok(history_model) = AIBlockModelImpl::<CLISubagentView>::new(
                history_exchange_id,
                conversation_id,
                false,
                false,
                ctx,
            ) {
                if cli_subagent_append_history_exchange_id(
                    &mut history_exchange_ids,
                    history_exchange_id,
                ) {
                    history_models.push(Rc::new(history_model));
                }
            }
        }

        ctx.subscribe_to_model(&subagent_controller, |me, _, event, ctx| match event {
            CLISubagentEvent::UpdatedControl { block_id, .. } => {
                if *block_id == me.block_id {
                    ctx.notify();
                }
            }
            CLISubagentEvent::ToggledHideResponses => {
                let should_hide_responses = me
                    .terminal_model
                    .lock()
                    .block_list()
                    .block_with_id(&me.block_id)
                    .is_some_and(|block| block.should_hide_responses());
                let restored_scroll_offset = cli_subagent_response_visibility_scroll_offset(
                    me.state_handles
                        .conversation_scroll_state
                        .scroll_start()
                        .as_f32(),
                    should_hide_responses,
                    &mut me.hidden_response_scroll_offset,
                );
                if let Some(scroll_offset) = restored_scroll_offset {
                    me.state_handles
                        .conversation_scroll_state
                        .scroll_to(scroll_offset.into_pixels());
                }
                me.reset_input_dismiss_timer(ctx);
                ctx.notify();
            }
            _ => {}
        });

        let mut view = Self {
            block_id,
            model,
            history_models,
            history_exchange_ids,
            action_model,
            terminal_model,
            subagent_controller,
            conversation_id,
            terminal_view_id: ctx.view_id(),
            link_detection_state: Default::default(),
            code_editor_views: Default::default(),
            code_editor_buttons: Default::default(),
            table_section_handles: Default::default(),
            secret_redaction_state: Default::default(),
            state_handles: Default::default(),
            allow_button,
            reject_button,
            take_over_button,
            transfer_control_button,
            allow_menu,
            is_allow_menu_open: false,
            always_allow_write_to_pty_checked,
            always_allow_read_files_checked,
            is_conversation_scroll_pinned_to_bottom: true,
            hidden_response_scroll_offset: None,
            is_input_dismissed: false,
            input_dismiss_timer_handle: None,
            resizable_width: resizable_state_handle(MIN_RESIZABLE_WIDTH),
            resizable_height: resizable_state_handle(MAX_HEIGHT),
            current_working_directory,
            shell_launch_data,
            selected_text: Arc::new(RwLock::new(None)),
        };
        view.set_state_from_updated_inputs(ctx);
        view.refresh_output_secret_redaction_state(ctx);
        view
    }

    fn append_history_model(
        &mut self,
        exchange_id: AIAgentExchangeId,
        model: Rc<dyn AIBlockModel<View = CLISubagentView>>,
    ) -> bool {
        // 同一个 AppendedExchange 可能被多条订阅路径观察到，历史列表只保留一次。
        if cli_subagent_append_history_exchange_id(&mut self.history_exchange_ids, exchange_id) {
            self.history_models.push(model);
            true
        } else {
            false
        }
    }

    fn models_to_render_for_output_redaction(
        &self,
    ) -> Vec<Rc<dyn AIBlockModel<View = CLISubagentView>>> {
        // history_models 非空时已经按 render 顺序包含最新 exchange；否则只渲染当前 model。
        if self.history_models.is_empty() {
            vec![self.model.clone()]
        } else {
            self.history_models.clone()
        }
    }

    fn refresh_output_secret_redaction_state(&mut self, ctx: &mut ViewContext<Self>) {
        let secret_redaction_mode = get_secret_obfuscation_mode(ctx);
        let models_to_scan = self.models_to_render_for_output_redaction();

        self.secret_redaction_state.clear_output_locations();
        self.secret_redaction_state.reset();

        if !secret_redaction_mode.should_redact_secret() {
            return;
        }

        let should_obfuscate = secret_redaction_mode.is_visually_obfuscated();
        let mut text_section_index = 0;

        for model in models_to_scan {
            if let Some(output) = model.status(ctx).output_to_render() {
                let output = output.get();
                text_section_index += cli_subagent_run_redaction_on_rendered_output(
                    &mut self.secret_redaction_state,
                    &output,
                    text_section_index,
                    should_obfuscate,
                );
            }
        }
    }

    fn execute_pending_action(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(blocked_action) = self.model.blocked_action(&self.action_model, ctx) else {
            return;
        };

        self.action_model.update(ctx, |action_model, ctx| {
            action_model.execute_next_action_for_user(self.conversation_id, ctx);
        });

        self.maybe_update_speedbump(&blocked_action.action, ctx);
    }

    fn has_pending_transfer_control_action(&self, app: &AppContext) -> bool {
        self.model
            .blocked_action(&self.action_model, app)
            .is_some_and(|action| {
                matches!(
                    action.action,
                    AIAgentActionType::TransferShellCommandControlToUser { .. }
                )
            })
    }

    fn handle_execute_blocked_action(
        &mut self,
        is_autoexecuted: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        self.execute_pending_action(ctx);
        if is_autoexecuted {
            self.enable_autoexecute_override(ctx);
        }

        send_telemetry_from_ctx!(
            TelemetryEvent::CLISubagentActionExecuted {
                conversation_id: self.conversation_id,
                block_id: self.block_id.clone(),
                is_autoexecuted,
            },
            ctx
        );
    }

    fn handle_reject_blocked_action(
        &mut self,
        should_user_take_over: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        self.reject_blocked_action(should_user_take_over, ctx);

        send_telemetry_from_ctx!(
            TelemetryEvent::CLISubagentActionRejected {
                conversation_id: self.conversation_id,
                block_id: self.block_id.clone(),
                user_took_over: should_user_take_over,
            },
            ctx
        );
    }

    fn take_control_of_running_command(&mut self, ctx: &mut ViewContext<Self>) {
        if self.has_pending_transfer_control_action(ctx) {
            self.handle_execute_blocked_action(false, ctx);
        } else {
            self.handle_reject_blocked_action(true, ctx);
        }
    }
    fn reject_blocked_action(&mut self, should_user_take_over: bool, ctx: &mut ViewContext<Self>) {
        let Some(blocked_action) = self.model.blocked_action(&self.action_model, ctx) else {
            return;
        };

        self.action_model.update(ctx, |action_model, ctx| {
            action_model.cancel_action_with_id(
                self.conversation_id,
                &blocked_action.id,
                CancellationReason::ManuallyCancelled,
                ctx,
            );
        });

        if should_user_take_over {
            self.subagent_controller.update(ctx, |controller, ctx| {
                controller.switch_control_to_user(UserTakeOverReason::Manual, ctx);
            });
            ctx.notify();
        }

        self.maybe_update_speedbump(&blocked_action.action, ctx);
    }

    fn enable_autoexecute_override(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(conversation) =
            BlocklistAIHistoryModel::as_ref(ctx).conversation(&self.conversation_id)
        else {
            return;
        };
        if !conversation.autoexecute_any_action() {
            BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                history.toggle_autoexecute_override(
                    &self.conversation_id,
                    self.terminal_view_id,
                    ctx,
                );
            });
        }
    }
    fn toggle_allow_menu(&mut self, ctx: &mut ViewContext<Self>) {
        self.is_allow_menu_open = !self.is_allow_menu_open;
        if self.is_allow_menu_open {
            ctx.focus(&self.allow_menu);
        }
        ctx.notify();
    }

    // If the speedbump is shown, we update the settings such that the speedbump won't be shown again, and the permission reflect the checked value.
    // This is called on any user action instead of on render time to ensure the state is updated correctly.
    fn maybe_update_speedbump(&mut self, action: &AIAgentActionType, ctx: &mut ViewContext<Self>) {
        match action {
            AIAgentActionType::WriteToLongRunningShellCommand { .. }
                if should_show_write_to_pty_speedbump(ctx) =>
            {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let _ = settings
                        .should_show_agent_mode_write_to_pty_speedbump
                        .set_value(false, ctx);
                });

                BlocklistAIPermissions::handle(ctx).update(ctx, |permissions, ctx| {
                    if let Err(e) = permissions.set_always_allow_write_to_pty(
                        self.always_allow_write_to_pty_checked,
                        self.terminal_view_id,
                        ctx,
                    ) {
                        report_error!(e);
                    }
                });
                ctx.notify();
            }
            AIAgentActionType::ReadFiles(_)
            | AIAgentActionType::Grep { .. }
            | AIAgentActionType::FileGlobV2 { .. } => {
                if should_show_read_files_speedbump(ctx) {
                    AISettings::handle(ctx).update(ctx, |settings, ctx| {
                        let _ = settings
                            .should_show_agent_mode_autoread_files_speedbump
                            .set_value(false, ctx);
                    });

                    BlocklistAIPermissions::handle(ctx).update(ctx, |permissions, ctx| {
                        if let Err(e) = permissions.set_always_allow_read_files(
                            self.always_allow_read_files_checked,
                            self.terminal_view_id,
                            ctx,
                        ) {
                            report_error!(e);
                        }
                    });
                    ctx.notify();
                }
            }
            _ => {}
        }
    }

    fn handle_updated_exchange_output(
        &mut self,
        exchange_id: AIAgentExchangeId,
        ctx: &mut ViewContext<Self>,
    ) {
        if self.model.exchange_id(ctx) != Some(exchange_id) {
            self.refresh_output_secret_redaction_state(ctx);
            ctx.notify();
            return;
        }

        match self.model.status(ctx) {
            AIBlockOutputStatus::Pending => {
                self.refresh_output_secret_redaction_state(ctx);
            }
            AIBlockOutputStatus::PartiallyReceived { output } => {
                let output = output.get();
                self.handle_updated_output(&output, ctx);
                self.refresh_output_secret_redaction_state(ctx);
            }
            AIBlockOutputStatus::Complete { output } => {
                let output = output.get();
                self.handle_updated_output(&output, ctx);
                self.refresh_output_secret_redaction_state(ctx);
            }
            AIBlockOutputStatus::Cancelled { partial_output, .. } => {
                if let Some(output) = partial_output.as_ref() {
                    let output = output.get();
                    self.handle_updated_output(&output, ctx);
                }
                self.refresh_output_secret_redaction_state(ctx);
            }
            AIBlockOutputStatus::Failed { partial_output, .. } => {
                if let Some(output) = partial_output.as_ref() {
                    let output = output.get();
                    self.handle_updated_output(&output, ctx);
                }
                self.refresh_output_secret_redaction_state(ctx);
            }
        }
        ctx.notify();
    }

    fn handle_updated_output(&mut self, output: &AIAgentOutput, ctx: &mut ViewContext<Self>) {
        // Build the views and stream new content for suggested code snippets.
        output
            .all_text()
            .flat_map(|text| text.sections.iter())
            .filter_map(|section| match section {
                AIAgentTextSection::Code {
                    code,
                    language,
                    source,
                } => Some((code, language, source)),
                _ => None,
            })
            .enumerate()
            .for_each(|(index, (code, language, source))| {
                self.handle_code_section_stream_update(index, code, language, source, ctx);
            });
    }

    fn handle_code_section_stream_update(
        &mut self,
        index: usize,
        code: &str,
        language: &Option<ProgrammingLanguage>,
        source: &Option<CodeSource>,
        ctx: &mut ViewContext<Self>,
    ) {
        match self.code_editor_views.get_mut(index) {
            Some(embedded_view) => {
                embedded_view.view.update(ctx, |view, ctx| {
                    // The language and starting line number may not be specified in the output for the first iteration.
                    // Only set the language/starting line number the first time that they are specified or if they change.
                    if embedded_view.language != *language {
                        embedded_view.language = language.clone();
                        if let Some(extension) = language
                            .as_ref()
                            .and_then(|language| language.to_extension())
                        {
                            // Since this is a code snippet, construct a fake path name for looking up the language.
                            let fake_path_string = format!("snippet.{extension}");
                            let fake_path = std::path::Path::new(&fake_path_string);
                            view.set_language_with_path(fake_path, ctx);
                        }
                    }
                    let starting_line_number = source.as_ref().and_then(|s| {
                        if let CodeSource::Link { range_start, .. } = s {
                            range_start.as_ref().map(|ls| ls.line_num)
                        } else {
                            None
                        }
                    });
                    if view.starting_line_number() != starting_line_number {
                        view.set_starting_line_number(starting_line_number);
                    }

                    // Update the buffer with just the new or deleted range.
                    // Assumption: Only the end of the string is updated.
                    // Assumption: The only time text is deleted is at the end of parsing, where it has partially
                    // received the ``` end marker.
                    // Ex: Iteration 57: "a += 12\n``"
                    // Ex: Iteration 58: "a += 12"
                    match code.len().cmp(&embedded_view.length) {
                        Ordering::Greater => {
                            view.append_at_end(&code[embedded_view.length..], ctx);
                            ctx.notify();
                        }
                        Ordering::Less => {
                            view.truncate(code.len(), ctx);
                            ctx.notify();
                        }
                        Ordering::Equal => {}
                    }
                    embedded_view.length = code.len();
                });
            }
            None => {
                let view = ctx.add_typed_action_view(|ctx| {
                    CodeEditorView::new(
                        None,
                        None,
                        CodeEditorRenderOptions::new(VerticalExpansionBehavior::InfiniteHeight),
                        ctx,
                    )
                    .with_can_show_diff_ui(false)
                });
                view.update(ctx, |view, ctx| {
                    view.set_starting_line_number({
                        source.as_ref().and_then(|s| match s {
                            CodeSource::Link { range_start, .. } => {
                                range_start.as_ref().map(|ls| ls.line_num)
                            }
                            _ => None,
                        })
                    });
                    view.set_show_current_line_highlights(false, ctx);
                    view.set_interaction_state(InteractionState::Selectable, ctx);
                    let state = InitialBufferState::plain_text(code);
                    view.reset(state, ctx);
                    ctx.notify();
                });
                ctx.subscribe_to_view(&view, |me, view, event, ctx| match event {
                    CodeEditorEvent::SelectionChanged => {
                        if let Some(selected_text) = view.as_ref(ctx).selected_text(ctx) {
                            me.maybe_copy_on_select(selected_text, ctx);
                            me.clear_other_selections(Some(view.id()), ctx);
                            ctx.emit(CLISubagentViewEvent::TextSelected);
                        }
                    }
                    CodeEditorEvent::CopiedEmptyText => {
                        ctx.emit(CLISubagentViewEvent::CopiedEmptyText);
                    }
                    #[cfg(windows)]
                    CodeEditorEvent::WindowsCtrlC { .. } => {
                        ctx.emit(CLISubagentViewEvent::WindowsCtrlC);
                    }
                    _ => {}
                });
                self.code_editor_views.push(EmbeddedCodeEditorView {
                    view,
                    language: Default::default(),
                    length: code.len(),
                });
                self.code_editor_buttons.push(Default::default());
            }
        }
    }

    fn reset_input_dismiss_timer(&mut self, ctx: &mut ViewContext<Self>) {
        self.is_input_dismissed = false;
        if let Some(handle) = self.input_dismiss_timer_handle.take() {
            handle.abort();
        }

        let has_user_input = if self.history_models.is_empty() {
            self.model
                .inputs_to_render(ctx)
                .iter()
                .any(|input| input.is_user_query())
        } else {
            self.history_models.iter().any(|model| {
                model
                    .inputs_to_render(ctx)
                    .iter()
                    .any(|input| input.is_user_query())
            })
        };
        let should_hide_responses = self
            .terminal_model
            .lock()
            .block_list()
            .active_block()
            .should_hide_responses();

        if has_user_input && should_hide_responses {
            let handle = ctx.spawn_abortable(
                Timer::after(Duration::from_secs(4)),
                |me, _, ctx| {
                    me.is_input_dismissed = true;
                    me.input_dismiss_timer_handle = None;
                    ctx.notify();
                },
                |_, _| {},
            );
            self.input_dismiss_timer_handle = Some(handle);
        }
    }

    fn set_state_from_updated_inputs(&mut self, ctx: &mut ViewContext<Self>) {
        // Clear existing link detection state
        self.link_detection_state.detected_links_by_location.clear();

        self.reset_input_dismiss_timer(ctx);

        let user_queries = if self.history_models.is_empty() {
            self.model
                .inputs_to_render(ctx)
                .iter()
                .filter_map(|input| match input {
                    AIAgentInput::UserQuery { query, .. } => Some(query.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        } else {
            self.history_models
                .iter()
                .flat_map(|model| model.inputs_to_render(ctx))
                .filter_map(|input| match input {
                    AIAgentInput::UserQuery { query, .. } => Some(query.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };

        // 按历史顺序检测所有用户 query，确保旧轮次重新渲染后链接和脱敏索引仍然连续。
        for (input_index, query) in user_queries.iter().enumerate() {
            detect_links(
                &mut self.link_detection_state,
                query,
                TextLocation::Query { input_index },
                self.current_working_directory.as_ref(),
                self.shell_launch_data.as_ref(),
            );

            let secret_redaction_mode = get_secret_obfuscation_mode(ctx);
            if secret_redaction_mode.should_redact_secret() {
                let should_obfuscate = secret_redaction_mode.is_visually_obfuscated();
                self.secret_redaction_state.run_redaction_for_location(
                    query,
                    TextLocation::Query { input_index },
                    should_obfuscate,
                );
            }
        }
    }

    /// Clears text selections at the `CLISubagentView` level (e.g. user query text).
    /// This does _not_ clear the selection of the child views (code blocks).
    fn clear_view_level_selection(&mut self) {
        clear_selection_handles(&self.state_handles.query_selection_handles);
        clear_selection_handles(&self.state_handles.output_selection_handles);
        clear_selection_handles(&self.state_handles.action_selection_handles);
        *self.selected_text.write() = None;
    }

    /// Clears all text selections in all components within this `CLISubagentView`'s view sub-hierarchy
    /// _other_ than the one that triggered a selection change.
    ///
    /// Call this after text is selected in one part of the view (e.g. a code snippet), to ensure
    /// that there's only one active selection at a time.
    fn clear_other_selections(
        &mut self,
        source_view_id: Option<EntityId>,
        ctx: &mut ViewContext<Self>,
    ) {
        for editor_view in self.code_editor_views.iter() {
            // Don't clear selections for the view that triggered this change.
            if source_view_id.is_some_and(|entity_id| editor_view.view.id() == entity_id) {
                continue;
            }
            editor_view
                .view
                .update(ctx, |view, ctx| view.clear_selection(ctx));
        }

        // If the event was dispatched by a nested view (i.e. code block),
        // clear the text selection at the `CLISubagentView` level (outside the code block).
        // We want to have only 1 selection active at any one point in time.
        if source_view_id.is_some() {
            self.clear_view_level_selection();
        }
    }

    /// Clears all text selections in all components within this `CLISubagentView`'s view sub-hierarchy.
    /// This includes the `CLISubagentView` level and all child views (code blocks).
    pub fn clear_all_selections(&mut self, ctx: &mut ViewContext<Self>) {
        self.clear_other_selections(None, ctx);
        self.clear_view_level_selection();
    }

    pub fn selected_text(&self, ctx: &AppContext) -> Option<String> {
        self.code_editor_views
            .iter()
            .find_map(|editor_view| editor_view.view.as_ref(ctx).selected_text(ctx))
            .or_else(|| self.selected_text.read().clone())
            .filter(|selection| !selection.is_empty())
    }

    fn maybe_copy_on_select(&self, selection: String, ctx: &mut ViewContext<Self>) {
        SelectionSettings::handle(ctx).update(ctx, |selection_settings, ctx| {
            selection_settings.maybe_copy_on_select(ClipboardContent::plain_text(selection), ctx);
        });
    }
}

#[derive(Debug, Clone)]
pub enum CLISubagentViewEvent {
    TextSelected,
    CopiedEmptyText,
    #[cfg(windows)]
    WindowsCtrlC,
}

impl Entity for CLISubagentView {
    type Event = CLISubagentViewEvent;
}

impl View for CLISubagentView {
    fn ui_name() -> &'static str {
        "CLISubagentView"
    }

    fn render(&self, app: &warpui::AppContext) -> Box<dyn warpui::Element> {
        let terminal_model = self.terminal_model.lock();
        let Some(block) = terminal_model.block_list().block_with_id(&self.block_id) else {
            return Empty::new().finish();
        };

        if !block.is_agent_monitoring() || block.is_eligible_for_agent_handoff() {
            return Empty::new().finish();
        }

        let appearance = Appearance::as_ref(app);
        let theme = appearance.theme();
        let semantic_selection = SemanticSelection::handle(app).as_ref(app);
        let resizable_height = self
            .resizable_height
            .lock()
            .map(|state| state.size())
            .unwrap_or(MAX_HEIGHT);

        let mut conversation_items = Flex::column()
            .with_main_axis_size(MainAxisSize::Min)
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch);

        let models_to_render = if self.history_models.is_empty() {
            vec![&self.model]
        } else {
            self.history_models.iter().collect::<Vec<_>>()
        };
        let mut rendered_query_index = 0;
        let mut rendered_output_index = 0;
        let mut text_section_index = 0;
        let model_count = models_to_render.len();

        for (model_index, model) in models_to_render.into_iter().enumerate() {
            let is_latest_model = model_index + 1 == model_count;
            let should_hide_responses = block.should_hide_responses();

            // Render user queries/follow-ups with avatar and interactive text
            let inputs = model.inputs_to_render(app);
            for input in inputs.iter() {
                if let AIAgentInput::UserQuery { query, .. } = input {
                    let input_index = rendered_query_index;
                    rendered_query_index += 1;
                    let query_selection_handle = selection_handle_for_index(
                        &self.state_handles.query_selection_handles,
                        input_index,
                    );
                    let text = render_query_text(
                        UserQueryProps {
                            text: query.to_owned(),
                            query_prefix_highlight_len: None,
                            detected_links_state: &self.link_detection_state,
                            secret_redaction_state: &self.secret_redaction_state,
                            input_index,
                            is_selecting: query_selection_handle.is_selecting(),
                            is_ai_input_enabled: false,
                            find_context: None,
                            font_properties: &Properties {
                                style: Style::Normal,
                                weight: Weight::Normal,
                            },
                        },
                        app,
                    );

                    let selected_text = self.selected_text.clone();
                    let query_selection_handles =
                        self.state_handles.query_selection_handles.clone();
                    let output_selection_handles =
                        self.state_handles.output_selection_handles.clone();
                    let action_selection_handles =
                        self.state_handles.action_selection_handles.clone();
                    // 克隆一份本区域句柄进闭包，判断"本区域是否真的在参与选择"。
                    // Flex 会把同一鼠标事件广播给所有兄弟 SelectableArea，未命中的气泡也会触发本回调，
                    // 此时不能用未命中的回调去清掉真正命中区域的划词状态。
                    let query_selection_handle_clone = query_selection_handle.clone();
                    let mut selectable_text = SelectableArea::new(
                        query_selection_handle,
                        move |selection_args, ctx, _| {
                            let selection = selection_args.selection;
                            // 只有本区域确实参与选择时（正在 selecting 或已产生非空选中文本），
                            // 才清掉其它同级区域的旧选择；未命中广播则保持原状。
                            let is_this_area_active = query_selection_handle_clone.is_selecting()
                                || selection.as_ref().is_some_and(|s| !s.is_empty());
                            if is_this_area_active {
                                clear_selection_handles_for_active_area(
                                    &query_selection_handles,
                                    &output_selection_handles,
                                    &action_selection_handles,
                                    SelectionHandleGroup::Query,
                                    input_index,
                                );
                            }
                            if let Some(selection) =
                                selection.filter(|selection| !selection.is_empty())
                            {
                                ctx.dispatch_typed_action(CLISubagentAction::CopyOnSelect(
                                    selection.clone(),
                                ));
                                *selected_text.write() = Some(selection);
                                ctx.dispatch_typed_action(CLISubagentAction::SelectText);
                            } else if is_this_area_active {
                                *selected_text.write() = None;
                            }
                        },
                        text.finish(),
                    )
                    .with_word_boundaries_policy(semantic_selection.word_boundary_policy())
                    .with_smart_select_fn(semantic_selection.smart_select_fn());

                    if FeatureFlag::RectSelection.is_enabled() {
                        selectable_text = selectable_text.should_support_rect_select();
                    }

                    let query_container = render_framed_container(FramedContainerProps {
                        child: selectable_text.finish(),
                        background_color: internal_colors::accent_bg(theme).into(),
                        border: Some(Border::all(1.).with_border_fill(theme.accent())),
                    })
                    .with_margin_bottom(8.)
                    .finish();

                    let dismissable_stack = render_dismissable_container(
                        DismissableContainerProps {
                            child: query_container,
                            hover_state: self.state_handles.input_hover_state.clone(),
                            dismiss_mouse_state: self
                                .state_handles
                                .dismiss_input_mouse_state
                                .clone(),
                            position_id: USER_QUERY_POSITION_ID.to_string(),
                        },
                        app,
                    );

                    if cli_subagent_should_render_user_input(
                        should_hide_responses,
                        self.is_input_dismissed,
                    ) {
                        conversation_items.add_child(dismissable_stack);
                    }
                }
            }

            // Render agent outputs and actions
            let mut output_items = Flex::column()
                .with_main_axis_size(MainAxisSize::Min)
                .with_cross_axis_alignment(CrossAxisAlignment::Stretch);

            let status = model.status(app);
            let blocked_action = if is_latest_model {
                model.blocked_action(&self.action_model, app)
            } else {
                None
            };
            let output_index = rendered_output_index;
            let output_selection_handle = selection_handle_for_index(
                &self.state_handles.output_selection_handles,
                output_index,
            );

            if let Some(output) = status.output_to_render() {
                let output = output.get();

                let mut code_section_index = 0;
                let mut table_section_index = 0;
                let mut image_section_index = 0;

                fn copy_code_action(snippet: String) -> CLISubagentAction {
                    CLISubagentAction::CopyCode(snippet)
                }

                fn open_code_block_action(source: CodeSource) -> CLISubagentAction {
                    CLISubagentAction::OpenCodeBlock(source)
                }

                for output_message in output.messages.iter() {
                    match &output_message.message {
                        AIAgentOutputMessageType::Text(AIAgentText { sections })
                            if !are_all_text_sections_empty(sections) =>
                        {
                            let text_color = blended_colors::text_main(theme, theme.surface_1());
                            output_items.add_child(render_text_sections(
                                TextSectionsProps {
                                    model: model.as_ref(),
                                    starting_text_section_index: &mut text_section_index,
                                    starting_code_section_index: &mut code_section_index,
                                    starting_table_section_index: &mut table_section_index,
                                    starting_image_section_index: &mut image_section_index,
                                    sections,
                                    is_selecting_text: output_selection_handle.is_selecting(),
                                    selectable: true,
                                    text_color,
                                    is_ai_input_enabled: false,
                                    secret_redaction_state: &self.secret_redaction_state,
                                    find_context: None,
                                    shell_launch_data: None,
                                    current_working_directory: None,
                                    embedded_code_editor_views: if is_latest_model {
                                        &self.code_editor_views
                                    } else {
                                        &[]
                                    },
                                    code_snippet_button_handles: if is_latest_model {
                                        &self.code_editor_buttons
                                    } else {
                                        &[]
                                    },
                                    table_section_handles: if is_latest_model {
                                        &self.table_section_handles
                                    } else {
                                        &[]
                                    },
                                    // CLI subagent blocks don't render block-list images yet,
                                    // so there are no per-image tooltip handles to thread.
                                    image_section_tooltip_handles: &[],
                                    open_code_block_action_factory: Some(&open_code_block_action),
                                    copy_code_action_factory: Some(&copy_code_action),
                                    detected_links: Some(&self.link_detection_state),
                                    item_spacing: CONTENT_PADDING,
                                    #[cfg(feature = "local_fs")]
                                    resolved_code_block_paths: None,
                                    #[cfg(feature = "local_fs")]
                                    resolved_blocklist_image_sources: None,
                                },
                                app,
                            ));
                        }
                        AIAgentOutputMessageType::Action(action) => {
                            let is_cancelled = self
                                .action_model
                                .as_ref(app)
                                .get_action_status(&action.id)
                                .is_some_and(|status| status.is_cancelled());
                            if is_latest_model
                                && blocked_action.is_none()
                                && !is_cancelled
                                && !should_hide_responses
                            {
                                if let Some(rendered_action) =
                                    render_action(action.action.clone(), app)
                                {
                                    conversation_items.add_child(
                                        render_framed_container(FramedContainerProps {
                                            child: rendered_action,
                                            background_color: internal_colors::neutral_2(
                                                appearance.theme(),
                                            ),
                                            border: Some(Border::all(1.).with_border_fill(
                                                internal_colors::neutral_3(theme),
                                            )),
                                        })
                                        .with_margin_bottom(8.)
                                        .finish(),
                                    );
                                }
                            }
                        }
                        AIAgentOutputMessageType::WebSearch(WebSearchStatus::Searching {
                            query,
                        }) => {
                            if is_latest_model && !should_hide_responses {
                                conversation_items.add_child(
                                    render_framed_container(FramedContainerProps {
                                        child: render_web_search(query.clone(), app),
                                        background_color: internal_colors::neutral_2(
                                            appearance.theme(),
                                        ),
                                        border: Some(
                                            Border::all(1.).with_border_fill(
                                                internal_colors::neutral_3(theme),
                                            ),
                                        ),
                                    })
                                    .with_margin_bottom(8.)
                                    .finish(),
                                );
                            }
                        }
                        _ => (),
                    }
                }
            }

            let mut output_border =
                Border::all(1.).with_border_fill(internal_colors::neutral_3(theme));
            if let AIBlockOutputStatus::Failed { error, .. } = &status {
                output_border = Border::all(1.).with_border_color(theme.ui_error_color());
                output_items.add_child(render_failed_output(
                    FailedOutputProps {
                        error,
                        is_ai_input_enabled: false,
                        invalid_api_key_button_handle: &self
                            .state_handles
                            .invalid_api_key_button_handle,
                        aws_bedrock_credentials_error_view: None,
                        icon_right_margin: AVATAR_RIGHT_MARGIN,
                    },
                    app,
                ));

                if is_latest_model && !model.is_restored() && !error.is_invalid_api_key() {
                    output_items.add_child(
                    Container::new(render_informational_footer(
                        app,
                        "This response won't count towards your usage. \"Take over\" to continue."
                            .to_string(),
                    ))
                    .with_margin_top(8.)
                    .with_margin_left(icon_size(app) + AVATAR_RIGHT_MARGIN)
                    .finish(),
                );

                    output_items.add_child(
                        Container::new(render_debug_footer(
                            DebugFooterProps {
                                conversation: model.conversation(app),
                                model: model.as_ref(),
                                debug_copy_button_handle: self
                                    .state_handles
                                    .debug_copy_button_handle
                                    .clone(),
                                submit_issue_button_handle: self
                                    .state_handles
                                    .submit_issue_button_handle
                                    .clone(),
                                should_render_feedback_below: true,
                            },
                            |debug_id, ctx| {
                                ctx.dispatch_typed_action(CLISubagentAction::CopyDebugId(debug_id))
                            },
                            |ctx| ctx.dispatch_typed_action(CLISubagentAction::OpenFeedbackDocs),
                            app,
                        ))
                        .with_margin_top(8.)
                        .with_margin_left(icon_size(app) + AVATAR_RIGHT_MARGIN)
                        .finish(),
                    );
                }
            }

            if !output_items.is_empty() && !should_hide_responses {
                let selected_text = self.selected_text.clone();
                let query_selection_handles = self.state_handles.query_selection_handles.clone();
                let output_selection_handles = self.state_handles.output_selection_handles.clone();
                let action_selection_handles = self.state_handles.action_selection_handles.clone();
                // 克隆一份本区域的句柄进闭包，用于判断"本区域是否真的在参与选择"。
                // Flex 会把同一鼠标事件广播给所有兄弟 SelectableArea，未命中的气泡也会触发本回调，
                // 此时不能用未命中的回调去清掉真正命中区域的划词状态。
                let output_selection_handle_clone = output_selection_handle.clone();
                let mut output = SelectableArea::new(
                    output_selection_handle.clone(),
                    move |selection_args, ctx, _| {
                        let selection = selection_args.selection;
                        // 只有本区域确实参与选择时（正在 selecting 或已产生非空选中文本），
                        // 才清掉其它同级区域的旧选择；未命中广播则保持原状。
                        let is_this_area_active = output_selection_handle_clone.is_selecting()
                            || selection.as_ref().is_some_and(|s| !s.is_empty());
                        if is_this_area_active {
                            clear_selection_handles_for_active_area(
                                &query_selection_handles,
                                &output_selection_handles,
                                &action_selection_handles,
                                SelectionHandleGroup::Output,
                                output_index,
                            );
                        }
                        if let Some(selection) = selection.filter(|selection| !selection.is_empty())
                        {
                            ctx.dispatch_typed_action(CLISubagentAction::CopyOnSelect(
                                selection.clone(),
                            ));
                            *selected_text.write() = Some(selection);
                            ctx.dispatch_typed_action(CLISubagentAction::SelectText);
                        } else if is_this_area_active {
                            *selected_text.write() = None;
                        }
                    },
                    output_items.finish(),
                )
                .with_word_boundaries_policy(semantic_selection.word_boundary_policy())
                .with_smart_select_fn(semantic_selection.smart_select_fn());

                if FeatureFlag::RectSelection.is_enabled() {
                    output = output.should_support_rect_select();
                }

                conversation_items.add_child(
                    render_framed_container(FramedContainerProps {
                        child: output.finish(),
                        background_color: internal_colors::neutral_2(appearance.theme()),
                        border: Some(output_border),
                    })
                    .with_margin_bottom(8.)
                    .finish(),
                );
                rendered_output_index += 1;
            }

            if let Some(rendered_action) = blocked_action.and_then(|action| match action.action {
                AIAgentActionType::WriteToLongRunningShellCommand { input, mode, .. } => {
                    Some(render_blocked_action(
                        BlockedActionProps {
                            header: BLOCKED_ACTION_MESSAGE_FOR_WRITE_TO_LONG_RUNNING_SHELL_COMMAND
                                .to_string(),
                            description: Some(render_write_to_pty_input(
                                WriteToPtyInputProps {
                                    input: input.clone(),
                                    mode,
                                },
                                app,
                            )),
                            is_allow_menu_open: self.is_allow_menu_open,
                            allow_menu: Some(&self.allow_menu),
                            buttons: vec![
                                &self.allow_button,
                                &self.reject_button,
                                &self.take_over_button,
                            ],
                            speedbump: should_show_write_to_pty_speedbump(app).then_some(
                                PermissionsSpeedbumpProps {
                                    always_allow_checked: self.always_allow_write_to_pty_checked,
                                    speedbump_checkbox_handle: &self
                                        .state_handles
                                        .speedbump_checkbox_handle,
                                    speedbump_checkbox_action:
                                        CLISubagentAction::ToggleAlwaysAllowWriteToPty,
                                    ai_settings_link: &self.state_handles.ai_settings_link,
                                },
                            ),
                        },
                        app,
                    ))
                }
                AIAgentActionType::TransferShellCommandControlToUser { ref reason } => {
                    Some(render_blocked_action(
                        BlockedActionProps {
                            header: BLOCKED_ACTION_MESSAGE_FOR_TRANSFER_CONTROL.to_string(),
                            description: Some(render_transfer_control_reason(reason, app)),
                            is_allow_menu_open: false,
                            allow_menu: None,
                            buttons: vec![&self.reject_button, &self.transfer_control_button],
                            speedbump: None,
                        },
                        app,
                    ))
                }
                AIAgentActionType::ReadFiles(..)
                | AIAgentActionType::Grep { .. }
                | AIAgentActionType::FileGlobV2 { .. } => Some(render_blocked_action(
                    BlockedActionProps {
                        header: get_blocked_action_header(action.action.clone())
                            .unwrap_or_default(),
                        description: render_search_action_input(action.action.clone(), app),
                        is_allow_menu_open: self.is_allow_menu_open,
                        allow_menu: Some(&self.allow_menu),
                        buttons: vec![
                            &self.allow_button,
                            &self.reject_button,
                            &self.take_over_button,
                        ],
                        speedbump: should_show_read_files_speedbump(app).then_some(
                            PermissionsSpeedbumpProps {
                                always_allow_checked: self.always_allow_read_files_checked,
                                speedbump_checkbox_handle: &self
                                    .state_handles
                                    .speedbump_checkbox_handle,
                                speedbump_checkbox_action:
                                    CLISubagentAction::ToggleAlwaysAllowReadFiles,
                                ai_settings_link: &self.state_handles.ai_settings_link,
                            },
                        ),
                    },
                    app,
                )),
                _ => None,
            }) {
                let action_selection_handle = selection_handle_for_index(
                    &self.state_handles.action_selection_handles,
                    model_index,
                );
                let selected_text = self.selected_text.clone();
                let query_selection_handles = self.state_handles.query_selection_handles.clone();
                let output_selection_handles = self.state_handles.output_selection_handles.clone();
                let action_selection_handles = self.state_handles.action_selection_handles.clone();
                // 克隆一份本区域句柄进闭包，判断"本区域是否真的在参与选择"。
                // Flex 会把同一鼠标事件广播给所有兄弟 SelectableArea，未命中的气泡也会触发本回调，
                // 此时不能用未命中的回调去清掉真正命中区域的划词状态。
                let action_selection_handle_clone = action_selection_handle.clone();
                let mut selectable_action = SelectableArea::new(
                    action_selection_handle,
                    move |selection_args, ctx, _| {
                        let selection = selection_args.selection;
                        // 只有本区域确实参与选择时（正在 selecting 或已产生非空选中文本），
                        // 才清掉其它同级区域的旧选择；未命中广播则保持原状。
                        let is_this_area_active = action_selection_handle_clone.is_selecting()
                            || selection.as_ref().is_some_and(|s| !s.is_empty());
                        if is_this_area_active {
                            clear_selection_handles_for_active_area(
                                &query_selection_handles,
                                &output_selection_handles,
                                &action_selection_handles,
                                SelectionHandleGroup::Action,
                                model_index,
                            );
                        }
                        if let Some(selection) = selection.filter(|selection| !selection.is_empty())
                        {
                            ctx.dispatch_typed_action(CLISubagentAction::CopyOnSelect(
                                selection.clone(),
                            ));
                            *selected_text.write() = Some(selection);
                            ctx.dispatch_typed_action(CLISubagentAction::SelectText);
                        } else if is_this_area_active {
                            *selected_text.write() = None;
                        }
                    },
                    rendered_action,
                )
                .with_word_boundaries_policy(semantic_selection.word_boundary_policy())
                .with_smart_select_fn(semantic_selection.smart_select_fn());

                if FeatureFlag::RectSelection.is_enabled() {
                    selectable_action = selectable_action.should_support_rect_select();
                }

                conversation_items.add_child(
                    Container::new(selectable_action.finish())
                        .with_margin_bottom(8.)
                        .finish(),
                );
            }
        }

        let bottom_position_id =
            format!("{CONVERSATION_SCROLL_BOTTOM_POSITION_ID}-{}", self.block_id);
        conversation_items.add_child(
            SavePosition::new(
                ConstrainedBox::new(Empty::new().finish())
                    .with_height(1.)
                    .finish(),
                &bottom_position_id,
            )
            .finish(),
        );

        if self.is_conversation_scroll_pinned_to_bottom {
            self.state_handles
                .conversation_scroll_state
                .scroll_to_position(ScrollTarget {
                    position_id: bottom_position_id,
                    mode: ScrollToPositionMode::FullyIntoView,
                });
        }

        let scrollable_content = NewScrollable::vertical(
            SingleAxisConfig::Clipped {
                handle: self.state_handles.conversation_scroll_state.clone(),
                child: conversation_items.finish(),
            },
            Fill::None,
            Fill::None,
            Fill::None,
        )
        .with_propagate_mousewheel_if_not_handled(true)
        .finish();

        let clipped_content = ConstrainedBox::new(scrollable_content)
            .with_max_height(resizable_height)
            .finish();
        let content = EventHandler::new(clipped_content)
            .with_always_handle()
            .on_scroll_wheel(|ctx, _app, delta, _| {
                ctx.dispatch_typed_action(CLISubagentAction::ConversationScrollWheel {
                    vertical_delta: delta.y(),
                });
                cli_subagent_conversation_scroll_wheel_dispatch_result()
            })
            .finish();
        let width_resizable = Resizable::new(self.resizable_width.clone(), content)
            .with_dragbar_side(DragBarSide::Left)
            .on_resize(|ctx, _| ctx.notify())
            .with_bounds_callback(Box::new(|window_size| {
                cli_subagent_width_bounds(window_size.x())
            }))
            .finish();

        // 外层负责纵向缩放，内层负责横向缩放；拖拽边放在左上两侧，贴合右下角浮窗形态。
        Resizable::new(self.resizable_height.clone(), width_resizable)
            .with_dragbar_side(DragBarSide::Top)
            .on_resize(|ctx, _| ctx.notify())
            .with_bounds_callback(Box::new(|window_size| {
                cli_subagent_height_bounds(window_size.y())
            }))
            .finish()
    }

    fn keymap_context(&self, app: &AppContext) -> warpui::keymap::Context {
        let mut context = Self::default_keymap_context();

        let terminal_model = self.terminal_model.lock();
        let active_block = terminal_model.block_list().active_block();
        if active_block.is_agent_blocked() {
            context.set.insert(HAS_PENDING_CLI_ACTION_CONTEXT_KEY);
            if !self.has_pending_transfer_control_action(app) {
                context
                    .set
                    .insert(HAS_PENDING_NON_TRANSFER_CONTROL_ACTION_CONTEXT_KEY);
            }
        }
        context
    }
}

#[derive(Debug, Clone)]
pub enum CLISubagentAction {
    CopyCode(String),
    OpenCodeBlock(CodeSource),
    ExecuteBlockedAction,
    ExecuteAndAutoApprove,
    RejectBlockedAction { should_user_take_over: bool },
    TakeControlOfRunningCommand,
    ToggleAllowMenu,
    ToggleAlwaysAllowWriteToPty,
    ToggleAlwaysAllowReadFiles,
    DismissInput,
    SelectText,
    CopyOnSelect(String),
    CopyDebugId(String),
    OpenFeedbackDocs,
    ConversationScrollWheel { vertical_delta: f32 },
}

impl TypedActionView for CLISubagentView {
    type Action = CLISubagentAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            CLISubagentAction::CopyCode(code) => {
                ctx.clipboard()
                    .write(ClipboardContent::plain_text(code.clone()));
                let window_id = ctx.window_id();
                ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    toast_stack.add_ephemeral_toast(
                        DismissibleToast::success(crate::t!("common-copied-to-clipboard")),
                        window_id,
                        ctx,
                    );
                });
            }
            CLISubagentAction::OpenCodeBlock(source) => {
                // TODO(zachbai): Implement this.
                log::info!("Received open code block action: {source:?}");
            }
            CLISubagentAction::ExecuteBlockedAction => {
                self.handle_execute_blocked_action(false, ctx);
            }
            CLISubagentAction::ExecuteAndAutoApprove => {
                self.handle_execute_blocked_action(true, ctx);
            }
            CLISubagentAction::RejectBlockedAction {
                should_user_take_over,
            } => {
                self.handle_reject_blocked_action(*should_user_take_over, ctx);
            }
            CLISubagentAction::TakeControlOfRunningCommand => {
                self.take_control_of_running_command(ctx);
            }
            CLISubagentAction::ToggleAllowMenu => {
                self.toggle_allow_menu(ctx);
            }
            CLISubagentAction::ToggleAlwaysAllowWriteToPty => {
                self.always_allow_write_to_pty_checked = !self.always_allow_write_to_pty_checked;
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    if let Err(e) = model.set_always_allow_write_to_pty(
                        self.always_allow_write_to_pty_checked,
                        self.terminal_view_id,
                        ctx,
                    ) {
                        report_error!(e);
                    }
                });
                ctx.notify();
            }
            CLISubagentAction::ToggleAlwaysAllowReadFiles => {
                self.always_allow_read_files_checked = !self.always_allow_read_files_checked;
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    if let Err(e) = model.set_always_allow_read_files(
                        self.always_allow_read_files_checked,
                        self.terminal_view_id,
                        ctx,
                    ) {
                        report_error!(e);
                    }
                });
                ctx.notify();
            }
            CLISubagentAction::DismissInput => {
                self.is_input_dismissed = true;
                if let Some(handle) = self.input_dismiss_timer_handle.take() {
                    handle.abort();
                }
                ctx.notify();
                send_telemetry_from_ctx!(
                    TelemetryEvent::CLISubagentInputDismissed {
                        conversation_id: self.conversation_id,
                        block_id: self.block_id.clone(),
                    },
                    ctx
                );
            }
            CLISubagentAction::SelectText => {
                self.clear_other_selections(None, ctx);
                ctx.reset_cursor();
                ctx.focus_self();
                ctx.emit(CLISubagentViewEvent::TextSelected);
            }
            CLISubagentAction::CopyOnSelect(selection) => {
                self.maybe_copy_on_select(selection.clone(), ctx);
            }
            CLISubagentAction::CopyDebugId(debug_id) => {
                ctx.clipboard()
                    .write(ClipboardContent::plain_text(debug_id.clone()));
            }
            CLISubagentAction::OpenFeedbackDocs => {
                ctx.open_url("");
            }
            CLISubagentAction::ConversationScrollWheel { vertical_delta } => {
                let did_change = cli_subagent_update_conversation_scroll_pin_after_wheel(
                    &mut self.is_conversation_scroll_pinned_to_bottom,
                    *vertical_delta,
                );
                if did_change {
                    ctx.notify();
                }
            }
        }
    }
}

fn should_show_write_to_pty_speedbump(app: &AppContext) -> bool {
    is_agent_mode_autonomy_allowed(app)
        && *AISettings::as_ref(app).should_show_agent_mode_write_to_pty_speedbump
}

fn should_show_read_files_speedbump(app: &AppContext) -> bool {
    is_agent_mode_autonomy_allowed(app)
        && *AISettings::as_ref(app).should_show_agent_mode_autoread_files_speedbump
}

fn get_action_loading_text(action: AIAgentActionType) -> Option<String> {
    match action {
        AIAgentActionType::ReadFiles(_) => Some(LOAD_OUTPUT_MESSAGE_FOR_READING_FILES.to_string()),
        AIAgentActionType::Grep { .. } => Some(LOAD_OUTPUT_MESSAGE_FOR_GREP.to_string()),
        AIAgentActionType::FileGlobV2 { .. } => Some(LOAD_OUTPUT_MESSAGE_FOR_FILE_GLOB.to_string()),
        _ => None,
    }
}

fn get_action_icon(action: AIAgentActionType) -> Option<Icon> {
    match action {
        AIAgentActionType::ReadFiles(_)
        | AIAgentActionType::Grep { .. }
        | AIAgentActionType::FileGlobV2 { .. } => Some(Icon::Search),
        _ => None,
    }
}

fn render_action(action: AIAgentActionType, app: &AppContext) -> Option<Box<dyn Element>> {
    let appearance = Appearance::as_ref(app);
    let theme = appearance.theme();

    let text = get_action_loading_text(action.clone())?;
    let icon = get_action_icon(action)?;

    let icon = Container::new(
        ConstrainedBox::new(
            warpui::elements::Icon::new(icon.into(), internal_colors::neutral_5(theme)).finish(),
        )
        .with_width(icon_size(app))
        .with_height(icon_size(app))
        .finish(),
    )
    .with_margin_right(AVATAR_RIGHT_MARGIN)
    .finish();

    let text = Expanded::new(
        1.,
        Text::new(
            text,
            appearance.monospace_font_family(),
            appearance.monospace_font_size(),
        )
        .with_color(blended_colors::text_main(theme, theme.surface_1()))
        .finish(),
    )
    .finish();

    let row = Flex::row()
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_main_axis_alignment(MainAxisAlignment::Center)
        .with_children([icon, text])
        .finish();

    Some(row)
}

fn render_web_search(query: Option<String>, app: &AppContext) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    let theme = appearance.theme();

    let text = if let Some(q) = query {
        format!("Searching the web for \"{q}\"")
    } else {
        LOAD_OUTPUT_MESSAGE_FOR_WEB_SEARCH.to_string()
    };

    let icon = Container::new(
        ConstrainedBox::new(
            warpui::elements::Icon::new(Icon::Search.into(), internal_colors::neutral_5(theme))
                .finish(),
        )
        .with_width(icon_size(app))
        .with_height(icon_size(app))
        .finish(),
    )
    .with_margin_right(AVATAR_RIGHT_MARGIN)
    .finish();

    let text = Expanded::new(
        1.,
        Text::new(
            text,
            appearance.monospace_font_family(),
            appearance.monospace_font_size(),
        )
        .with_color(blended_colors::text_main(theme, theme.surface_1()))
        .finish(),
    )
    .finish();

    Flex::row()
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_main_axis_alignment(MainAxisAlignment::Center)
        .with_children([icon, text])
        .finish()
}

struct DismissableContainerProps {
    child: Box<dyn Element>,
    hover_state: MouseStateHandle,
    dismiss_mouse_state: MouseStateHandle,
    position_id: String,
}

fn render_dismissable_container(
    props: DismissableContainerProps,
    app: &AppContext,
) -> Box<dyn Element> {
    let DismissableContainerProps {
        child,
        hover_state,
        dismiss_mouse_state,
        position_id,
    } = props;

    let hoverable = Hoverable::new(hover_state, |mouse_state| {
        let mut stack = Stack::new().with_child(SavePosition::new(child, &position_id).finish());
        if mouse_state.is_hovered() {
            let appearance = Appearance::as_ref(app);
            let theme = appearance.theme();
            let ui_builder = appearance.ui_builder();

            let dismiss_button = Container::new(
                ui_builder
                    .close_button(16., dismiss_mouse_state.clone())
                    .with_style(UiComponentStyles {
                        font_color: Some(blended_colors::text_main(theme, theme.surface_1())),
                        background: Some(internal_colors::accent_bg(theme).into()),
                        border_radius: Some(CornerRadius::with_all(Radius::Percentage(50.))),
                        border_width: Some(1.),
                        border_color: Some(theme.accent().into()),
                        padding: Some(Coords::uniform(2.)),
                        ..Default::default()
                    })
                    .build()
                    .on_click(move |ctx, _, _| {
                        ctx.dispatch_typed_action(CLISubagentAction::DismissInput);
                    })
                    .finish(),
            )
            .finish();

            stack.add_positioned_child(
                dismiss_button,
                OffsetPositioning::offset_from_save_position_element(
                    position_id,
                    vec2f(4., -4.),
                    PositionedElementOffsetBounds::WindowByPosition,
                    PositionedElementAnchor::TopRight,
                    ChildAnchor::TopRight,
                ),
            );
        }
        stack.finish()
    });

    hoverable
        .with_hover_out_delay(Duration::from_millis(500))
        .finish()
}
struct FramedContainerProps {
    child: Box<dyn Element>,
    background_color: ColorU,
    border: Option<Border>,
}

fn render_framed_container(props: FramedContainerProps) -> Container {
    let FramedContainerProps {
        child,
        background_color,
        border,
    } = props;

    let mut container = Container::new(child)
        .with_background_color(background_color)
        .with_horizontal_padding(CONTENT_PADDING)
        .with_vertical_padding(CONTENT_PADDING)
        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
        .with_drop_shadow(DropShadow::default());

    if let Some(border) = border {
        container = container.with_border(border);
    }

    container
}

fn render_action_buttons(
    buttons: Vec<&dyn RenderCompactibleActionButton>,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    let theme = appearance.theme();

    let (regular_row, compact_row) =
        render_compact_and_regular_button_rows(buttons, None, appearance, app);

    let regular_wrapped = Container::new(regular_row)
        .with_vertical_padding(8.)
        .with_horizontal_padding(CONTENT_PADDING)
        .with_background_color(internal_colors::neutral_2(theme))
        .with_border(Border::top(1.).with_border_color(internal_colors::neutral_3(theme)))
        .finish();

    let compact_wrapped = Container::new(compact_row)
        .with_vertical_padding(8.)
        .with_horizontal_padding(CONTENT_PADDING)
        .with_background_color(internal_colors::neutral_2(theme))
        .with_border(Border::top(1.).with_border_color(internal_colors::neutral_3(theme)))
        .finish();

    let size_switch_threshold = 250. * appearance.monospace_ui_scalar();
    SizeConstraintSwitch::new(
        regular_wrapped,
        vec![(
            SizeConstraintCondition::WidthLessThan(size_switch_threshold),
            compact_wrapped,
        )],
    )
    .finish()
}

struct PermissionsSpeedbumpProps<'a> {
    always_allow_checked: bool,
    speedbump_checkbox_handle: &'a MouseStateHandle,
    speedbump_checkbox_action: CLISubagentAction,
    ai_settings_link: &'a HighlightedHyperlink,
}

fn render_permissions_speedbump(
    props: PermissionsSpeedbumpProps<'_>,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    let theme = appearance.theme();
    let font_size = appearance.monospace_font_size() - 2.;
    let font_family = appearance.ui_font_family();
    let font_color = internal_colors::neutral_6(theme);

    let checkbox = appearance
        .ui_builder()
        .checkbox(props.speedbump_checkbox_handle.clone(), Some(font_size))
        .check(props.always_allow_checked)
        .with_style(UiComponentStyles {
            font_color: Some(font_color),
            font_size: Some(font_size),
            ..Default::default()
        })
        .build()
        .on_click(move |ctx, _, _| {
            ctx.dispatch_typed_action(props.speedbump_checkbox_action.clone());
        })
        .with_cursor(Cursor::PointingHand)
        .finish();

    let checkbox_text = appearance
        .ui_builder()
        .span(crate::t!("ai-block-always-allow"))
        .with_style(UiComponentStyles {
            font_color: Some(font_color),
            font_size: Some(font_size),
            padding: Some(Coords::default().left(4.)),
            ..Default::default()
        })
        .with_soft_wrap()
        .build()
        .finish();

    let formatted_text = FormattedTextElement::new(
        FormattedText::new([FormattedTextLine::Line(vec![
            FormattedTextFragment::hyperlink(
                crate::t!("ai-block-manage-agent-permissions"),
                "Settings > AI",
            ),
        ])]),
        font_size,
        font_family,
        font_family,
        font_color,
        props.ai_settings_link.clone(),
    )
    .with_heading_to_font_size_multipliers(appearance.heading_font_size_multipliers().clone())
    .with_hyperlink_font_color(blended_colors::accent_fg_strong(theme).into())
    .register_default_click_handlers(|_, ctx, _| {
        ctx.dispatch_typed_action(WorkspaceAction::ShowSettingsPage(
            SettingsSection::WarpAgent,
        ));
    })
    .finish();

    Container::new(
        Flex::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                Shrinkable::new(
                    1.0,
                    Flex::row()
                        .with_cross_axis_alignment(CrossAxisAlignment::Center)
                        .with_child(checkbox)
                        .with_child(Shrinkable::new(1.0, checkbox_text).finish())
                        .finish(),
                )
                .finish(),
            )
            .with_child(Shrinkable::new(1.0, formatted_text).finish())
            .finish(),
    )
    .with_vertical_padding(8.)
    .with_horizontal_padding(CONTENT_PADDING)
    .with_background_color(internal_colors::neutral_2(theme))
    .with_border(Border::top(1.).with_border_color(internal_colors::neutral_3(theme)))
    .finish()
}

fn render_transfer_control_reason(reason: &str, app: &AppContext) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    let text = Text::new(
        reason.to_string(),
        appearance.ai_font_family(),
        appearance.monospace_font_size(),
    )
    .with_color(blended_colors::text_main(
        appearance.theme(),
        appearance.theme().surface_1(),
    ))
    .finish();

    Container::new(text)
        .with_background_color(internal_colors::neutral_2(appearance.theme()))
        .with_horizontal_padding(CONTENT_PADDING)
        .with_vertical_padding(8.)
        .finish()
}

fn get_blocked_action_header(action: AIAgentActionType) -> Option<String> {
    match action {
        AIAgentActionType::WriteToLongRunningShellCommand { .. } => {
            Some(BLOCKED_ACTION_MESSAGE_FOR_WRITE_TO_LONG_RUNNING_SHELL_COMMAND.to_string())
        }
        AIAgentActionType::ReadFiles(..) => {
            Some(BLOCKED_ACTION_MESSAGE_FOR_READING_FILES.to_string())
        }
        AIAgentActionType::Grep { .. } | AIAgentActionType::FileGlobV2 { .. } => {
            Some(BLOCKED_ACTION_MESSAGE_FOR_GREP_OR_FILE_GLOB.to_string())
        }
        _ => None,
    }
}

struct WriteToPtyInputProps {
    input: bytes::Bytes,
    mode: AIAgentPtyWriteMode,
}

fn render_write_to_pty_input(props: WriteToPtyInputProps, app: &AppContext) -> Box<dyn Element> {
    let WriteToPtyInputProps { input, mode } = props;

    let appearance = Appearance::as_ref(app);
    let theme = appearance.theme();

    let decorated_bytes = mode.decorate_bytes(input.to_vec(), false);
    let parsed = if let AIAgentPtyWriteMode::Block = mode {
        ParsedControlCodeOutput {
            display: String::from_utf8_lossy(&input).to_string(),
            control_code_ranges: vec![],
        }
    } else {
        parse_control_codes_from_bytes(&decorated_bytes)
    };

    let text = Text::new(
        parsed.display,
        appearance.monospace_font_family(),
        appearance.monospace_font_size(),
    )
    .with_color(theme.sub_text_color(theme.background()).into())
    .with_single_highlight(
        Highlight::new()
            .with_foreground_color(theme.hint_text_color(theme.surface_2()).into())
            .with_properties(Properties {
                style: Style::Italic,
                ..Default::default()
            }),
        parsed.control_code_ranges.into_iter().flatten().collect(),
    )
    .finish();

    Container::new(text)
        .with_background_color(internal_colors::neutral_2(theme))
        .with_horizontal_padding(CONTENT_PADDING)
        .with_vertical_padding(8.)
        .finish()
}

fn render_search_action_input(
    action: AIAgentActionType,
    app: &AppContext,
) -> Option<Box<dyn Element>> {
    let appearance = Appearance::as_ref(app);
    let theme = appearance.theme();

    let description_text = match action {
        AIAgentActionType::ReadFiles(ref request) => request
            .locations
            .iter()
            .map(|loc| loc.name.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        AIAgentActionType::Grep {
            ref queries,
            ref path,
        } => {
            let display_path = if path == "." {
                "the current directory"
            } else {
                path.as_str()
            };

            if queries.len() == 1 {
                format!("Grep for `{}` in {}", queries[0], display_path)
            } else {
                let patterns_list = queries
                    .iter()
                    .map(|q| format!(" - `{q}`"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("Grep for the following patterns in {display_path}:\n{patterns_list}")
            }
        }
        AIAgentActionType::FileGlobV2 {
            ref patterns,
            ref search_dir,
        } => {
            let display_path = search_dir.as_deref().unwrap_or("the current directory");

            if patterns.len() == 1 {
                format!(
                    "Search for files that match `{}` in {}",
                    patterns[0], display_path
                )
            } else {
                let patterns_list = patterns
                    .iter()
                    .map(|p| format!(" - `{p}`"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "Find files that match the following patterns in {display_path}:\n{patterns_list}"
                )
            }
        }
        _ => return None,
    };

    let text = Text::new(
        description_text,
        appearance.monospace_font_family(),
        appearance.monospace_font_size(),
    )
    .with_color(blended_colors::text_main(theme, theme.surface_1()))
    .finish();

    Some(
        Container::new(text)
            .with_background_color(internal_colors::neutral_2(theme))
            .with_uniform_padding(CONTENT_PADDING)
            .finish(),
    )
}

struct BlockedActionProps<'a> {
    header: String,
    description: Option<Box<dyn Element>>,

    is_allow_menu_open: bool,
    allow_menu: Option<&'a ViewHandle<Menu<CLISubagentAction>>>,
    buttons: Vec<&'a dyn RenderCompactibleActionButton>,
    speedbump: Option<PermissionsSpeedbumpProps<'a>>,
}

fn render_blocked_action(props: BlockedActionProps<'_>, app: &AppContext) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    let theme = appearance.theme();

    let header_text = props.header.clone();
    let text = Text::new(
        header_text,
        appearance.ui_font_family(),
        appearance.monospace_font_size(),
    )
    .with_color(theme.active_ui_text_color().into())
    .finish();

    let icon = Container::new(
        ConstrainedBox::new(yellow_stop_icon(appearance).finish())
            .with_width(icon_size(app))
            .with_height(icon_size(app))
            .finish(),
    )
    .with_margin_right(AVATAR_RIGHT_MARGIN)
    .finish();

    let header = Container::new(
        Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_children(vec![icon, Shrinkable::new(1.0, text).finish()])
            .finish(),
    )
    .with_background_color(internal_colors::neutral_3(theme))
    .with_uniform_padding(CONTENT_PADDING)
    .finish();

    let mut body_children = vec![header];

    if let Some(description) = props.description {
        body_children.push(description);
    }

    let buttons = render_action_buttons(props.buttons, app);
    body_children.push(buttons);
    if let Some(speedbump) = props.speedbump {
        body_children.push(render_permissions_speedbump(speedbump, app));
    }

    let body = Flex::column()
        .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
        .with_children(body_children)
        .finish();

    let mut stack = Stack::new();
    stack.add_child(
        Container::new(body)
            .with_drop_shadow(DropShadow::default())
            .finish(),
    );

    if props.is_allow_menu_open {
        if let Some(allow_menu) = props.allow_menu {
            stack.add_positioned_child(
                ChildView::new(allow_menu).finish(),
                OffsetPositioning::offset_from_save_position_element(
                    ALLOW_ACTION_POSITION_ID.to_string(),
                    vec2f(0., 8.),
                    PositionedElementOffsetBounds::WindowByPosition,
                    PositionedElementAnchor::BottomRight,
                    ChildAnchor::TopRight,
                ),
            );
        }
    }

    Expanded::new(
        1.0,
        Container::new(stack.finish())
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
            .with_border(Border::all(1.).with_border_color(internal_colors::neutral_3(theme)))
            .finish(),
    )
    .finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent::{AIAgentOutputMessage, AgentOutputText, MessageId};
    use warpui::{elements::SelectionBound, text::SelectionType};

    fn cli_subagent_test_text_output(
        message_id: &str,
        sections: Vec<AIAgentTextSection>,
    ) -> AIAgentOutput {
        AIAgentOutput {
            messages: vec![AIAgentOutputMessage::text(
                MessageId::new(message_id.to_string()),
                AIAgentText { sections },
            )],
            ..Default::default()
        }
    }

    fn cli_subagent_test_reasoning_message(message_id: &str) -> AIAgentOutputMessage {
        AIAgentOutputMessage::reasoning(
            MessageId::new(message_id.to_string()),
            AIAgentText {
                sections: vec![AIAgentTextSection::PlainText {
                    text: AgentOutputText::from("hidden reasoning".to_string()),
                }],
            },
            None,
        )
    }

    fn cli_subagent_test_plain_text_section(text: &str) -> AIAgentTextSection {
        AIAgentTextSection::PlainText {
            text: AgentOutputText::from(text.to_string()),
        }
    }

    #[test]
    fn cli_subagent_resize_width_bounds_allow_nearly_full_window() {
        assert_eq!(cli_subagent_width_bounds(1000.0), (360.0, 984.0));
    }

    #[test]
    fn cli_subagent_resize_width_bounds_do_not_drop_below_panel_minimum() {
        assert_eq!(cli_subagent_width_bounds(320.0), (360.0, 360.0));
    }

    #[test]
    fn cli_subagent_resize_height_bounds_allow_nearly_full_window() {
        assert_eq!(cli_subagent_height_bounds(700.0), (40.0, 684.0));
    }

    #[test]
    fn cli_subagent_resize_height_bounds_do_not_drop_below_panel_minimum() {
        assert_eq!(cli_subagent_height_bounds(50.0), (40.0, 40.0));
    }

    #[test]
    fn cli_subagent_selection_handles_clear_other_items_only() {
        let handles = SelectionHandleList::default();
        let first_handle = selection_handle_for_index(&handles, 0);
        let second_handle = selection_handle_for_index(&handles, 1);

        first_handle.start_selection_outside(SelectionBound::TopLeft, SelectionType::Simple);
        second_handle.start_selection_outside(SelectionBound::TopLeft, SelectionType::Simple);

        clear_selection_handles_except(&handles, 1);

        assert!(!first_handle.is_selecting());
        assert!(second_handle.is_selecting());
    }

    #[test]
    fn cli_subagent_active_selection_clears_peer_groups() {
        let query_handles = SelectionHandleList::default();
        let output_handles = SelectionHandleList::default();
        let action_handles = SelectionHandleList::default();
        let query_handle = selection_handle_for_index(&query_handles, 0);
        let output_handle = selection_handle_for_index(&output_handles, 0);
        let action_handle = selection_handle_for_index(&action_handles, 0);

        query_handle.start_selection_outside(SelectionBound::TopLeft, SelectionType::Simple);
        output_handle.start_selection_outside(SelectionBound::TopLeft, SelectionType::Simple);
        action_handle.start_selection_outside(SelectionBound::TopLeft, SelectionType::Simple);

        clear_selection_handles_for_active_area(
            &query_handles,
            &output_handles,
            &action_handles,
            SelectionHandleGroup::Output,
            0,
        );

        assert!(!query_handle.is_selecting());
        assert!(output_handle.is_selecting());
        assert!(!action_handle.is_selecting());
    }

    #[test]
    fn cli_subagent_history_exchange_ids_append_new_rounds_in_order() {
        let first_exchange_id = crate::ai::agent::AIAgentExchangeId::new();
        let second_exchange_id = crate::ai::agent::AIAgentExchangeId::new();
        let mut exchange_ids = Vec::new();

        assert!(cli_subagent_append_history_exchange_id(
            &mut exchange_ids,
            first_exchange_id
        ));
        assert!(cli_subagent_append_history_exchange_id(
            &mut exchange_ids,
            second_exchange_id
        ));

        assert_eq!(exchange_ids, vec![first_exchange_id, second_exchange_id]);
    }

    #[test]
    fn cli_subagent_history_exchange_ids_ignore_duplicate_exchange() {
        let exchange_id = crate::ai::agent::AIAgentExchangeId::new();
        let mut exchange_ids = Vec::new();

        assert!(cli_subagent_append_history_exchange_id(
            &mut exchange_ids,
            exchange_id
        ));
        assert!(!cli_subagent_append_history_exchange_id(
            &mut exchange_ids,
            exchange_id
        ));

        assert_eq!(exchange_ids, vec![exchange_id]);
    }

    #[test]
    fn cli_subagent_output_redaction_section_count_matches_rendered_text_messages() {
        let mut output = cli_subagent_test_text_output(
            "message-1",
            vec![
                cli_subagent_test_plain_text_section("first"),
                cli_subagent_test_plain_text_section("second"),
            ],
        );
        output
            .messages
            .push(cli_subagent_test_reasoning_message("reasoning-message"));

        assert_eq!(cli_subagent_rendered_output_text_section_count(&output), 2);
    }

    #[test]
    fn cli_subagent_output_redaction_section_indices_accumulate_across_history() {
        let history_output = cli_subagent_test_text_output(
            "history-message",
            vec![
                cli_subagent_test_plain_text_section("history 1"),
                cli_subagent_test_plain_text_section("history 2"),
            ],
        );
        let latest_output = cli_subagent_test_text_output(
            "latest-message",
            vec![cli_subagent_test_plain_text_section("latest")],
        );
        let history_start_index = 0;
        let latest_start_index =
            history_start_index + cli_subagent_rendered_output_text_section_count(&history_output);
        let final_section_index =
            latest_start_index + cli_subagent_rendered_output_text_section_count(&latest_output);

        assert_eq!(latest_start_index, 2);
        assert_eq!(final_section_index, 3);
    }

    #[test]
    fn cli_subagent_conversation_scroll_unpins_after_manual_scroll() {
        let mut is_pinned = true;

        cli_subagent_mark_conversation_scroll_manually_moved(&mut is_pinned);

        assert!(!is_pinned);
    }

    #[test]
    fn cli_subagent_conversation_scroll_pins_after_new_exchange() {
        let mut is_pinned = false;

        cli_subagent_mark_conversation_scroll_should_follow_latest(&mut is_pinned);

        assert!(is_pinned);
    }

    #[test]
    fn cli_subagent_response_visibility_restores_saved_scroll_offset() {
        let mut saved_scroll_offset = None;

        let restored_scroll_offset =
            cli_subagent_response_visibility_scroll_offset(120.0, true, &mut saved_scroll_offset);

        assert_eq!(saved_scroll_offset, Some(120.0));
        assert_eq!(restored_scroll_offset, None);

        let restored_scroll_offset =
            cli_subagent_response_visibility_scroll_offset(0.0, false, &mut saved_scroll_offset);

        assert_eq!(saved_scroll_offset, None);
        assert_eq!(restored_scroll_offset, Some(120.0));
    }

    #[test]
    fn cli_subagent_conversation_scroll_wheel_stops_parent_propagation() {
        assert!(matches!(
            cli_subagent_conversation_scroll_wheel_dispatch_result(),
            DispatchEventResult::StopPropagation
        ));
    }

    #[test]
    fn cli_subagent_user_input_hidden_when_responses_are_hidden() {
        assert!(!cli_subagent_should_render_user_input(true, false));
    }

    #[test]
    fn cli_subagent_user_input_visible_when_responses_are_shown() {
        assert!(cli_subagent_should_render_user_input(false, false));
    }

    #[test]
    fn cli_subagent_user_input_hidden_after_manual_dismiss() {
        assert!(!cli_subagent_should_render_user_input(false, true));
    }

    #[test]
    fn cli_subagent_conversation_scroll_keeps_pinned_when_confirming_bottom() {
        let mut is_pinned = true;

        cli_subagent_update_conversation_scroll_pin_after_wheel(&mut is_pinned, -1.);

        assert!(is_pinned);
    }

    #[test]
    fn cli_subagent_conversation_scroll_unpins_when_scrolling_up_from_bottom() {
        let mut is_pinned = true;

        cli_subagent_update_conversation_scroll_pin_after_wheel(&mut is_pinned, 1.);

        assert!(!is_pinned);
    }
}
