use super::*;

fn kind(name: &str) -> RedactedToolKind {
    RedactedToolKind::new(name)
}

fn key(task_id: &str, assistant_message_id: &str, tool_call_id: &str) -> ToolCallKey {
    ToolCallKey::new(task_id, assistant_message_id, tool_call_id)
}

fn call(task_id: &str, assistant_message_id: &str, tool_call_id: &str) -> ProjectedToolCall {
    ProjectedToolCall::new(task_id, assistant_message_id, tool_call_id, kind("shell"))
}

fn call_ref(task_id: &str, assistant_message_id: &str, tool_call_id: &str) -> ToolCallRef {
    ToolCallRef::new(
        key(task_id, assistant_message_id, tool_call_id),
        kind("shell"),
    )
}

fn assistant_with_one_call() -> ProjectionItem {
    ProjectionItem::assistant_tool_calls(
        "task-1",
        "assistant-1",
        vec![call("task-1", "assistant-1", "call-1")],
    )
}

fn persisted_result(message_id: &str, result_kind: TerminalResultKind) -> ProjectionItem {
    ProjectionItem::tool_result(ProjectedToolResult::new(
        "task-1",
        message_id,
        Some("assistant-1".to_string()),
        "call-1",
        kind("shell"),
        ToolResultSource::PersistedHistory,
        result_kind,
    ))
}

fn classify(items: Vec<ProjectionItem>) -> ReadinessReport {
    classify_projection(&items, &ReadinessContext::default())
}

#[test]
fn complete_real_result_is_ready() {
    let report = classify(vec![
        assistant_with_one_call(),
        persisted_result("result-1", TerminalResultKind::Real),
    ]);

    assert_eq!(report.state, ReadinessState::Ready);
    assert!(report.ignored_repair_records.is_empty());
}

#[test]
fn current_input_result_uses_synthetic_projection_id_and_satisfies_readiness() {
    let result = ProjectedToolResult::current_input(
        3,
        "task-1",
        "assistant-1",
        "call-1",
        kind("shell"),
        TerminalResultKind::Cancellation,
    );
    assert_eq!(result.message_id, "current_input:3:call-1");
    assert_eq!(result.source, ToolResultSource::CurrentInput);

    let report = classify(vec![
        assistant_with_one_call(),
        ProjectionItem::tool_result(result),
    ]);

    assert_eq!(report.state, ReadinessState::Ready);
}

#[test]
fn compacted_structured_error_and_local_interception_results_satisfy_readiness() {
    for result_kind in [
        TerminalResultKind::Compacted,
        TerminalResultKind::StructuredError,
        TerminalResultKind::LocalInterception,
    ] {
        let report = classify(vec![
            assistant_with_one_call(),
            persisted_result("result-1", result_kind),
        ]);
        assert_eq!(report.state, ReadinessState::Ready);
    }
}

#[test]
fn accepted_history_repair_is_distinct_from_ready() {
    let repair = RepairRecord::new(
        RepairSource::ForkedHistory,
        key("task-1", "assistant-1", "call-1"),
    );
    let context = ReadinessContext {
        repair_records: vec![repair.clone()],
        live_tool_calls: Vec::new(),
    };

    let report = classify_projection(&[assistant_with_one_call()], &context);

    assert_eq!(
        report.state,
        ReadinessState::AcceptedHistoryRepair {
            repairs: vec![AcceptedRepair {
                record: repair,
                tool_call: call_ref("task-1", "assistant-1", "call-1"),
            }],
        }
    );
}

#[test]
fn repair_record_must_match_full_key_not_only_tool_call_id() {
    let stale_repair = RepairRecord::new(
        RepairSource::ForkedHistory,
        key("other-task", "assistant-1", "call-1"),
    );
    let context = ReadinessContext {
        repair_records: vec![stale_repair.clone()],
        live_tool_calls: Vec::new(),
    };

    let report = classify_projection(&[assistant_with_one_call()], &context);

    assert_eq!(
        report.state,
        ReadinessState::MissingResultWithoutRepairSource {
            tool_calls: vec![call_ref("task-1", "assistant-1", "call-1")],
            reason: MissingResultReason::NoResult,
        }
    );
    assert_eq!(
        report.ignored_repair_records,
        vec![IgnoredRepairRecord {
            record: stale_repair,
            category: ReadinessCategory::StaleRepairRecordIgnored,
        }]
    );
}

