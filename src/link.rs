use synt_client::{Autostart, Client};
use synt_proto::{ClientKind, Hello};
use synt_transport::Endpoint;
use tokio::sync::OnceCell;

use crate::BridgeError;

const SOFTWARE: &str = concat!("synt-mcp ", env!("CARGO_PKG_VERSION"));
const UNNAMED_AGENT: &str = "agent";

pub struct Link {
    endpoint: Endpoint,
    autostart: Autostart,
    client: OnceCell<Client>,
}

impl Link {
    pub fn new(endpoint: Endpoint, autostart: Autostart) -> Self {
        Self {
            endpoint,
            autostart,
            client: OnceCell::new(),
        }
    }

    pub async fn open(&self, agent: &str) -> Result<&Client, BridgeError> {
        let hello = Hello::new(Self::agent(agent), SOFTWARE);

        let client = self
            .client
            .get_or_try_init(|| Client::connect(&self.endpoint, hello, &self.autostart))
            .await?;

        Ok(client)
    }

    pub fn client(&self) -> Result<&Client, BridgeError> {
        self.client.get().ok_or(BridgeError::NotInitialized)
    }

    fn agent(name: &str) -> ClientKind {
        let name = name.split_whitespace().collect::<Vec<_>>().join(" ");

        ClientKind::Agent {
            name: if name.is_empty() {
                UNNAMED_AGENT.to_owned()
            } else {
                name
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use synt_proto::ClientKind;

    use super::Link;

    fn named(kind: ClientKind) -> String {
        match kind {
            ClientKind::Agent { name } => name,
            ClientKind::App => panic!("an agent was expected"),
        }
    }

    #[test]
    fn the_agent_keeps_the_name_its_client_gives() {
        assert_eq!(named(Link::agent("claude-code")), "claude-code");
    }

    #[test]
    fn line_breaks_and_blank_names_never_reach_the_core() {
        assert_eq!(named(Link::agent("evil\nname ")), "evil name");
        assert_eq!(named(Link::agent("  ")), "agent");
    }
}
