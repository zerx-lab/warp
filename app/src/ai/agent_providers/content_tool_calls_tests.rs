use super::extract_tool_calls_from_assistant_text;

fn shell_tools() -> Vec<String> {
    vec!["run_shell_command".to_owned()]
}

#[test]
fn extracts_qwen_fenced_echo_json() {
    let text = r#"```json
{"name":"echo","arguments":{"message":"hello"}}
```"#;
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_name, "run_shell_command");
    assert_eq!(calls[0].fn_arguments["command"], "echo hello");
}

#[test]
fn extracts_qwen_multiline_fenced_echo_json() {
    let text = "```json\n{\n  \"name\": \"echo\",\n  \"arguments\": {\n    \"message\": \"hello\"\n  }\n}\n```";
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_name, "run_shell_command");
    assert_eq!(calls[0].fn_arguments["command"], "echo hello");
}

#[test]
fn extracts_llama_wrong_long_running_tool_as_shell() {
    let text = r#"{"type":"function","name":"write_to_long_running_shell_command","parameters":{"command":"echo hello"}}"#;
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_name, "run_shell_command");
    assert_eq!(calls[0].fn_arguments["command"], "echo hello");
}

#[test]
fn gemma_prose_only_returns_empty() {
    let text = "I ran echo hello and the output was hello.";
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert!(calls.is_empty());
}

#[test]
fn unknown_tool_name_is_skipped() {
    let text = r#"{"name":"websearch","arguments":{"query":"test"}}"#;
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert!(calls.is_empty());
}

#[test]
fn extracts_run_code_command_alias() {
    let text =
        r#"{"type":"function","name":"run_code_command","parameters":{"command":"echo hello"}}"#;
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_name, "run_shell_command");
    assert_eq!(calls[0].fn_arguments["command"], "echo hello");
}

#[test]
fn unknown_tool_with_command_field_falls_back_to_shell() {
    let text = r#"{"name":"totally_made_up_tool","parameters":{"command":"echo hi"}}"#;
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_name, "run_shell_command");
}

#[test]
fn get_previous_command_output_is_not_a_shell_tool() {
    let text = r#"{"type":"function","name":"get_previous_command_output","parameters":{}}"#;
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert!(calls.is_empty());
}

#[test]
fn extracts_openai_tool_calls_array_wrapper() {
    let text = r#"{"tool_calls":[{"type":"function","function":{"name":"run_shell_command","arguments":{"command":"echo hello"}}}]}"#;
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_name, "run_shell_command");
    assert_eq!(calls[0].fn_arguments["command"], "echo hello");
}

#[test]
fn prefers_run_shell_over_earlier_webfetch() {
    let text = r#"{"type":"function","name":"webfetch","parameters":{"url":"https://example.com"}}
{"type":"function","name":"run_shell_command","parameters":{"command":"echo hello"}}"#;
    let tools = vec!["run_shell_command".to_owned(), "webfetch".to_owned()];
    let calls = extract_tool_calls_from_assistant_text(text, &tools);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_name, "run_shell_command");
}

#[test]
fn disallowed_websearch_filtered_leaves_echo_as_shell_call() {
    // "websearch" is a real registered tool but isn't in `shell_tools()`'s allow-list,
    // so it's dropped; "echo" normalizes to run_shell_command and survives as the
    // only candidate. Not actually an ordering test (see below for that).
    let text = r#"{"name":"websearch","arguments":{"query":"x"}}
{"name":"echo","arguments":{"message":"hi"}}"#;
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_arguments["command"], "echo hi");
}

#[test]
fn last_valid_json_wins_when_no_shell_call_present() {
    // Real `pick_best_tool_call` policy: prefer run_shell_command; otherwise the
    // *last* valid candidate wins (not the first).
    let text = r#"{"type":"function","name":"webfetch","parameters":{"url":"https://example.com"}}
{"type":"function","name":"websearch","parameters":{"query":"rust async"}}"#;
    let tools = vec!["webfetch".to_owned(), "websearch".to_owned()];
    let calls = extract_tool_calls_from_assistant_text(text, &tools);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_name, "websearch");
    assert_eq!(calls[0].fn_arguments["query"], "rust async");
}