#[test]
fn stale_repair_record_for_satisfied_visible_call_is_ignored() {
    let stale_repair = RepairRecord::new(
        RepairSource::ForkedHistory,
        key("task-1", "assistant-1", "call-1"),
    );
    let context = ReadinessContext {
        repair_records: vec![stale_repair.clone()],
        live_tool_calls: Vec::new(),
    };

    let report = classify_projection(
        &[
            assistant_with_one_call(),
            persisted_result("result-1", TerminalResultKind::Real),
        ],
        &context,
    );

    assert_eq!(report.state, ReadinessState::Ready);
    assert_eq!(
        report.ignored_repair_records,
        vec![IgnoredRepairRecord {
            record: stale_repair,
            category: ReadinessCategory::StaleRepairRecordIgnored,
        }]
    );
}

#[test]
fn hidden_repair_record_is_unused_without_stale_diagnostic() {
    let hidden_repair = RepairRecord::new(
        RepairSource::ForkedHistory,
        key("task-1", "assistant-hidden", "call-hidden"),
    );
    let context = ReadinessContext {
        repair_records: vec![hidden_repair],
        live_tool_calls: Vec::new(),
    };

    let report = classify_projection(&[], &context);

    assert_eq!(report.state, ReadinessState::Ready);
    assert!(report.ignored_repair_records.is_empty());
}

#[test]
fn running_live_action_is_pending() {
    let context = ReadinessContext {
        repair_records: Vec::new(),
        live_tool_calls: vec![LiveToolCall::new(
            call_ref("task-1", "assistant-1", "call-1"),
            LiveToolCallState::Running,
        )],
    };

    let report = classify_projection(&[assistant_with_one_call()], &context);

    assert_eq!(
        report.state,
        ReadinessState::PendingToolResults {
            tool_calls: vec![call_ref("task-1", "assistant-1", "call-1")],
        }
    );
}

#[test]
fn cancellation_requested_returns_needs_cancellation_commit() {
    let context = ReadinessContext {
        repair_records: Vec::new(),
        live_tool_calls: vec![LiveToolCall::new(
            call_ref("task-1", "assistant-1", "call-1"),
            LiveToolCallState::CancellationRequested,
        )],
    };

    let report = classify_projection(&[assistant_with_one_call()], &context);

    assert_eq!(
        report.state,
        ReadinessState::NeedsCancellationCommit {
            tool_calls: vec![call_ref("task-1", "assistant-1", "call-1")],
        }
    );
}

#[test]
fn duplicate_tool_results_block_readiness() {
    let report = classify(vec![
        assistant_with_one_call(),
        persisted_result("result-1", TerminalResultKind::Real),
        persisted_result("result-2", TerminalResultKind::Cancellation),
    ]);

    assert_eq!(
        report.state,
        ReadinessState::DuplicateToolResults {
            tool_call: call_ref("task-1", "assistant-1", "call-1"),
            results: vec![
                ToolResultRef {
                    message_id: "result-1".to_string(),
                    source: ToolResultSource::PersistedHistory,
                    result_kind: TerminalResultKind::Real,
                },
                ToolResultRef {
                    message_id: "result-2".to_string(),
                    source: ToolResultSource::PersistedHistory,
                    result_kind: TerminalResultKind::Cancellation,
                },
            ],
        }
    );
}

#[test]
fn duplicate_projection_preserves_result_source() {
    let current_result = ProjectedToolResult::current_input(
        0,
        "task-1",
        "assistant-1",
        "call-1",
        kind("shell"),
        TerminalResultKind::Cancellation,
    );

    let report = classify(vec![
        assistant_with_one_call(),
        persisted_result("result-1", TerminalResultKind::Real),
        ProjectionItem::tool_result(current_result),
    ]);

    assert_eq!(
        report.state,
        ReadinessState::DuplicateToolResults {
            tool_call: call_ref("task-1", "assistant-1", "call-1"),
            results: vec![
                ToolResultRef {
                    message_id: "result-1".to_string(),
                    source: ToolResultSource::PersistedHistory,
                    result_kind: TerminalResultKind::Real,
                },
                ToolResultRef {
                    message_id: "current_input:0:call-1".to_string(),
                    source: ToolResultSource::CurrentInput,
                    result_kind: TerminalResultKind::Cancellation,
                },
            ],
        }
    );
}

