# Synt MCP

Lets AI agents use [Synt](https://github.com/syntso/synt) over the [Model Context Protocol](https://modelcontextprotocol.io).

Claude Code, Claude Desktop, Cursor, Codex, VS Code and any other MCP client get the actions of the Synt core as tools: commands on your hosts, files, Kubernetes, databases, runbooks. The agent goes through the same core as you, so it can never do more than you allow: anything that changes a host waits for your approval in Synt, and every call is recorded in the journal.

<br />

## Install

You need [Rust](https://rustup.rs). Install the core and the bridge into the same folder, so the bridge can start the core when it is not running:

```bash
cargo install --locked --git https://github.com/syntso/synt synt-core
cargo install --locked --git https://github.com/syntso/mcp
```

Both land in `~/.cargo/bin`. Check:

```bash
synt-mcp --version
```

On macOS, a core you build yourself is not signed, so it cannot keep its journal key in the protected keychain and refuses to start. Until you have the signed Synt app, let it use the login keychain, which other programs of your user can read, by adding `--allow-login-keychain` to `synt-mcp` in your client's config.

<br />

## Connect your client

### Claude Code

```bash
claude mcp add synt -- synt-mcp
```

On macOS with a self-built core:

```bash
claude mcp add synt -- synt-mcp --allow-login-keychain
```

### Claude Desktop

Settings → Developer → Edit Config, then add:

```json
{
  "mcpServers": {
    "synt": {
      "command": "/Users/you/.cargo/bin/synt-mcp"
    }
  }
}
```

Claude Desktop does not read your shell's `PATH`, so give the full path.

### Cursor

`~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "synt": {
      "command": "synt-mcp"
    }
  }
}
```

### Codex

`~/.codex/config.toml`:

```toml
[mcp_servers.synt]
command = "synt-mcp"
```

### VS Code

`.vscode/mcp.json`:

```json
{
  "servers": {
    "synt": {
      "type": "stdio",
      "command": "synt-mcp"
    }
  }
}
```

<br />

## How it works

- Every action of the core becomes a tool, with its name spelled with underscores: `ssh.exec` is `ssh_exec`, `k8s.logs` is `k8s_logs`. The list comes from the core each time, so actions added by extensions show up too.
- Actions that only a person may take, such as importing keys or answering approvals, are never offered.
- Actions on a host take a `target`: a resource as `@project/name`, or a host from `~/.ssh/config`.
- When an action needs your approval, the call waits until you answer in Synt, for as long as Synt waits for an answer (two minutes unless you change it). A denial comes back to the agent as a tool error it can read.
- The agent introduces itself with its client's name, such as `claude-code`, and that name is what you see in approvals and in the journal.

<br />

## Options

```text
synt-mcp [--endpoint <socket>] [--core <path>] [--no-autostart] [--allow-login-keychain]
```

| Option | |
|---|---|
| `--endpoint <socket>` | Connect to a core on another socket than the default for your user |
| `--core <path>` | The `synt-core` to start when none is running, instead of the one next to `synt-mcp` |
| `--no-autostart` | Fail when no core is running instead of starting one |
| `--allow-login-keychain` | Start an unsigned macOS core with its journal key in the login keychain |

Pass options after the command in your client's config, for example `"args": ["--allow-login-keychain"]`.

<br />

## Build

```bash
git clone https://github.com/syntso/mcp.git
cd mcp
git config core.hooksPath .githooks
cargo test
```

<br />

## License

[FSL-1.1-ALv2](LICENSE.md). Each version becomes Apache 2.0 two years after its release.
