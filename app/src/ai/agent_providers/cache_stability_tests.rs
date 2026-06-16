//! Prompt cache 序列化稳定性测试集合(对应文档 P1-8 / P1-9 / P1-13)。
//!
//! Anthropic 文档明确警告:
//! > Verify that the keys in your `tool_use` content blocks have stable
//! > ordering as some languages (for example, Swift, Go) randomize key order
//! > during JSON conversion, breaking caches
//!
//! 这意味着任何 `serde_json::Value` 在 Rust 端的产出**必须**:
//!   1. 同输入跨调用 byte-equal(确定性)
//!   2. 不依赖 `HashMap` iterate 顺序
//!   3. 不依赖外部状态(时间戳、随机、PID 等)
//!
//! 这套测试是 Zap 的"防退化护栏"——后续任何修改 prompt
//! 构造路径的改动只要破坏字节级稳定性,这里就会断言失败。

use crate::ai::agent::{MCPContext, MCPServer};
use api::message;
use warp_multi_agent_api as api;

use super::chat_stream;
use super::tools;

// ---------------------------------------------------------------------------
// P1-8: tool schema 字段顺序稳定性
// ---------------------------------------------------------------------------

/// 对 `REGISTRY` 中每个 tool 调 `(parameters)()` 两次,断言 byte-equal。
///
/// 风险点:tool schema 内嵌的 enum / oneof 内部如果用 `HashMap<String, Schema>`
/// 转 Value,顺序就乱。`json!({...})` 字面量产出的 `serde_json::Map` 默认按
/// **insertion order** 保留(`preserve_order` 默认开,见 Cargo.toml),所以
/// 字面写死的 key 顺序跨调用稳定。这条测试守护这个不变量。
#[test]
fn registry_tool_schemas_are_deterministic() {
    for tool in tools::REGISTRY {
        let s1 = (tool.parameters)();
        let s2 = (tool.parameters)();
        let j1 = serde_json::to_string(&s1).unwrap();
        let j2 = serde_json::to_string(&s2).unwrap();
        assert_eq!(
            j1, j2,
            "tool `{}` 的 schema 跨调用 byte-equal 必须成立(prompt cache 命中前置)",
            tool.name
        );
    }
}

/// 对 `REGISTRY` 中每个 tool 反复调 50 次,断言所有调用产出 byte-equal。
/// 防止偶发的 HashMap iterate 顺序漂移(只跑 2 次可能恰巧相同)。
#[test]
fn registry_tool_schemas_stable_under_repetition() {
    for tool in tools::REGISTRY {
        let baseline = serde_json::to_string(&(tool.parameters)()).unwrap();
        for i in 0..50 {
            let candidate = serde_json::to_string(&(tool.parameters)()).unwrap();
            assert_eq!(
                baseline, candidate,
                "tool `{}` 第 {i} 次调用产出与基准不一致(可能存在 HashMap 顺序漂移)",
                tool.name
            );
        }
    }
}

/// `tools::REGISTRY` 自身顺序静态,但仍验证一遍:
/// 同一进程内 iterate 多次得到相同的 (name, description) 序列。
#[test]
fn registry_iteration_order_is_stable() {
    let names1: Vec<&str> = tools::REGISTRY.iter().map(|t| t.name).collect();
    let names2: Vec<&str> = tools::REGISTRY.iter().map(|t| t.name).collect();
    assert_eq!(names1, names2);
}

// ---------------------------------------------------------------------------
// P1-9: serialize_outgoing_tool_call 历史回放稳定性
// ---------------------------------------------------------------------------

/// 模拟一个 Grep 工具调用,验证两次序列化产出 byte-equal。
/// `serialize_outgoing_tool_call` 在每次 build_chat_request 都会重跑,
/// 把历史轮的 ToolCall 转成 (name, args Value)。任何 HashMap / 时间相关的
/// 不稳定就会让 messages 段后半段缓存失效。
///
/// 选 Grep 是因为它字段最简单(`queries: Vec<String>`, `path: String`),
/// 不依赖任何 prost 隐式默认字段。
#[test]
fn serialize_grep_tool_call_is_deterministic() {
    let tc = message::ToolCall {
        tool_call_id: "call-grep-1".to_owned(),
        tool: Some(message::tool_call::Tool::Grep(message::tool_call::Grep {
            queries: vec!["fn main".to_owned(), "Result<".to_owned()],
            path: "src/".to_owned(),
        })),
    };

    let (n1, v1) = chat_stream::serialize_outgoing_tool_call_for_test(&tc, None, "");
    let (n2, v2) = chat_stream::serialize_outgoing_tool_call_for_test(&tc, None, "");
    assert_eq!(n1, n2, "tool name 必须一致");
    let j1 = serde_json::to_string(&v1).unwrap();
    let j2 = serde_json::to_string(&v2).unwrap();
    assert_eq!(j1, j2, "同一 ToolCall 跨序列化 byte-equal");
}

