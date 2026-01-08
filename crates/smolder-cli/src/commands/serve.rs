use color_eyre::eyre::{eyre, Result};
use console::style;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

use crate::server::ServerConfig;

const DB_FILE: &str = "smolder.db";

pub async fn run(host: &str, port: u16) -> Result<()> {
    // Check if database exists
    if !std::path::Path::new(DB_FILE).exists() {
        return Err(eyre!(
            "Database not found. Run {} first.",
            style("smolder init").yellow()
        ));
    }

    // Connect to database
    let options = SqliteConnectOptions::from_str(DB_FILE)?
        .create_if_missing(false)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    let config = ServerConfig {
        host: host.to_string(),
        port,
    };

    println!("{} Starting Smolder server...", style("→").blue());
    println!();
    println!(
        "  {} Dashboard: {}",
        style("◆").cyan(),
        style(format!("http://{}:{}", host, port))
            .underlined()
            .cyan()
    );
    println!(
        "  {} API:       {}",
        style("◆").cyan(),
        style(format!("http://{}:{}/api", host, port))
            .underlined()
            .cyan()
    );
    println!();
    println!("  Press {} to stop the server", style("Ctrl+C").yellow());
    println!();

    crate::server::run_server(pool, config)
        .await
        .map_err(|e| eyre!("Server error: {}", e))?;

    Ok(())
}
