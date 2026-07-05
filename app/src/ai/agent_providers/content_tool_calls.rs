//! Fallback: parse tool-call-shaped JSON embedded in assistant `content` text.
//!
//! Local models (Ollama native) often emit pseudo-tool JSON as prose instead of
//! structured `message.tool_calls`. When the BYOP stream ends with `tool_bufs`
//! empty, `chat_stream` calls [`extract_tool_calls_from_assistant_text`].

use std::collections::HashSet;

use genai::chat::ToolCall;
use serde_json::Value;
use uuid::Uuid;

/// Extract executable tool calls from assistant text when the provider did not
/// emit native `tool_calls` stream events.
pub fn extract_tool_calls_from_assistant_text(
    text: &str,
    available_tool_names: &[String],
) -> Vec<ToolCall> {
    let allowed: HashSet<&str> = available_tool_names.iter().map(String::as_str).collect();
    let mut out = Vec::new();
    for candidate in collect_json_candidates(text) {
        if let Some(call) = parse_tool_call_value(&candidate, &allowed) {
            out.push(call);
            break;
        }
    }
    out
}

fn collect_json_candidates(text: &str) -> Vec<Value> {
    let mut candidates = Vec::new();
    for block in extract_fenced_json_blocks(text) {
        if let Ok(v) = serde_json::from_str::<Value>(&block) {
            candidates.push(v);
        }
    }
    for obj in extract_balanced_json_objects(text) {
        if let Ok(v) = serde_json::from_str::<Value>(&obj) {
            candidates.push(v);
        }
    }
    candidates
}

fn extract_fenced_json_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("```") {
        let after_fence = &rest[start + 3..];
        let lang_end = after_fence.find('\n').unwrap_or(0);
        let body_start = &after_fence[lang_end..];
        let body_start = body_start.strip_prefix('\n').unwrap_or(body_start);
        if let Some(end) = body_start.find("```") {
            let body = body_start[..end].trim();
            if !body.is_empty() {
                blocks.push(body.to_owned());
            }
            rest = &body_start[end + 3..];
        } else {
            break;
        }
    }
    blocks
}

fn extract_balanced_json_objects(text: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(end) = find_json_object_end(text, i) {
                objects.push(text[i..=end].to_owned());
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    objects
}

fn find_json_object_end(text: &str, start: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (idx, ch) in text[start..].char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_tool_call_value(value: &Value, allowed: &HashSet<&str>) -> Option<ToolCall> {
    let (raw_name, args) = extract_name_and_args(value)?;
    let (name, args) = normalize_tool_call(raw_name, args)?;
    if !allowed.contains(name.as_str()) {
        return None;
    }
    Some(ToolCall {
        call_id: Uuid::new_v4().to_string(),
        fn_name: name,
        fn_arguments: args,
        thought_signatures: None,
    })
}

fn extract_name_and_args(value: &Value) -> Option<(String, Value)> {
    let obj = value.as_object()?;
    if obj.get("type").and_then(Value::as_str) == Some("function") {
        let name = obj.get("name")?.as_str()?.to_owned();
        let args = obj
            .get("parameters")
            .or_else(|| obj.get("arguments"))
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        return Some((name, args));
    }
    let name = obj.get("name")?.as_str()?.to_owned();
    let args = obj
        .get("arguments")
        .or_else(|| obj.get("parameters"))
        .cloned()
        .unwrap_or(Value::Object(Default::default()));
    Some((name, args))
}

fn normalize_tool_call(raw_name: String, args: Value) -> Option<(String, Value)> {
    match raw_name.as_str() {
        "echo" => {
            let command = echo_args_to_command(&args)?;
            Some(("run_shell_command".to_owned(), shell_command_args(command)))
        }
        "write_to_long_running_shell_command" => {
            if args.get("command_id").is_some() {
                return Some((raw_name, args));
            }
            if let Some(command) = args.get("command").and_then(Value::as_str) {
                return Some((
                    "run_shell_command".to_owned(),
                    shell_command_args(command.to_owned()),
                ));
            }
            None
        }
        name if super::tools::lookup(name).is_some() => Some((raw_name, args)),
        _ => None,
    }
}

fn echo_args_to_command(args: &Value) -> Option<String> {
    if let Some(command) = args.get("command").and_then(Value::as_str) {
        return Some(command.to_owned());
    }
    if let Some(message) = args.get("message").and_then(Value::as_str) {
        return Some(format!("echo {message}"));
    }
    if let Some(text) = args.as_str() {
        return Some(format!("echo {text}"));
    }
    None
}

fn shell_command_args(command: String) -> Value {
    serde_json::json!({ "command": command })
}

#[cfg(test)]
#[path = "content_tool_calls_tests.rs"]
mod tests;
