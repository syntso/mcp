mod bridge;
mod call;
mod catalog;
mod error;
mod invocation;
mod link;

use std::process::ExitCode;

use bridge::Bridge;
use error::BridgeError;
use invocation::Invocation;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("synt-mcp: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), BridgeError> {
    match Invocation::from_env()? {
        Invocation::Serve(link) => Bridge::new(link).serve_stdio().await,
        Invocation::Help => {
            println!("{}", Invocation::USAGE);
            Ok(())
        }
        Invocation::Version => {
            println!("synt-mcp {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
