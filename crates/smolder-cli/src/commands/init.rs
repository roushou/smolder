//! Initialize smolder in a Foundry project

use std::path::Path;

use clap::Args;
use color_eyre::eyre::{eyre, Result};
use console::style;

use crate::config::FoundryConfig;
use crate::db::Database;

const DB_FILE: &str = "smolder.db";

/// Initialize smolder in a Foundry project
#[derive(Args)]
pub struct InitCommand;

impl InitCommand {
    pub async fn run(self) -> Result<()> {
        // Check if we're in a Foundry project
        if !FoundryConfig::exists() {
            return Err(eyre!(
                "Not a Foundry project. Please run this command in a directory with foundry.toml"
            ));
        }

        // Check if already initialized
        if Path::new(DB_FILE).exists() {
            return Err(eyre!(
                "Smolder is already initialized in this project ({} exists)",
                DB_FILE
            ));
        }

        // Create and initialize database
        let db = Database::connect().await?;
        db.init_schema().await?;
        println!("{} Created {}", style("✓").green(), DB_FILE);

        // Optionally add to .gitignore
        add_to_gitignore()?;

        println!();
        println!(
            "{} Smolder initialized successfully!",
            style("✓").green().bold()
        );
        println!();
        println!("Next steps:");
        println!(
            "  1. Configure networks in foundry.toml under {}",
            style("[rpc_endpoints]").cyan()
        );
        println!(
            "  2. Run {} to deploy contracts",
            style("smolder deploy <script> --network <name>").cyan()
        );

        Ok(())
    }
}

fn add_to_gitignore() -> Result<()> {
    let gitignore_path = Path::new(".gitignore");
    let entry = "smolder.db";

    if gitignore_path.exists() {
        let content = std::fs::read_to_string(gitignore_path)?;
        if !content.lines().any(|line| line.trim() == entry) {
            let mut new_content = content;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str(entry);
            new_content.push('\n');
            std::fs::write(gitignore_path, new_content)?;
            println!("{} Added {} to .gitignore", style("✓").green(), entry);
        }
    }

    Ok(())
}