#[test]
fn orphan_tool_result_blocks_readiness() {
    let result = ProjectedToolResult::new(
        "task-1",
        "result-1",
        Some("assistant-unknown".to_string()),
        "call-1",
        kind("shell"),
        ToolResultSource::PersistedHistory,
        TerminalResultKind::Real,
    );

    let report = classify(vec![ProjectionItem::tool_result(result.clone())]);

    assert_eq!(report.state, ReadinessState::OrphanToolResult { result });
}

#[test]
fn tool_result_after_user_boundary_is_out_of_order() {
    let result = ProjectedToolResult::new(
        "task-1",
        "result-1",
        Some("assistant-1".to_string()),
        "call-1",
        kind("shell"),
        ToolResultSource::PersistedHistory,
        TerminalResultKind::Real,
    );

    let report = classify(vec![
        assistant_with_one_call(),
        ProjectionItem::user_boundary("task-1", "user-2"),
        ProjectionItem::tool_result(result.clone()),
    ]);

    assert_eq!(
        report.state,
        ReadinessState::OutOfOrderToolResult { result }
    );
}

#[test]
fn unreadable_local_interception_does_not_satisfy_readiness() {
    let report = classify(vec![
        assistant_with_one_call(),
        persisted_result("result-1", TerminalResultKind::UnreadableLocalInterception),
    ]);

    assert_eq!(
        report.state,
        ReadinessState::MissingResultWithoutRepairSource {
            tool_calls: vec![call_ref("task-1", "assistant-1", "call-1")],
            reason: MissingResultReason::UnreadableLocalInterception,
        }
    );
}

#[test]
fn projection_infers_backref_for_unique_valid_pending_call() {
    let result = ProjectedToolResult::new(
        "task-1",
        "result-1",
        None,
        "call-1",
        kind("shell"),
        ToolResultSource::PersistedHistory,
        TerminalResultKind::Real,
    );
    let normalized = normalize_projection(vec![
        assistant_with_one_call(),
        ProjectionItem::tool_result(result),
    ]);

    let ProjectionItemKind::ToolResult(result) = &normalized[1].kind else {
        panic!("expected tool result");
    };
    assert_eq!(
        result.assistant_tool_call_message_id.as_deref(),
        Some("assistant-1")
    );
}

#[test]
fn projection_does_not_infer_backref_across_boundary() {
    let result = ProjectedToolResult::new(
        "task-1",
        "result-1",
        None,
        "call-1",
        kind("shell"),
        ToolResultSource::PersistedHistory,
        TerminalResultKind::Real,
    );
    let normalized = normalize_projection(vec![
        assistant_with_one_call(),
        ProjectionItem::assistant_boundary("task-1", "assistant-2"),
        ProjectionItem::tool_result(result),
    ]);

    let ProjectionItemKind::ToolResult(result) = &normalized[2].kind else {
        panic!("expected tool result");
    };
    assert!(result.assistant_tool_call_message_id.is_none());
}

#[test]
fn projection_does_not_infer_ambiguous_backref() {
    let result = ProjectedToolResult::new(
        "task-1",
        "result-1",
        None,
        "call-1",
        kind("shell"),
        ToolResultSource::PersistedHistory,
        TerminalResultKind::Real,
    );
    let normalized = normalize_projection(vec![
        ProjectionItem::assistant_tool_calls(
            "task-1",
            "assistant-1",
            vec![
                call("task-1", "assistant-1", "call-1"),
                call("task-1", "assistant-1", "call-1"),
            ],
        ),
        ProjectionItem::tool_result(result.clone()),
    ]);

    let ProjectionItemKind::ToolResult(normalized_result) = &normalized[1].kind else {
        panic!("expected tool result");
    };
    assert!(normalized_result.assistant_tool_call_message_id.is_none());

    let report = classify_projection(&normalized, &ReadinessContext::default());
    assert_eq!(
        report.state,
        ReadinessState::OutOfOrderToolResult { result }
    );
}

