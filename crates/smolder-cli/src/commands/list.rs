//! List all deployments

use clap::Args;
use color_eyre::eyre::Result;
use console::style;

use smolder_db::Database;

/// List all deployments
#[derive(Args)]
pub struct ListCommand {
    /// Filter by network
    #[arg(long)]
    pub network: Option<String>,
}

impl ListCommand {
    pub async fn run(self) -> Result<()> {
        let db = Database::connect().await?;
        let deployments = db.list_deployments(self.network.as_deref()).await?;

        if deployments.is_empty() {
            println!("No deployments found.");
            if self.network.is_some() {
                println!(
                    "Try running without {} to see all deployments.",
                    style("--network").yellow()
                );
            }
            return Ok(());
        }

        // Print table header
        println!(
            "{:<15} {:<20} {:<8} {:<44} {:<20}",
            "Network", "Contract", "Version", "Address", "Deployed At"
        );
        println!("{}", "-".repeat(110));

        // Print each deployment
        for d in &deployments {
            println!(
                "{:<15} {:<20} {:<8} {:<44} {:<20}",
                d.network_name,
                d.contract_name,
                format!("v{}", d.version),
                d.address,
                &d.deployed_at[..19] // Trim to just date and time
            );
        }

        println!();
        println!("Total: {} deployment(s)", deployments.len());

        Ok(())
    }
}
