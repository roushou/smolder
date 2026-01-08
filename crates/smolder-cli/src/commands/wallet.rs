use alloy::signers::local::PrivateKeySigner;
use color_eyre::eyre::{eyre, Result};
use console::style;
use dialoguer::{Confirm, Password};
use smolder_core::{keyring, NewWallet};

use crate::db::Database;

/// Add a new wallet
pub async fn add(name: &str) -> Result<()> {
    // Check if database exists
    let db = Database::connect().await?;

    // Check if wallet name already exists
    if db.get_wallet(name).await?.is_some() {
        return Err(eyre!("Wallet '{}' already exists", name));
    }

    // Prompt for private key
    println!(
        "{} Adding wallet '{}'",
        style("->").blue(),
        style(name).cyan()
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
    if db.get_wallet_by_address(&address).await?.is_some() {
        return Err(eyre!(
            "A wallet with address {} already exists",
            style(&address).yellow()
        ));
    }

    // Store private key in keyring
    keyring::store_private_key(name, &private_key)?;

    // Store wallet metadata in database
    db.create_wallet(&NewWallet {
        name: name.to_string(),
        address: address.clone(),
    })
    .await?;

    println!();
    println!(
        "{} Wallet '{}' added successfully",
        style("*").green().bold(),
        style(name).cyan()
    );
    println!("   Address: {}", style(&address).yellow());

    Ok(())
}

/// List all wallets
pub async fn list() -> Result<()> {
    let db = Database::connect().await?;
    let wallets = db.list_wallets().await?;

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
        let has_key = keyring::has_private_key(&wallet.name);
        let key_status = if has_key {
            style("*").green()
        } else {
            style("!").yellow()
        };

        println!(
            "   {} {} {}",
            key_status,
            style(&wallet.name).cyan().bold(),
            style(&wallet.address).yellow()
        );
    }

    println!();

    Ok(())
}

/// Remove a wallet
pub async fn remove(name: &str, force: bool) -> Result<()> {
    let db = Database::connect().await?;

    // Check if wallet exists
    let wallet = db
        .get_wallet(name)
        .await?
        .ok_or_else(|| eyre!("Wallet '{}' not found", name))?;

    // Confirm deletion unless forced
    if !force {
        println!(
            "{} About to remove wallet '{}'",
            style("!").yellow(),
            style(name).cyan()
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

    // Delete private key from keyring
    if keyring::has_private_key(name) {
        keyring::delete_private_key(name)?;
    }

    // Delete wallet from database
    db.delete_wallet(name).await?;

    println!();
    println!(
        "{} Wallet '{}' removed",
        style("*").green().bold(),
        style(name).cyan()
    );

    Ok(())
}
