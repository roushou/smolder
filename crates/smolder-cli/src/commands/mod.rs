//! CLI commands for smolder

use clap::Subcommand;
use color_eyre::eyre::Result;

pub mod deploy;
pub mod export;
pub mod get;
pub mod init;
pub mod list;
pub mod serve;
pub mod sync;
pub mod wallet;

/// All available CLI commands
#[derive(Subcommand)]
pub enum Command {
    /// Initialize smolder in a Foundry project
    Init(init::InitCommand),

    /// Deploy contracts via forge script and track in database
    Deploy(deploy::DeployCommand),

    /// List all deployments
    List(list::ListCommand),

    /// Get the address of a deployed contract
    Get(get::GetCommand),

    /// Export deployments to various formats
    Export(export::ExportCommand),

    /// Start the web server for the dashboard UI
    Serve(serve::ServeCommand),

    /// Sync deployments from broadcast directory
    Sync(sync::SyncCommand),

    /// Manage wallets for signing transactions
    Wallet(wallet::WalletCommand),
}

impl Command {
    /// Execute the command
    pub async fn run(self) -> Result<()> {
        match self {
            Command::Init(cmd) => cmd.run().await,
            Command::Deploy(cmd) => cmd.run().await,
            Command::List(cmd) => cmd.run().await,
            Command::Get(cmd) => cmd.run().await,
            Command::Export(cmd) => cmd.run().await,
            Command::Serve(cmd) => cmd.run().await,
            Command::Sync(cmd) => cmd.run().await,
            Command::Wallet(cmd) => cmd.run().await,
        }
    }
}
