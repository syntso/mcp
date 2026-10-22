use std::io;

use rmcp::ErrorData;
use rmcp::service::ServerInitializeError;
use synt_client::ClientError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("{0}")]
    Core(#[from] ClientError),

    #[error("cannot find the core's socket: {0}")]
    Endpoint(#[source] io::Error),

    #[error("unknown argument {0}; see synt-mcp --help")]
    UnknownArgument(String),

    #[error("{flag} needs a value")]
    MissingValue { flag: &'static str },

    #[error("the MCP client called a tool before it initialized the session")]
    NotInitialized,

    #[error("the MCP session did not start: {0}")]
    Start(#[from] Box<ServerInitializeError>),

    #[error("the MCP session ended abruptly: {0}")]
    Session(#[from] JoinError),
}

impl From<BridgeError> for ErrorData {
    fn from(error: BridgeError) -> Self {
        Self::internal_error(error.to_string(), None)
    }
}
