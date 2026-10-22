use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::transport::stdio;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};

use crate::BridgeError;
use crate::call::ToolCall;
use crate::catalog::Catalog;
use crate::link::Link;

const INSTRUCTIONS: &str = "Synt holds the person's servers, Kubernetes clusters and databases. \
Every tool runs through the Synt core: anything that changes a host may wait until the person \
approves it in the Synt app, and every call is recorded in the Synt journal. Targets are \
resources written as @project/name, or hosts from ~/.ssh/config.";

pub struct Bridge {
    link: Link,
}

impl Bridge {
    pub fn new(link: Link) -> Self {
        Self { link }
    }

    pub async fn serve_stdio(self) -> Result<(), BridgeError> {
        let session = self.serve(stdio()).await.map_err(Box::new)?;
        session.waiting().await?;

        Ok(())
    }

    async fn catalog(&self) -> Result<Catalog, BridgeError> {
        let descriptions = self.link.client()?.describe().await?;

        Ok(Catalog::of(descriptions))
    }
}

impl ServerHandler for Bridge {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("synt", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }

    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        context.peer.set_peer_info(request.clone());
        self.link.open(&request.client_info.name).await?;

        self.negotiate_initialize(&request)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let catalog = self.catalog().await?;

        Ok(ListToolsResult::with_all_items(catalog.into_tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let catalog = self.catalog().await?;

        let Some(action) = catalog.action(&request.name) else {
            return Err(ErrorData::invalid_params(
                format!("there is no tool {}", request.name),
                None,
            ));
        };

        let result = match ToolCall::new(action, request.arguments) {
            Ok(call) => call.run(self.link.client()?).await,
            Err(refusal) => refusal,
        };

        Ok(result.into())
    }
}
