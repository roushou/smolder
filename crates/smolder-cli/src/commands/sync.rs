//! Sync deployments from broadcast directory

use std::collections::HashMap;
use std::path::Path;

use clap::Args;
use color_eyre::eyre::{eyre, Result};
use console::style;
use smolder_db::{ChainId, NewContract, NewDeployment, NewNetwork};

use crate::config::FoundryConfig;
use crate::forge::{BroadcastOutput, BroadcastParser, ForgeBroadcastParser};
use crate::rpc::get_chain_id;
use smolder_db::Database;

/// Sync deployments from broadcast directory
#[derive(Args)]
pub struct SyncCommand;

impl SyncCommand {
    pub async fn run(self) -> Result<()> {
        // Load foundry config
        let config = FoundryConfig::load()?;

        // Scan for broadcast files
        println!("{} Scanning broadcast directory...", style("->").blue());
        let broadcast_files = scan_broadcast_directory()?;

        if broadcast_files.is_empty() {
            println!(
                "{} No broadcast files found in broadcast/",
                style("!").yellow()
            );
            return Ok(());
        }

        println!(
            "   Found {} broadcast file(s)",
            style(broadcast_files.len()).cyan()
        );

        // Build chain_id -> network mapping by querying RPC for each network
        println!(
            "{} Resolving networks from foundry.toml...",
            style("->").blue()
        );
        let mut chain_to_network: HashMap<u64, (String, String, Option<String>)> = HashMap::new();

        for network_name in config.network_names() {
            let network = match config.get_network(network_name) {
                Ok(n) => n,
                Err(e) => {
                    println!(
                        "   {} Skipping {}: {}",
                        style("!").yellow(),
                        network_name,
                        e
                    );
                    continue;
                }
            };

            match get_chain_id(&network.rpc_url).await {
                Ok(chain_id) => {
                    chain_to_network.insert(
                        chain_id,
                        (
                            network.name.clone(),
                            network.rpc_url.clone(),
                            network.explorer_url.clone(),
                        ),
                    );
                    println!(
                        "   {} {} (chain ID: {})",
                        style("*").dim(),
                        style(&network.name).cyan(),
                        chain_id
                    );
                }
                Err(e) => {
                    println!(
                        "   {} Could not connect to {}: {}",
                        style("!").yellow(),
                        network_name,
                        e
                    );
                }
            }
        }

        if chain_to_network.is_empty() {
            return Err(eyre!(
                "No networks could be resolved. Check your foundry.toml and RPC endpoints."
            ));
        }

        // Connect to database
        let db = Database::connect().await?;

        let mut total_imported = 0;
        let mut total_skipped = 0;

        // Process each broadcast file
        for broadcast_file in &broadcast_files {
            let network_info = match chain_to_network.get(&broadcast_file.chain_id) {
                Some(info) => info,
                None => {
                    println!(
                        "{} Skipping {} - no network configured for chain ID {}",
                        style("!").yellow(),
                        broadcast_file.script_name,
                        broadcast_file.chain_id
                    );
                    continue;
                }
            };

            let (network_name, rpc_url, explorer_url) = network_info;

            println!(
                "{} Processing {} on {}...",
                style("->").blue(),
                style(&broadcast_file.script_name).cyan(),
                style(network_name).cyan()
            );

            // Load and parse broadcast
            let broadcast = match load_broadcast(&broadcast_file.path) {
                Ok(b) => b,
                Err(e) => {
                    println!(
                        "   {} Failed to parse {}: {}",
                        style("!").yellow(),
                        broadcast_file.path,
                        e
                    );
                    continue;
                }
            };

            // Extract deployments
            let parser = ForgeBroadcastParser::new();
            let deployments = match parser.extract_deployments(&broadcast) {
                Ok(d) => d,
                Err(e) => {
                    println!(
                        "   {} Failed to extract deployments: {}",
                        style("!").yellow(),
                        e
                    );
                    continue;
                }
            };

            if deployments.is_empty() {
                println!("   No deployments found");
                continue;
            }

            // Ensure network exists in database
            let network_id = db
                .upsert_network(&NewNetwork {
                    name: network_name.clone(),
                    chain_id: ChainId::from(broadcast_file.chain_id),
                    rpc_url: rpc_url.clone(),
                    explorer_url: explorer_url.clone(),
                })
                .await?;

            // Import each deployment
            for deployment in &deployments {
                // Check if already exists
                if db.deployment_exists_by_tx_hash(&deployment.tx_hash).await? {
                    println!(
                        "   {} {} already tracked (tx: {}...)",
                        style("-").dim(),
                        style(&deployment.contract_name).dim(),
                        &deployment.tx_hash[..10]
                    );
                    total_skipped += 1;
                    continue;
                }

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
                    "   {} {} at {}",
                    style("+").green(),
                    style(&deployment.contract_name).cyan(),
                    style(&deployment.address).yellow()
                );
                total_imported += 1;
            }
        }

        println!();
        if total_imported > 0 {
            println!(
                "{} Imported {} deployment(s)",
                style("*").green().bold(),
                total_imported
            );
        }
        if total_skipped > 0 {
            println!(
                "{} Skipped {} already tracked deployment(s)",
                style("*").dim(),
                total_skipped
            );
        }
        if total_imported == 0 && total_skipped == 0 {
            println!("{} No deployments found to import", style("*").yellow());
        }

        Ok(())
    }
}

/// Discovered broadcast file with metadata
struct BroadcastFile {
    path: String,
    chain_id: u64,
    script_name: String,
}

/// Scan the broadcast directory for all run-latest.json files
fn scan_broadcast_directory() -> Result<Vec<BroadcastFile>> {
    let broadcast_dir = Path::new("broadcast");
    if !broadcast_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    // broadcast/<ScriptName>/<chainId>/run-latest.json
    for script_entry in std::fs::read_dir(broadcast_dir)? {
        let script_entry = script_entry?;
        let script_path = script_entry.path();

        if !script_path.is_dir() {
            continue;
        }

        let script_name = script_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        for chain_entry in std::fs::read_dir(&script_path)? {
            let chain_entry = chain_entry?;
            let chain_path = chain_entry.path();

            if !chain_path.is_dir() {
                continue;
            }

            let chain_id_str = chain_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let chain_id: u64 = match chain_id_str.parse() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let run_latest = chain_path.join("run-latest.json");
            if run_latest.exists() {
                files.push(BroadcastFile {
                    path: run_latest.to_string_lossy().to_string(),
                    chain_id,
                    script_name: script_name.clone(),
                });
            }
        }
    }

    Ok(files)
}

/// Load and parse a broadcast file
fn load_broadcast(path: &str) -> Result<BroadcastOutput> {
    let content = std::fs::read_to_string(path)?;
    let output: BroadcastOutput = serde_json::from_str(&content)?;
    Ok(output)
}
