use std::str::FromStr;

use mcp_tools::Tool;
use serde_json::{Value, json};

pub const CODE_PARSE_ERROR: i64 = -32700;
pub const CODE_INVALID_PARAMS: i64 = -32602;
pub const CODE_DENIED: i64 = -32001;
pub const CODE_UNKNOWN_TOOL: i64 = -32002;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub id: Value,
    pub tool: Tool,
    pub organization: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rejection {
    pub id: Value,
    pub code: i64,
    pub message: &'static str,
    pub http_status: u16,
}

impl Rejection {
    pub fn denied(id: Value) -> Self {
        Self {
            id,
            code: CODE_DENIED,
            message: "denied by policy",
            http_status: 200,
        }
    }

    pub fn body(&self) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": self.id,
            "error": { "code": self.code, "message": self.message },
        })
    }
}

pub fn inspect(body: &[u8]) -> Result<Vec<ToolCall>, Rejection> {
    let parsed: Value = serde_json::from_slice(body).map_err(|_| Rejection {
        id: Value::Null,
        code: CODE_PARSE_ERROR,
        message: "parse error",
        http_status: 400,
    })?;
    let messages = match &parsed {
        Value::Array(items) => items.iter().collect::<Vec<_>>(),
        other => vec![other],
    };
    messages.into_iter().filter_map(tool_call).collect()
}

fn tool_call(message: &Value) -> Option<Result<ToolCall, Rejection>> {
    if message.get("method").and_then(Value::as_str) != Some("tools/call") {
        return None;
    }
    let id = message.get("id").cloned().unwrap_or(Value::Null);
    let params = message.get("params");
    let Some(name) = params.and_then(|p| p.get("name")).and_then(Value::as_str) else {
        return Some(Err(Rejection {
            id,
            code: CODE_INVALID_PARAMS,
            message: "invalid params",
            http_status: 200,
        }));
    };
    let Ok(tool) = Tool::from_str(name) else {
        return Some(Err(Rejection {
            id,
            code: CODE_UNKNOWN_TOOL,
            message: "unknown tool",
            http_status: 200,
        }));
    };
    let organization = params
        .and_then(|p| p.get("arguments"))
        .and_then(|a| a.get("organization"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    Some(Ok(ToolCall {
        id,
        tool,
        organization,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_non_tool_method_needs_no_decision() {
        let body = br#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        assert_eq!(inspect(body), Ok(vec![]));
    }

    #[test]
    fn a_known_tool_carries_its_organization() {
        let body = br#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"list_does","arguments":{"organization":"acme"}}}"#;
        let calls = inspect(body).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool, Tool::ListDoes);
        assert_eq!(calls[0].organization.as_deref(), Some("acme"));
        assert_eq!(calls[0].id, json!(7));
    }

    #[test]
    fn an_unknown_tool_is_rejected_with_its_id() {
        let body = br#"{"jsonrpc":"2.0","id":"x","method":"tools/call","params":{"name":"delete_project"}}"#;
        let rejection = inspect(body).unwrap_err();
        assert_eq!(rejection.code, CODE_UNKNOWN_TOOL);
        assert_eq!(rejection.id, json!("x"));
    }

    #[test]
    fn a_batch_with_one_unknown_tool_is_rejected() {
        let body = br#"[{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_doe"}},{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"nope"}}]"#;
        assert_eq!(inspect(body).unwrap_err().code, CODE_UNKNOWN_TOOL);
    }

    #[test]
    fn garbage_is_a_parse_error() {
        let rejection = inspect(b"{").unwrap_err();
        assert_eq!(rejection.code, CODE_PARSE_ERROR);
        assert_eq!(rejection.http_status, 400);
    }

    #[test]
    fn a_missing_name_is_invalid_params() {
        let body = br#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{}}"#;
        assert_eq!(inspect(body).unwrap_err().code, CODE_INVALID_PARAMS);
    }
}
