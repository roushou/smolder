//! Deploy contracts via forge script and track in database

use std::process::Command;

use clap::Args;
use color_eyre::eyre::{eyre, Result};
use console::style;
use smolder_db::{ChainId, NewContract, NewDeployment, NewNetwork};

use crate::config::FoundryConfig;
use crate::forge::{BroadcastParser, ForgeBroadcastParser};
use crate::rpc::get_chain_id;
use smolder_db::Database;

/// Deploy contracts via forge script and track in database
#[derive(Args)]
pub struct DeployCommand {
    /// Path to the deployment script
    pub script: String,

    /// Network to deploy to
    #[arg(long)]
    pub network: String,

    /// Actually broadcast the transaction (dry-run if omitted)
    #[arg(long)]
    pub broadcast: bool,
}

impl DeployCommand {
    pub async fn run(self) -> Result<()> {
        // Load config from foundry.toml
        let config = FoundryConfig::load()?;
        let network = config.get_network(&self.network)?;

        // Fetch chain ID from RPC
        println!(
            "{} Connecting to {}...",
            style("→").blue(),
            style(&self.network).cyan()
        );
        let chain_id = get_chain_id(&network.rpc_url).await?;

        println!(
            "{} Deploying to {} (chain ID: {})",
            style("→").blue(),
            style(&network.name).cyan(),
            chain_id
        );

        // Build forge command
        let mut cmd = Command::new("forge");
        cmd.arg("script")
            .arg(&self.script)
            .arg("--rpc-url")
            .arg(&network.rpc_url);

        if self.broadcast {
            cmd.arg("--broadcast");
        }

        // Execute forge script
        println!("{} Running forge script...", style("→").blue());
        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(eyre!("Forge script failed:\n{}", stderr));
        }

        println!("{}", String::from_utf8_lossy(&output.stdout));

        if !self.broadcast {
            println!();
            println!(
                "{} Dry run complete. Use {} to actually deploy.",
                style("ℹ").blue(),
                style("--broadcast").yellow()
            );
            return Ok(());
        }

        // Parse broadcast output
        println!("{} Parsing deployment data...", style("→").blue());
        let parser = ForgeBroadcastParser::new();
        let broadcast_output = parser.parse(&self.script, chain_id)?;
        let deployments = parser.extract_deployments(&broadcast_output)?;

        if deployments.is_empty() {
            println!(
                "{} No contract deployments found in broadcast",
                style("⚠").yellow()
            );
            return Ok(());
        }

        // Connect to database
        let db = Database::connect().await?;

        // Ensure network exists in database
        let network_id = db
            .upsert_network(&NewNetwork {
                name: network.name.clone(),
                chain_id: ChainId::from(chain_id),
                rpc_url: network.rpc_url.clone(),
                explorer_url: network.explorer_url.clone(),
            })
            .await?;

        // Store each deployment
        for deployment in &deployments {
            // Upsert contract
            let contract_id = db
                .upsert_contract(&NewContract {
                    name: deployment.contract_name.clone(),
                    source_path: deployment.source_path.clone(),
                    abi: deployment.abi.clone(),
                    bytecode_hash: deployment.bytecode_hash.clone(),
                })
                .await?;

            // Create deployment record
            db.create_deployment(&NewDeployment {
                contract_id,
                network_id,
                address: deployment.address.clone(),
                deployer: deployment.deployer.clone(),
                tx_hash: deployment.tx_hash.clone(),
                block_number: deployment.block_number,
                constructor_args: deployment.constructor_args.clone(),
            })
            .await?;

            println!(
                "{} {} deployed at {}",
                style("✓").green(),
                style(&deployment.contract_name).cyan(),
                style(&deployment.address).yellow()
            );
        }

        println!();
        println!(
            "{} {} contract(s) deployed and tracked",
            style("✓").green().bold(),
            deployments.len()
        );

        Ok(())
    }
}
