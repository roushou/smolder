//! Manage wallets for signing transactions

use alloy::signers::local::PrivateKeySigner;
use clap::{Args, Subcommand};
use color_eyre::eyre::{eyre, Result};
use console::style;
use dialoguer::{Confirm, Password};
use smolder_core::encrypt_private_key;
use smolder_db::{Database, NewWallet, WalletRepository};

/// Manage wallets for signing transactions
#[derive(Args)]
pub struct WalletCommand {
    #[command(subcommand)]
    pub command: WalletSubcommand,
}

impl WalletCommand {
    pub async fn run(self) -> Result<()> {
        self.command.run().await
    }
}

#[derive(Subcommand)]
pub enum WalletSubcommand {
    /// Add a new wallet
    Add(AddWalletCommand),

    /// List all wallets
    List(ListWalletsCommand),

    /// Remove a wallet
    Remove(RemoveWalletCommand),
}

impl WalletSubcommand {
    pub async fn run(self) -> Result<()> {
        match self {
            Self::Add(cmd) => cmd.run().await,
            Self::List(cmd) => cmd.run().await,
            Self::Remove(cmd) => cmd.run().await,
        }
    }
}

/// Add a new wallet
#[derive(Args)]
pub struct AddWalletCommand {
    /// Wallet name (unique identifier)
    pub name: String,
}

impl AddWalletCommand {
    pub async fn run(self) -> Result<()> {
        let db = Database::connect().await?;

        // Check if wallet name already exists
        if WalletRepository::get_by_name(&db, &self.name)
            .await?
            .is_some()
        {
            return Err(eyre!("Wallet '{}' already exists", self.name));
        }

        // Prompt for private key
        println!(
            "{} Adding wallet '{}'",
            style("->").blue(),
            style(&self.name).cyan()
        );
        println!();

        let private_key: String = Password::new()
            .with_prompt("Enter private key (with or without 0x prefix)")
            .interact()?;

        // Normalize private key (add 0x prefix if missing)
        let private_key = if private_key.starts_with("0x") {
            private_key
        } else {
            format!("0x{}", private_key)
        };

        // Parse and validate private key, get address
        let signer: PrivateKeySigner = private_key
            .parse()
            .map_err(|e| eyre!("Invalid private key: {}", e))?;

        let address = format!("{:?}", signer.address());

        // Check if address already exists
        if WalletRepository::get_by_address(&db, &address)
            .await?
            .is_some()
        {
            return Err(eyre!(
                "A wallet with address {} already exists",
                style(&address).yellow()
            ));
        }

        // Encrypt and store wallet with private key in database
        let encrypted_key = encrypt_private_key(&private_key)
            .map_err(|e| eyre!("Failed to encrypt private key: {}", e))?;

        WalletRepository::create(
            &db,
            &NewWallet {
                name: self.name.clone(),
                address: address.clone(),
                encrypted_key,
            },
        )
        .await?;

        println!();
        println!(
            "{} Wallet '{}' added successfully",
            style("*").green().bold(),
            style(&self.name).cyan()
        );
        println!("   Address: {}", style(&address).yellow());

        Ok(())
    }
}

/// List all wallets
#[derive(Args)]
pub struct ListWalletsCommand;

impl ListWalletsCommand {
    pub async fn run(self) -> Result<()> {
        let db = Database::connect().await?;
        let wallets = WalletRepository::list(&db).await?;

        if wallets.is_empty() {
            println!("{} No wallets found", style("!").yellow());
            println!();
            println!(
                "   Add a wallet with: {}",
                style("smolder wallet add <name>").cyan()
            );
            return Ok(());
        }

        println!("{} {} wallet(s) found", style("*").green(), wallets.len());
        println!();

        for wallet in wallets {
            println!(
                "   {} {} {}",
                style("*").green(),
                style(&wallet.name).cyan().bold(),
                style(&wallet.address).yellow()
            );
        }

        println!();

        Ok(())
    }
}

/// Remove a wallet
#[derive(Args)]
pub struct RemoveWalletCommand {
    /// Wallet name to remove
    pub name: String,

    /// Skip confirmation prompt
    #[arg(long, short)]
    pub force: bool,
}

impl RemoveWalletCommand {
    pub async fn run(self) -> Result<()> {
        let db = Database::connect().await?;

        // Check if wallet exists
        let wallet = WalletRepository::get_by_name(&db, &self.name)
            .await?
            .ok_or_else(|| eyre!("Wallet '{}' not found", self.name))?;

        // Confirm deletion unless forced
        if !self.force {
            println!(
                "{} About to remove wallet '{}'",
                style("!").yellow(),
                style(&self.name).cyan()
            );
            println!("   Address: {}", style(&wallet.address).yellow());
            println!();

            let confirmed = Confirm::new()
                .with_prompt("Are you sure you want to remove this wallet?")
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{} Cancelled", style("*").dim());
                return Ok(());
            }
        }

        // Delete wallet from database
        WalletRepository::delete(&db, &self.name).await?;

        println!();
        println!(
            "{} Wallet '{}' removed",
            style("*").green().bold(),
            style(&self.name).cyan()
        );

        Ok(())
    }
}
