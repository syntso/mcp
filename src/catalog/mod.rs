mod offer;

use std::collections::HashMap;

use rmcp::model::Tool;
use synt_proto::ActionDescription;

use offer::Offer;

pub struct Catalog {
    tools: Vec<Tool>,
    actions: HashMap<String, ActionDescription>,
}

impl Catalog {
    pub fn of(descriptions: Vec<ActionDescription>) -> Self {
        let mut tools = Vec::new();
        let mut actions = HashMap::new();

        for description in descriptions
            .into_iter()
            .filter(|action| !action.person_only)
        {
            let offer = Offer::new(&description);

            if actions.contains_key(offer.name()) {
                continue;
            }

            tools.push(offer.tool());
            actions.insert(offer.name().to_owned(), description);
        }

        Self { tools, actions }
    }

    pub fn into_tools(self) -> Vec<Tool> {
        self.tools
    }

    pub fn action(&self, tool: &str) -> Option<&ActionDescription> {
        self.actions.get(tool)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use synt_proto::ActionDescription;

    use super::Catalog;

    fn action(id: &str, person_only: bool) -> ActionDescription {
        serde_json::from_value(json!({
            "id": id,
            "title": "Run a command",
            "risk": "write",
            "person_only": person_only,
            "takes_target": true,
            "params": { "type": "object", "properties": {} }
        }))
        .unwrap()
    }

    #[test]
    fn person_only_actions_are_never_offered() {
        let catalog = Catalog::of(vec![action("ssh.exec", false), action("key.import", true)]);

        assert!(catalog.action("ssh_exec").is_some());
        assert!(catalog.action("key_import").is_none());
        assert_eq!(catalog.into_tools().len(), 1);
    }

    #[test]
    fn tools_lead_back_to_their_action() {
        let catalog = Catalog::of(vec![action("k8s.port_forward", false)]);

        let found = catalog.action("k8s_port_forward").unwrap();

        assert_eq!(found.id.as_str(), "k8s.port_forward");
    }
}
