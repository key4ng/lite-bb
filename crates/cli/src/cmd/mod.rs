pub mod auth;
pub mod pr;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        command: auth::AuthCommands,
    },
    /// Manage pull requests
    Pr {
        #[command(subcommand)]
        command: pr::PrCommands,
    },
}

pub async fn run(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Auth { command } => auth::run(command).await,
        Commands::Pr { command } => pr::run(command).await,
    }
}
