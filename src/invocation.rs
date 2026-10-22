use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use synt_client::Autostart;
use synt_platform::Endpoint;

use crate::BridgeError;
use crate::link::Link;

const WEAK_KEYCHAIN: &str = "--allow-login-keychain";

pub enum Invocation {
    Serve(Link),
    Help,
    Version,
}

impl Invocation {
    pub const USAGE: &str = "\
synt-mcp: the Synt core's actions as MCP tools, over stdio

Usage: synt-mcp [--endpoint <socket>] [--core <path>] [--no-autostart] [--allow-login-keychain]

  --endpoint <socket>      the core's socket, if not the default for this user
  --core <path>            the synt-core program to start when none is running
  --no-autostart           fail instead of starting the core
  --allow-login-keychain   start an unsigned macOS core with its key in the login keychain
  --help, --version";

    pub fn from_env() -> Result<Self, BridgeError> {
        Self::parse(env::args_os().skip(1))
    }

    fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, BridgeError> {
        let mut arguments = arguments.into_iter();
        let mut endpoint = None;
        let mut autostart = Autostart::core();

        while let Some(argument) = arguments.next() {
            match argument.to_str() {
                Some("--help" | "-h") => return Ok(Self::Help),
                Some("--version" | "-V") => return Ok(Self::Version),
                Some("--no-autostart") => autostart = Autostart::off(),
                Some(WEAK_KEYCHAIN) => autostart = autostart.arguments([WEAK_KEYCHAIN]),
                Some("--endpoint") => {
                    endpoint = Some(Endpoint::new(Self::value(&mut arguments, "--endpoint")?));
                }
                Some("--core") => {
                    autostart =
                        autostart.program(PathBuf::from(Self::value(&mut arguments, "--core")?));
                }
                _ => {
                    return Err(BridgeError::UnknownArgument(
                        argument.to_string_lossy().into_owned(),
                    ));
                }
            }
        }

        let endpoint = match endpoint {
            Some(endpoint) => endpoint,
            None => Endpoint::for_user().map_err(BridgeError::Endpoint)?,
        };

        Ok(Self::Serve(Link::new(endpoint, autostart)))
    }

    fn value(
        arguments: &mut impl Iterator<Item = OsString>,
        flag: &'static str,
    ) -> Result<OsString, BridgeError> {
        arguments.next().ok_or(BridgeError::MissingValue { flag })
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::Invocation;
    use crate::BridgeError;

    fn parse(arguments: &[&str]) -> Result<Invocation, BridgeError> {
        Invocation::parse(arguments.iter().map(OsString::from))
    }

    #[test]
    fn help_and_version_win_over_everything_else() {
        assert!(matches!(
            parse(&["--endpoint", "/tmp/s", "--help"]),
            Ok(Invocation::Help)
        ));
        assert!(matches!(parse(&["-V"]), Ok(Invocation::Version)));
    }

    #[test]
    fn an_explicit_endpoint_and_core_are_served() {
        let served = parse(&["--endpoint", "/tmp/synt.sock", "--core", "/opt/synt-core"]);

        assert!(matches!(served, Ok(Invocation::Serve(_))));
    }

    #[test]
    fn a_flag_without_its_value_is_refused() {
        assert!(matches!(
            parse(&["--endpoint"]),
            Err(BridgeError::MissingValue { flag: "--endpoint" })
        ));
    }

    #[test]
    fn unknown_arguments_are_refused_by_name() {
        let refused = parse(&["--verbose"]);

        assert!(matches!(refused, Err(BridgeError::UnknownArgument(name)) if name == "--verbose"));
    }
}
