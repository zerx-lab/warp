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
fn first_valid_json_wins_among_multiple() {
    let text = r#"{"name":"websearch","arguments":{"query":"x"}}
{"name":"echo","arguments":{"message":"hi"}}"#;
    let calls = extract_tool_calls_from_assistant_text(text, &shell_tools());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].fn_arguments["command"], "echo hi");
}
