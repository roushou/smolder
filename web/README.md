# Smolder Web UI

The web dashboard for Smolder - a contract registry and interaction platform for Foundry projects.

## Tech Stack

- **React 19** - UI framework
- **TypeScript** - Type safety
- **TailwindCSS v4** - Styling
- **TanStack Router** - Client-side routing
- **Vite** - Build tool
- **Biome** - Linting and formatting

## Pages

| Route | Page | Description |
|-------|------|-------------|
| `/` | Contracts | Unified view of all contracts and artifacts with deploy functionality |
| `/networks` | Networks | View configured blockchain networks from foundry.toml |
| `/wallets` | Wallets | Manage wallets for signing transactions |
| `/deployment/:contract/:network` | Deployment Details | View deployment details, interact with contract functions, view history |

## Development

```bash
# Install dependencies
bun install

# Start development server
bun dev

# Lint and format
bun check

# Build for production
bun run build
```

## Building

The production build outputs to `../crates/smolder-cli/assets/` where it's embedded into the Rust binary via `rust-embed`.

```bash
bun run build
```