#[test]
fn projection_does_not_store_raw_prompt_or_tool_payload() {
    let result = ProjectedToolResult::new(
        "task-1",
        "result-1",
        Some("assistant-1".to_string()),
        "call-1",
        kind("local_interception:websearch"),
        ToolResultSource::PersistedHistory,
        TerminalResultKind::LocalInterception,
    );
    let debug = format!("{result:?}");

    assert!(debug.contains("local_interception:websearch"));
    assert!(!debug.contains("secret user prompt"));
    assert!(!debug.contains("raw tool output"));
    assert!(!debug.contains("raw tool arguments"));
}

#[test]
fn repair_state_missing_sidecar_loads_as_valid_empty_state() {
    let status = RepairStateStatus::from_sidecar_json(None);

    assert_eq!(status, RepairStateStatus::Valid(RepairState::default()));
    assert!(status.repair_records().is_empty());
    assert_eq!(status.to_sidecar_json(), None);
}

#[test]
fn repair_state_accepts_version_one_and_roundtrips_records() {
    let record = RepairRecord::new(
        RepairSource::ForkedHistory,
        key("task-1", "assistant-1", "call-1"),
    );
    let json = serde_json::to_string(&RepairState::new(vec![record.clone()])).unwrap();
    let status = RepairStateStatus::from_sidecar_json(Some(json));

    assert_eq!(
        status,
        RepairStateStatus::Valid(RepairState::new(vec![record.clone()]))
    );
    assert_eq!(status.repair_records(), &[record]);
    assert!(status.to_sidecar_json().unwrap().contains("\"version\":1"));
}

#[test]
fn repair_state_invalid_json_is_invalid_and_preserved() {
    let raw_json = "{not valid json".to_string();
    let status = RepairStateStatus::from_sidecar_json(Some(raw_json.clone()));

    assert!(matches!(
        status,
        RepairStateStatus::Invalid(InvalidRepairState {
            error_category: RepairStateLoadError::InvalidJson,
            ..
        })
    ));
    assert!(status.repair_records().is_empty());
    assert_eq!(status.to_sidecar_json(), Some(raw_json));
}

#[test]
fn repair_state_missing_or_unsupported_version_is_unsupported_and_preserved() {
    for raw_json in [
        r#"{"records":[]}"#.to_string(),
        r#"{"version":2,"records":[]}"#.to_string(),
    ] {
        let status = RepairStateStatus::from_sidecar_json(Some(raw_json.clone()));

        assert!(matches!(
            status,
            RepairStateStatus::Invalid(InvalidRepairState {
                error_category: RepairStateLoadError::UnsupportedVersion,
                ..
            })
        ));
        assert!(status.repair_records().is_empty());
        assert_eq!(status.to_sidecar_json(), Some(raw_json));
    }
}

#[test]
fn repair_source_maps_to_stable_placeholder_reasons() {
    assert_eq!(
        RepairSource::ForkedHistory.placeholder_reason(),
        "forked_history_repair"
    );
    assert_eq!(
        RepairSource::RestoredLegacyHistory.placeholder_reason(),
        "restored_legacy_history_repair"
    );
}

