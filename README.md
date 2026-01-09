# Smolder

A contract registry and interaction platform for Foundry projects.

Smolder helps you track and manage smart contract deployments across multiple networks. It integrates with Foundry's broadcast system to automatically sync deployment data and provides a web dashboard for exploring your contracts.

## Features

- Track contract deployments across multiple networks
- Sync deployments from Foundry broadcast files
- Export deployment addresses to JSON, TypeScript, or ENV formats
- Web dashboard for exploring contracts and ABIs
- SQLite-based local storage

## Installation

```bash
cargo install smolder-cli
```

## Quick Start

1. Initialize smolder in your Foundry project:

```bash
smolder init
```

2. Deploy contracts using forge script and track them:

```bash
smolder deploy script/Deploy.s.sol --network mainnet --broadcast
```

3. Or sync existing deployments from broadcast files:

```bash
smolder sync
```

4. List all deployments:

```bash
smolder list
```

5. Start the web dashboard:

```bash
smolder serve
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize smolder in a Foundry project |
| `deploy` | Deploy contracts via forge script and track in database |
| `sync` | Sync deployments from broadcast directory |
| `list` | List all deployments |
| `get` | Get the address of a deployed contract |
| `export` | Export deployments to JSON, TypeScript, or ENV format |
| `serve` | Start the web dashboard |

## Crates

- [`smolder-cli`](./crates/smolder-cli) - Command-line interface
- [`smolder-core`](./crates/smolder-core) - Core library with data models and database schema
- [`smolder-server`](./crates/smolder-server) - Web server with embedded React dashboard

## License

MIT
