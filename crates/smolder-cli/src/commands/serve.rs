//! Start the web server for the dashboard UI

use clap::Args;
use color_eyre::eyre::{eyre, Result};
use console::style;

use crate::server::ServerConfig;
use smolder_db::Database;

const DB_FILE: &str = "smolder.db";

/// Start the web server for the dashboard UI
#[derive(Args)]
pub struct ServeCommand {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port to listen on
    #[arg(long, short, default_value = "3000")]
    pub port: u16,
}

impl ServeCommand {
    pub async fn run(self) -> Result<()> {
        // Check if database exists
        if !std::path::Path::new(DB_FILE).exists() {
            return Err(eyre!(
                "Database not found. Run {} first.",
                style("smolder init").yellow()
            ));
        }

        // Connect to database
        let db = Database::connect().await?;

        let config = ServerConfig {
            host: self.host.clone(),
            port: self.port,
        };

        println!("{} Starting Smolder server...", style("→").blue());
        println!();
        println!(
            "  {} Dashboard: {}",
            style("◆").cyan(),
            style(format!("http://{}:{}", self.host, self.port))
                .underlined()
                .cyan()
        );
        println!(
            "  {} API:       {}",
            style("◆").cyan(),
            style(format!("http://{}:{}/api", self.host, self.port))
                .underlined()
                .cyan()
        );
        println!();
        println!("  Press {} to stop the server", style("Ctrl+C").yellow());
        println!();

        crate::server::run_server(db, config)
            .await
            .map_err(|e| eyre!("Server error: {}", e))?;

        Ok(())
    }
}
