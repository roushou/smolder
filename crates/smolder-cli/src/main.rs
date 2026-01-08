mod commands;
mod config;
mod db;
mod forge;
mod rpc;
mod server;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[derive(Parser)]
#[command(name = "smolder")]
#[command(about = "Contract registry and interaction platform for Foundry")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize smolder in a Foundry project
    Init,

    /// Deploy contracts via forge script and track in database
    Deploy {
        /// Path to the deployment script
        script: String,

        /// Network to deploy to
        #[arg(long)]
        network: String,

        /// Actually broadcast the transaction (dry-run if omitted)
        #[arg(long)]
        broadcast: bool,
    },

    /// List all deployments
    List {
        /// Filter by network
        #[arg(long)]
        network: Option<String>,
    },

    /// Get the address of a deployed contract
    Get {
        /// Contract name
        contract: String,

        /// Network name
        #[arg(long)]
        network: String,
    },

    /// Export deployments to various formats
    Export {
        /// Output format: json, ts, env
        #[arg(long, default_value = "json")]
        format: String,

        /// Output file path
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Start the web server for the dashboard UI
    Serve {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Port to listen on
        #[arg(long, short, default_value = "3000")]
        port: u16,
    },

    /// Sync deployments from broadcast directory
    Sync,

    /// Manage wallets for signing transactions
    Wallet {
        #[command(subcommand)]
        command: WalletCommands,
    },
}

#[derive(Subcommand)]
enum WalletCommands {
    /// Add a new wallet
    Add {
        /// Wallet name (unique identifier)
        name: String,
    },

    /// List all wallets
    List,

    /// Remove a wallet
    Remove {
        /// Wallet name to remove
        name: String,

        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::run().await,
        Commands::Deploy {
            script,
            network,
            broadcast,
        } => commands::deploy::run(&script, &network, broadcast).await,
        Commands::List { network } => commands::list::run(network.as_deref()).await,
        Commands::Get { contract, network } => commands::get::run(&contract, &network).await,
        Commands::Export { format, output } => {
            commands::export::run(&format, output.as_deref()).await
        }
        Commands::Serve { host, port } => commands::serve::run(&host, port).await,
        Commands::Sync => commands::sync::run().await,
        Commands::Wallet { command } => match command {
            WalletCommands::Add { name } => commands::wallet::add(&name).await,
            WalletCommands::List => commands::wallet::list().await,
            WalletCommands::Remove { name, force } => commands::wallet::remove(&name, force).await,
        },
    }
}