/// Grep `queries` 是 `Vec<String>`,顺序需稳定(Vec 天然稳定,但作为防御断言)。
/// 这反映了一个更大的规则:任何用户 ToolCall 内的 Vec 字段都必须保留入参顺序。
#[test]
fn serialize_grep_preserves_queries_order() {
    let tc = message::ToolCall {
        tool_call_id: "call-grep-2".to_owned(),
        tool: Some(message::tool_call::Tool::Grep(message::tool_call::Grep {
            queries: vec!["zzz".to_owned(), "aaa".to_owned()],
            path: ".".to_owned(),
        })),
    };
    let (_, v) = chat_stream::serialize_outgoing_tool_call_for_test(&tc, None, "");
    let s = serde_json::to_string(&v).unwrap();
    let pos_z = s.find("zzz").expect("queries 应含 zzz");
    let pos_a = s.find("aaa").expect("queries 应含 aaa");
    assert!(pos_z < pos_a, "Vec 顺序必须按入参保留(zzz 先,aaa 后)");
}

/// MCP tool call 含 `prost_types::Struct`,验证序列化稳定。
/// `prost_types::Struct.fields` 内部用 `BTreeMap`,本身就稳定,这里覆盖一下确认。
#[test]
fn serialize_mcp_tool_call_is_deterministic() {
    use prost_types::{value::Kind, Struct, Value as ProstValue};
    use std::collections::BTreeMap;

    let mut fields = BTreeMap::new();
    fields.insert(
        "key_z".to_owned(),
        ProstValue {
            kind: Some(Kind::StringValue("v_z".to_owned())),
        },
    );
    fields.insert(
        "key_a".to_owned(),
        ProstValue {
            kind: Some(Kind::NumberValue(42.0)),
        },
    );

    let server_id = "srv-uuid-1".to_owned();
    let tc = message::ToolCall {
        tool_call_id: "call-mcp-1".to_owned(),
        tool: Some(message::tool_call::Tool::CallMcpTool(
            message::tool_call::CallMcpTool {
                name: "echo".to_owned(),
                args: Some(Struct { fields }),
                server_id: server_id.clone(),
            },
        )),
    };

    // 构造一个 mcp_context 让 sanitize_server_name 能查到 server name
    let ctx = MCPContext {
        #[allow(deprecated)]
        resources: vec![],
        #[allow(deprecated)]
        tools: vec![],
        servers: vec![MCPServer {
            id: server_id.clone(),
            name: "my-server".to_owned(),
            description: String::new(),
            resources: vec![],
            tools: vec![],
        }],
    };

    let (n1, v1) = chat_stream::serialize_outgoing_tool_call_for_test(&tc, Some(&ctx), "");
    let (n2, v2) = chat_stream::serialize_outgoing_tool_call_for_test(&tc, Some(&ctx), "");
    assert_eq!(n1, n2);
    let j1 = serde_json::to_string(&v1).unwrap();
    let j2 = serde_json::to_string(&v2).unwrap();
    assert_eq!(j1, j2);
    // BTreeMap 应该按 key 字典序输出(key_a 在 key_z 前)
    let pos_a = j1.find("key_a").expect("应含 key_a");
    let pos_z = j1.find("key_z").expect("应含 key_z");
    assert!(
        pos_a < pos_z,
        "prost_types::Struct 应按 BTreeMap key 字典序"
    );
}

// ---------------------------------------------------------------------------
// Issue #245: carrier with invalid JSON args must fall back to empty object
// ---------------------------------------------------------------------------

/// When the model emits a tool call with invalid JSON escape sequences (e.g. `\e`, `\``),
/// the carrier message stores the raw string in server_message_data.
/// On the next turn, serialize_outgoing_tool_call must return a Value::Object (not
/// Value::String) so that genai serializes it as a JSON object for `arguments`,
/// not a doubly-wrapped JSON string that the provider would reject with "Invalid \escape".
#[test]
fn carrier_with_invalid_json_args_falls_back_to_empty_object() {
    use api::message;
    // Simulate a carrier message: tool = None, server_message_data = "fn_name\n<invalid_json>"
    let tc = message::ToolCall {
        tool_call_id: "call-invalid".to_owned(),
        tool: None,
    };
    let server_message_data = "shell\n{\"command\": \"echo \\epath\"}";

    let (fn_name, args_value) =
        chat_stream::serialize_outgoing_tool_call_for_test(&tc, None, server_message_data);

    assert_eq!(fn_name, "shell");
    // Must NOT be a String (which would cause double-wrapping and Invalid \escape on the wire)
    assert!(
        !args_value.is_string(),
        "args must be a JSON object/value, not a raw string (would cause Invalid \\escape)"
    );
    // Must be a valid JSON value that serde_json can serialize
    let serialized = serde_json::to_string(&args_value).expect("args_value must be serializable");
    // The serialized form must itself be valid JSON
    serde_json::from_str::<serde_json::Value>(&serialized)
        .expect("serialized args must be valid JSON");
}

