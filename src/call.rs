use rmcp::model::{CallToolResult, ContentBlock, JsonObject};
use serde_json::Value;
use synt_client::{Client, ClientError};
use synt_proto::{ActionCall, ActionDescription, ErrorCode};

const TARGET: &str = "target";

pub struct ToolCall {
    action: ActionCall,
}

impl ToolCall {
    pub fn new(
        description: &ActionDescription,
        arguments: Option<JsonObject>,
    ) -> Result<Self, CallToolResult> {
        let mut params = arguments.unwrap_or_default();
        let mut action = ActionCall::new(description.id.as_str());

        if description.takes_target {
            let Some(Value::String(target)) = params.remove(TARGET) else {
                return Err(Self::refusal(
                    "This tool needs a target: a resource as @project/name, or a host from ~/.ssh/config.",
                ));
            };
            action = action.target(target);
        }

        Ok(Self {
            action: action.params(Value::Object(params)),
        })
    }

    pub async fn run(self, client: &Client) -> CallToolResult {
        match client.call(self.action).await {
            Ok(outcome) => Self::result(outcome.value),
            Err(error) => Self::failure(&error),
        }
    }

    fn result(value: Value) -> CallToolResult {
        match value {
            Value::Object(_) => CallToolResult::structured(value),
            Value::String(text) => CallToolResult::success(vec![ContentBlock::text(text)]),
            other => CallToolResult::success(vec![ContentBlock::text(other.to_string())]),
        }
    }

    fn failure(error: &ClientError) -> CallToolResult {
        let ClientError::Rpc(refused) = error else {
            return Self::refusal(&format!("Synt could not run this: {error}"));
        };

        match refused.code {
            ErrorCode::DENIED => Self::refusal(&format!(
                "The person did not allow this in Synt: {}",
                refused.message
            )),
            _ => Self::refusal(&refused.message),
        }
    }

    fn refusal(message: &str) -> CallToolResult {
        CallToolResult::error(vec![ContentBlock::text(message)])
    }
}

#[cfg(test)]
mod tests {
    use rmcp::model::{CallToolResult, JsonObject};
    use serde_json::{Value, json};
    use synt_client::ClientError;
    use synt_proto::{ActionDescription, ErrorCode, RpcError};

    use super::ToolCall;

    fn action(takes_target: bool) -> ActionDescription {
        serde_json::from_value(json!({
            "id": "ssh.exec",
            "title": "Run a command",
            "risk": "write",
            "person_only": false,
            "takes_target": takes_target,
            "params": {}
        }))
        .unwrap()
    }

    fn arguments(value: Value) -> Option<JsonObject> {
        match value {
            Value::Object(arguments) => Some(arguments),
            _ => None,
        }
    }

    fn text(result: &CallToolResult) -> String {
        serde_json::to_value(result).unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    #[test]
    fn the_target_leaves_the_params_for_the_call() {
        let call = ToolCall::new(
            &action(true),
            arguments(json!({ "target": "@shop/api-1", "command": "uptime" })),
        )
        .unwrap();

        assert_eq!(call.action.action, "ssh.exec");
        assert_eq!(call.action.target.as_deref(), Some("@shop/api-1"));
        assert_eq!(call.action.params, json!({ "command": "uptime" }));
    }

    #[test]
    fn a_missing_target_is_told_to_the_agent() {
        let refused = ToolCall::new(&action(true), None).err().unwrap();

        assert_eq!(refused.is_error, Some(true));
        assert!(text(&refused).contains("needs a target"));
    }

    #[test]
    fn objects_come_back_structured_and_text_as_text() {
        let structured = ToolCall::result(json!({ "exit": 0 }));
        let plain = ToolCall::result(json!("done"));

        assert_eq!(structured.structured_content, Some(json!({ "exit": 0 })));
        assert_eq!(text(&plain), "done");
    }

    #[test]
    fn a_denial_reads_as_the_person_saying_no() {
        let denied = ClientError::Rpc(RpcError::new(ErrorCode::DENIED, "denied by the person"));

        let result = ToolCall::failure(&denied);

        assert_eq!(result.is_error, Some(true));
        assert!(text(&result).starts_with("The person did not allow this in Synt"));
    }
}
