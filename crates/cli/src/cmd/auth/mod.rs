mod login;
mod logout;
mod status;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Authenticate with Bitbucket
    Login,
    /// Log out of Bitbucket
    Logout,
    /// View authentication status
    Status,
}

pub async fn run(command: AuthCommands) -> anyhow::Result<()> {
    match command {
        AuthCommands::Login => login::run().await,
        AuthCommands::Logout => logout::run().await,
        AuthCommands::Status => status::run().await,
    }
}
