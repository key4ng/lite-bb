pub mod auth;
pub mod pr;
pub mod repo;
pub mod search;

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
    /// Manage repositories
    Repo {
        #[command(subcommand)]
        command: repo::RepoCommands,
    },
    /// Search code
    Search {
        #[command(subcommand)]
        command: search::SearchCommands,
    },
}

pub async fn run(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Auth { command } => auth::run(command).await,
        Commands::Pr { command } => pr::run(command).await,
        Commands::Repo { command } => repo::run(command).await,
        Commands::Search { command } => search::run(command).await,
    }
}
