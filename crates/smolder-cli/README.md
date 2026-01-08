# smolder-cli

Command-line interface for Smolder - a contract registry and interaction platform for Foundry projects.

## Installation

```bash
cargo install smolder-cli
```

## Commands

### Initialize

Initialize smolder in a Foundry project:

```bash
smolder init
```

### Deploy

Deploy contracts via forge script and track in the database:

```bash
smolder deploy script/Deploy.s.sol --network mainnet --broadcast
```

### Sync

Sync deployments from Foundry broadcast files:

```bash
smolder sync
```

### List

List all deployments:

```bash
smolder list
smolder list --network mainnet
```

### Get

Get the address of a deployed contract:

```bash
smolder get MyContract --network mainnet
```

### Export

Export deployment addresses:

```bash
smolder export --format json
smolder export --format ts --output src/addresses.ts
smolder export --format env --output .env.contracts
```

### Serve

Start the web dashboard:

```bash
smolder serve
smolder serve --host 0.0.0.0 --port 8080
```

## License

MIT
