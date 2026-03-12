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

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct RepoArgs {
    /// Repository in WORKSPACE/REPO format
    #[arg(short = 'R', long)]
    pub repo: Option<String>,
}

#[derive(Args)]
pub struct JsonFlag {
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum PrCommands {
    /// List pull requests
    List {
        #[command(flatten)]
        repo: RepoArgs,
        #[command(flatten)]
        json: JsonFlag,
        /// Filter by state (OPEN, MERGED, DECLINED, SUPERSEDED)
        #[arg(short, long, default_value = "OPEN")]
        state: String,
        /// Maximum number of results
        #[arg(short = 'L', long, default_value = "30")]
        limit: u32,
    },
    /// View a pull request
    View {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
        #[command(flatten)]
        json: JsonFlag,
    },
    /// Create a pull request
    Create {
        #[command(flatten)]
        repo: RepoArgs,
        /// PR title
        #[arg(short, long)]
        title: Option<String>,
        /// PR description
        #[arg(short = 'b', long)]
        body: Option<String>,
        /// Source branch (defaults to current branch)
        #[arg(short = 'H', long)]
        head: Option<String>,
        /// Destination branch
        #[arg(short = 'B', long)]
        base: Option<String>,
    },
    /// Merge a pull request
    Merge {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
        /// Merge strategy (merge_commit, squash, fast_forward)
        #[arg(short, long)]
        strategy: Option<String>,
        /// Merge commit message
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Checkout a PR branch locally
    Checkout {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
    },
    /// Close/decline a pull request
    Close {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
    },
    /// Reopen a declined pull request (re-open via update)
    Reopen {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
    },
    /// Edit PR title, description, or destination branch
    Edit {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
        /// New title
        #[arg(short, long)]
        title: Option<String>,
        /// New description
        #[arg(short = 'b', long)]
        body: Option<String>,
        /// New destination branch
        #[arg(short = 'B', long)]
        base: Option<String>,
    },
    /// Add a review (approve/request-changes)
    Review {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
        /// Approve the PR
        #[arg(long)]
        approve: bool,
        /// Request changes (unapprove)
        #[arg(long)]
        request_changes: bool,
    },
    /// Add a comment to a PR
    Comment {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
        /// Comment body
        #[arg(short = 'b', long)]
        body: String,
        /// File path for inline comment
        #[arg(long)]
        path: Option<String>,
        /// Line number for inline comment (new file side)
        #[arg(long)]
        line: Option<u32>,
    },
    /// View pull request diff
    Diff {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
    },
    /// View CI/CD status checks
    Checks {
        /// Pull request number
        number: u64,
        #[command(flatten)]
        repo: RepoArgs,
        #[command(flatten)]
        json: JsonFlag,
    },
}

pub async fn run(command: PrCommands) -> anyhow::Result<()> {
    match command {
        PrCommands::List { repo, json, state, limit } => {
            list::run(repo, json, &state, limit).await
        }
        PrCommands::View { number, repo, json } => view::run(number, repo, json).await,
        PrCommands::Create { repo, title, body, head, base } => {
            create::run(repo, title, body, head, base).await
        }
        PrCommands::Merge { number, repo, strategy, message } => {
            merge::run(number, repo, strategy, message).await
        }
        PrCommands::Checkout { number, repo } => checkout::run(number, repo).await,
        PrCommands::Close { number, repo } => close::run(number, repo).await,
        PrCommands::Reopen { number, repo } => reopen::run(number, repo).await,
        PrCommands::Edit { number, repo, title, body, base } => {
            edit::run(number, repo, title, body, base).await
        }
        PrCommands::Review { number, repo, approve, request_changes } => {
            review::run(number, repo, approve, request_changes).await
        }
        PrCommands::Comment { number, repo, body, path, line } => {
            comment::run(number, repo, &body, path.as_deref(), line).await
        }
        PrCommands::Diff { number, repo } => diff::run(number, repo).await,
        PrCommands::Checks { number, repo, json } => {
            checks::run(number, repo, json).await
        }
    }
}
