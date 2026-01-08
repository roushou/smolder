use color_eyre::eyre::{eyre, Result};

use crate::db::Database;

pub async fn run(contract: &str, network: &str) -> Result<()> {
    let db = Database::connect().await?;

    let deployment = db.get_current_deployment(contract, network).await?;

    match deployment {
        Some(d) => {
            // Just print the address for easy scripting: $(smolder get MyToken --network tempo)
            println!("{}", d.address);
            Ok(())
        }
        None => Err(eyre!(
            "No deployment found for contract '{}' on network '{}'",
            contract,
            network
        )),
    }
}
