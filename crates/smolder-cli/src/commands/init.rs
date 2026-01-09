//! Initialize smolder in a Foundry project

use std::path::Path;

use clap::Args;
use color_eyre::eyre::{eyre, Result};
use console::style;
use smolder_core::SmolderDir;
use smolder_db::Database;

use crate::config::FoundryConfig;

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
        if Database::exists() {
            return Err(eyre!(
                "Smolder is already initialized in this project ({} exists)",
                SmolderDir::NAME
            ));
        }

        // Create .smolder/ directory
        let dir = SmolderDir::new();
        dir.create()?;
        println!("{} Created {}/", style("✓").green(), SmolderDir::NAME);

        // Create and initialize database
        let db = Database::connect().await?;
        db.init_schema().await?;
        println!("{} Initialized database", style("✓").green());

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
    let entry = SmolderDir::NAME;

    if gitignore_path.exists() {
        let content = std::fs::read_to_string(gitignore_path)?;
        // Check for both `.smolder` and `.smolder/` patterns
        let has_entry = content
            .lines()
            .any(|line| line.trim() == entry || line.trim() == format!("{}/", entry));
        if !has_entry {
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
