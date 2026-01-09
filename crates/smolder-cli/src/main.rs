mod commands;
mod config;
mod forge;
mod rpc;
mod server;

use clap::Parser;
use color_eyre::eyre::Result;

use commands::Command;

#[derive(Parser)]
#[command(name = "smolder")]
#[command(about = "Contract registry and interaction platform for Foundry")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    cli.command.run().await
}