#[test]
fn diagnostics_for_observed_gap_use_categories_and_redacted_metadata() {
    let state = ReadinessState::MissingResultWithoutRepairSource {
        tool_calls: vec![ToolCallRef::new(
            key("task-1", "assistant-1", "call-1"),
            kind("local_interception:websearch"),
        )],
        reason: MissingResultReason::NoResult,
    };

    assert_eq!(
        state.category(),
        ReadinessCategory::MissingResultWithoutRepairSource
    );
    let targets = readiness_state_targets(&state);
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].task_id, "task-1");
    assert_eq!(targets[0].assistant_tool_call_message_id, "assistant-1");
    assert_eq!(targets[0].tool_call_id, "call-1");
    assert_eq!(
        targets[0].redacted_tool_kind,
        "local_interception:websearch"
    );

    let rendered = format!("{:?} {:?}", state.category(), targets[0]);
    assert!(!rendered.contains("secret user prompt"));
    assert!(!rendered.contains("raw tool arguments"));
    assert!(!rendered.contains("raw tool output"));
    assert!(!rendered.contains("raw local interception payload"));

    let context = ReadinessDiagnosticContext::new(
        "conversation-1",
        "attempt-1",
        ReadinessTriggerLayer::SerializerValidation,
    )
    .with_iteration(3);
    let mut coalescer = ReadinessDiagnosticCoalescer::default();
    let entries =
        coalescer.test_log_state_entries(&state, &context, ReadinessDiagnosticLevel::Error);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].level, ReadinessDiagnosticLevel::Error);
    let message = &entries[0].message;
    assert!(message.contains("category=MissingResultWithoutRepairSource"));
    assert!(message.contains("conversation_id=conversation-1"));
    assert!(message.contains("task_id=task-1"));
    assert!(message.contains("assistant_tool_call_message_id=assistant-1"));
    assert!(message.contains("tool_call_id=call-1"));
    assert!(message.contains("redacted_tool_kind=local_interception:websearch"));
    assert!(message.contains("trigger_layer=serializer_validation"));
    assert!(message.contains("request_attempt_id=attempt-1"));
    assert!(message.contains("iteration=3"));
    assert!(!message.contains("secret user prompt"));
    assert!(!message.contains("raw tool arguments"));
    assert!(!message.contains("raw tool output"));
    assert!(!message.contains("raw local interception payload"));
}

#[test]
fn diagnostic_coalescing_uses_request_local_safe_key() {
    let state = ReadinessState::MissingResultWithoutRepairSource {
        tool_calls: vec![call_ref("task-1", "assistant-1", "call-1")],
        reason: MissingResultReason::NoResult,
    };
    let context = ReadinessDiagnosticContext::new(
        "conversation-1",
        "attempt-1",
        ReadinessTriggerLayer::ControllerPreflight,
    )
    .with_iteration(2);
    let mut coalescer = ReadinessDiagnosticCoalescer::default();

    coalescer.log_state(&state, &context, ReadinessDiagnosticLevel::Error);
    coalescer.log_state(&state, &context, ReadinessDiagnosticLevel::Error);

    let (key, suppressed_count) = coalescer
        .suppressed_by_key
        .iter()
        .next()
        .expect("duplicate diagnostic should be coalesced");
    assert_eq!(*suppressed_count, 1);
    assert_eq!(
        key.category,
        ReadinessCategory::MissingResultWithoutRepairSource
    );
    assert_eq!(key.conversation_id, "conversation-1");
    assert_eq!(key.task_id, "task-1");
    assert_eq!(key.assistant_tool_call_message_id, "assistant-1");
    assert_eq!(key.tool_call_id, "call-1");
    assert_eq!(
        key.trigger_layer,
        ReadinessTriggerLayer::ControllerPreflight
    );

    let summary_entries = coalescer.test_finish_entries(&context, ReadinessDiagnosticLevel::Error);
    assert_eq!(summary_entries.len(), 1);
    assert_eq!(summary_entries[0].level, ReadinessDiagnosticLevel::Error);
    let message = &summary_entries[0].message;
    assert!(message.contains("diagnostic coalesced"));
    assert!(message.contains("suppressed_count=1"));
    assert!(message.contains("category=MissingResultWithoutRepairSource"));
    assert!(message.contains("conversation_id=conversation-1"));
    assert!(message.contains("task_id=task-1"));
    assert!(message.contains("assistant_tool_call_message_id=assistant-1"));
    assert!(message.contains("tool_call_id=call-1"));
    assert!(message.contains("redacted_tool_kind=omitted"));
    assert!(message.contains("trigger_layer=controller_preflight"));
    assert!(message.contains("request_attempt_id=attempt-1"));
}

#[test]
fn smoke_blocked_readiness_error_uses_user_facing_copy() {
    let report = classify(vec![assistant_with_one_call()]);
    assert!(
        matches!(
            report.state,
            ReadinessState::MissingResultWithoutRepairSource { .. }
        ),
        "missing tool result should not be Ready: {:?}",
        report.state
    );

    let error = BlockedByopReadinessError::new(report.state.category());
    assert_eq!(
        error.to_string(),
        BLOCKED_BYOP_REQUEST_MESSAGE,
        "blocked readiness must surface the stable user-facing copy"
    );
}
