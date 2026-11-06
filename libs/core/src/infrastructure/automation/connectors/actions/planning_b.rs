use serde_json::Value;

use super::ActionContext;

pub(super) async fn run(_kind: &str, _c: &ActionContext<'_>) -> Option<Result<Value, String>> {
    None
}