// ---------------------------------------------------------------------------
// P1-13: build_tools_array 整体稳定性(配合 P0-3 的 MCP 排序)
// ---------------------------------------------------------------------------

/// 端到端断言:同一 `(REGISTRY + 同 mcp_context)` 跑两次 tools 数组拼接,
/// 字符串 byte-equal。这覆盖了 prompt 中 tools 数组的关键稳定性约束
/// (Anthropic 文档:tool definitions 改动 → 全部 cache 失效)。
///
/// 不直接调 `build_tools_array(params: &RequestParams)` 是因为 `RequestParams`
/// 字段太多构造门槛高;这里复刻它对 REGISTRY 与 mcp 部分的核心拼接逻辑。
#[test]
fn full_tools_array_serialization_is_stable() {
    let assemble = || -> String {
        let mut buf = String::new();
        // 内置 tools(REGISTRY iterate 顺序静态)
        for t in tools::REGISTRY {
            buf.push_str(t.name);
            buf.push('|');
            buf.push_str(t.description);
            buf.push('|');
            let schema = (t.parameters)();
            buf.push_str(&serde_json::to_string(&schema).unwrap());
            buf.push('\n');
        }
        // MCP tools(已在 build_mcp_tool_defs 内排序,无 ctx 时为空)
        buf
    };
    let a = assemble();
    let b = assemble();
    assert_eq!(a.len(), b.len());
    assert_eq!(a, b, "tools array 序列化结果跨调用必须 byte-equal");
}

/// 带 MCP server 的端到端拼接稳定性(对接 P0-3 排序保证)。
#[test]
fn full_tools_array_with_mcp_is_stable() {
    use rmcp::model::{AnnotateAble, RawResource, Tool as McpTool};
    use serde_json::json;
    use std::sync::Arc;

    let schema_obj = json!({
        "type": "object",
        "properties": { "x": { "type": "string" } }
    })
    .as_object()
    .unwrap()
    .clone();

    let server_a = MCPServer {
        id: "id-a".to_owned(),
        name: "server-a".to_owned(),
        description: String::new(),
        resources: vec![RawResource::new("file:///x.txt", "X").no_annotation()],
        tools: vec![
            McpTool::new("zeta", "Z desc", Arc::new(schema_obj.clone())),
            McpTool::new("alpha", "A desc", Arc::new(schema_obj.clone())),
        ],
    };
    let ctx1 = MCPContext {
        #[allow(deprecated)]
        resources: vec![],
        #[allow(deprecated)]
        tools: vec![],
        servers: vec![server_a.clone()],
    };
    // 同 ctx 重新构造一次(servers Vec 顺序相同):
    let ctx2 = MCPContext {
        #[allow(deprecated)]
        resources: vec![],
        #[allow(deprecated)]
        tools: vec![],
        servers: vec![server_a],
    };

    let assemble = |ctx: &MCPContext| -> String {
        let mut buf = String::new();
        for t in tools::REGISTRY {
            buf.push_str(t.name);
            buf.push('|');
            buf.push_str(t.description);
            buf.push('|');
            let schema = (t.parameters)();
            buf.push_str(&serde_json::to_string(&schema).unwrap());
            buf.push('\n');
        }
        for (name, desc, schema) in tools::mcp::build_mcp_tool_defs(ctx) {
            buf.push_str(&name);
            buf.push('|');
            buf.push_str(&desc);
            buf.push('|');
            buf.push_str(&serde_json::to_string(&schema).unwrap());
            buf.push('\n');
        }
        buf
    };

    let a = assemble(&ctx1);
    let b = assemble(&ctx2);
    assert_eq!(a, b, "含 MCP 的 tools array 跨调用必须 byte-equal");
    // 验证 MCP tools 按 function_name 字典序(alpha 在 zeta 前)
    let pos_alpha = a.find("mcp__server-a__alpha").expect("应含 alpha");
    let pos_zeta = a.find("mcp__server-a__zeta").expect("应含 zeta");
    assert!(pos_alpha < pos_zeta, "P0-3 排序保证 alpha < zeta");
}
