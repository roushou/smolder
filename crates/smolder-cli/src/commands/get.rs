//! Get the address of a deployed contract

use clap::Args;
use color_eyre::eyre::{eyre, Result};

use smolder_db::Database;

/// Get the address of a deployed contract
#[derive(Args)]
pub struct GetCommand {
    /// Contract name
    pub contract: String,

    /// Network name
    #[arg(long)]
    pub network: String,
}

impl GetCommand {
    pub async fn run(self) -> Result<()> {
        let db = Database::connect().await?;

        let deployment = db
            .get_current_deployment(&self.contract, &self.network)
            .await?;

        match deployment {
            Some(d) => {
                // Just print the address for easy scripting: $(smolder get MyToken --network tempo)
                println!("{}", d.address);
                Ok(())
            }
            None => Err(eyre!(
                "No deployment found for contract '{}' on network '{}'",
                self.contract,
                self.network
            )),
        }
    }
}
