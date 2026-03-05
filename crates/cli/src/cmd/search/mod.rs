mod code;

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct RepoArgs {
    /// Repository in WORKSPACE/REPO or ~username/REPO format
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
pub enum SearchCommands {
    /// Search for code
    Code {
        /// Search query
        query: String,
        #[command(flatten)]
        repo: RepoArgs,
        #[command(flatten)]
        json: JsonFlag,
        /// Maximum number of results
        #[arg(short = 'L', long, default_value = "30")]
        limit: u32,
        /// Filter by file extension (e.g. rs, py)
        #[arg(long)]
        extension: Option<String>,
        /// Filter by filename
        #[arg(long)]
        filename: Option<String>,
    },
}

pub async fn run(command: SearchCommands) -> anyhow::Result<()> {
    match command {
        SearchCommands::Code {
            query,
            repo,
            json,
            limit,
            extension,
            filename,
        } => code::run(query, repo, json, limit, extension, filename).await,
    }
}
