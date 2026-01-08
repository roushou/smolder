use color_eyre::eyre::Result;
use console::style;

use crate::db::Database;

pub async fn run(network: Option<&str>) -> Result<()> {
    let db = Database::connect().await?;
    let deployments = db.list_deployments(network).await?;

    if deployments.is_empty() {
        println!("No deployments found.");
        if network.is_some() {
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
