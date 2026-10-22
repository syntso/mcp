use std::sync::Arc;

use rmcp::model::{JsonObject, Tool, ToolAnnotations};
use serde_json::{Value, json};
use synt_proto::{ActionDescription, Risk};

const TARGET: &str = "target";

pub struct Offer<'a> {
    description: &'a ActionDescription,
    name: String,
}

impl<'a> Offer<'a> {
    pub fn new(description: &'a ActionDescription) -> Self {
        let name = description.id.as_str().replace('.', "_");

        Self { description, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tool(&self) -> Tool {
        Tool::new(
            self.name.clone(),
            self.explanation(),
            Arc::new(self.input_schema()),
        )
        .with_title(self.description.title.clone())
        .with_annotations(self.annotations())
    }

    fn explanation(&self) -> String {
        let risk = match self.description.risk {
            Risk::Read => "It only reads.",
            Risk::Write => "It changes things, so the person may be asked to approve it in Synt.",
            Risk::Privileged => "It is privileged, so the person is asked to approve it in Synt.",
        };

        format!("{}. {risk}", self.description.title)
    }

    fn annotations(&self) -> ToolAnnotations {
        ToolAnnotations::with_title(self.description.title.clone())
            .read_only(self.description.risk == Risk::Read)
            .destructive(self.description.risk == Risk::Privileged)
    }

    fn input_schema(&self) -> JsonObject {
        let mut schema = match &self.description.params {
            Value::Object(schema) => schema.clone(),
            _ => JsonObject::new(),
        };

        schema.insert("type".to_owned(), json!("object"));
        let properties = schema.entry("properties").or_insert_with(|| json!({}));

        if self.description.takes_target {
            if let Value::Object(properties) = properties {
                properties.insert(TARGET.to_owned(), Self::target_schema());
            }
            Self::require(&mut schema, TARGET);
        }

        schema
    }

    fn target_schema() -> Value {
        json!({
            "type": "string",
            "description": "What to act on: a resource as @project/name, or a host from ~/.ssh/config"
        })
    }

    fn require(schema: &mut JsonObject, field: &str) {
        let required = schema.entry("required").or_insert_with(|| json!([]));

        if let Value::Array(required) = required {
            required.push(json!(field));
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use synt_proto::ActionDescription;

    use super::Offer;

    fn action(risk: &str, takes_target: bool, params: &Value) -> ActionDescription {
        serde_json::from_value(json!({
            "id": "ssh.exec",
            "title": "Run a command",
            "risk": risk,
            "person_only": false,
            "takes_target": takes_target,
            "params": params
        }))
        .unwrap()
    }

    #[test]
    fn dots_become_underscores_in_the_tool_name() {
        let description = action("read", false, &Value::Null);

        assert_eq!(Offer::new(&description).name(), "ssh_exec");
    }

    #[test]
    fn a_target_joins_the_params_and_is_required() {
        let params = json!({
            "type": "object",
            "properties": { "command": { "type": "string" } },
            "required": ["command"]
        });
        let description = action("write", true, &params);

        let tool = Offer::new(&description).tool();
        let schema = Value::Object((*tool.input_schema).clone());

        assert_eq!(schema["properties"]["command"]["type"], "string");
        assert_eq!(schema["properties"]["target"]["type"], "string");
        assert_eq!(schema["required"], json!(["command", "target"]));
    }

    #[test]
    fn an_action_without_params_still_offers_an_object() {
        let description = action("read", false, &Value::Null);

        let tool = Offer::new(&description).tool();

        assert_eq!(tool.input_schema.get("type"), Some(&json!("object")));
        assert_eq!(tool.input_schema.get("properties"), Some(&json!({})));
    }

    #[test]
    fn the_risk_shapes_the_hints_and_the_explanation() {
        let read = action("read", false, &Value::Null);
        let privileged = action("privileged", false, &Value::Null);

        let read = Offer::new(&read).tool();
        let privileged = Offer::new(&privileged).tool();

        assert_eq!(read.annotations.unwrap().read_only_hint, Some(true));
        assert_eq!(privileged.annotations.unwrap().destructive_hint, Some(true));
        assert!(privileged.description.unwrap().contains("asked to approve"));
    }
}
