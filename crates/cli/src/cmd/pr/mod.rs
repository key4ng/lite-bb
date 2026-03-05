mod checkout;
mod checks;
mod close;
mod comment;
mod create;
mod diff;
mod edit;
mod list;
mod merge;
mod reopen;
mod review;
mod view;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum PrCommands {
    /// List pull requests
    List,
    /// View a pull request
    View {
        /// Pull request number
        number: u64,
    },
    /// Create a pull request
    Create,
    /// Merge a pull request
    Merge {
        /// Pull request number
        number: u64,
    },
    /// Checkout a PR branch locally
    Checkout {
        /// Pull request number
        number: u64,
    },
    /// Close/decline a pull request
    Close {
        /// Pull request number
        number: u64,
    },
    /// Reopen a declined pull request
    Reopen {
        /// Pull request number
        number: u64,
    },
    /// Edit PR title, description, or base
    Edit {
        /// Pull request number
        number: u64,
    },
    /// Add a review (approve/request-changes)
    Review {
        /// Pull request number
        number: u64,
    },
    /// Add a comment to a PR
    Comment {
        /// Pull request number
        number: u64,
    },
    /// View pull request diff
    Diff {
        /// Pull request number
        number: u64,
    },
    /// View CI/CD status checks
    Checks {
        /// Pull request number
        number: u64,
    },
}

pub async fn run(command: PrCommands) -> anyhow::Result<()> {
    match command {
        PrCommands::List => list::run().await,
        PrCommands::View { number } => view::run(number).await,
        PrCommands::Create => create::run().await,
        PrCommands::Merge { number } => merge::run(number).await,
        PrCommands::Checkout { number } => checkout::run(number).await,
        PrCommands::Close { number } => close::run(number).await,
        PrCommands::Reopen { number } => reopen::run(number).await,
        PrCommands::Edit { number } => edit::run(number).await,
        PrCommands::Review { number } => review::run(number).await,
        PrCommands::Comment { number } => comment::run(number).await,
        PrCommands::Diff { number } => diff::run(number).await,
        PrCommands::Checks { number } => checks::run(number).await,
    }
}
