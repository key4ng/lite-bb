pub mod api;
pub mod auth;
pub mod pr;
pub mod repo;
pub mod search;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Make an authenticated API request
    Api {
        /// API endpoint path (relative to base URL)
        endpoint: String,
        /// HTTP method [default: GET]
        #[arg(short = 'X', long, default_value = "GET")]
        method: String,
        /// Add a JSON body field (key=value). Repeat for multiple fields.
        #[arg(short = 'F', long = "field", value_name = "key=value")]
        fields: Vec<String>,
        /// Add a request header ('Key: Value'). Repeat for multiple headers.
        #[arg(short = 'H', long = "header", value_name = "key:value")]
        headers: Vec<String>,
        /// Filter JSON output with a jq expression (requires jq)
        #[arg(short = 'q', long)]
        jq: Option<String>,
        /// Print response body as-is without JSON formatting
        #[arg(long)]
        raw: bool,
        /// JSON string to use as request body (overrides --field)
        #[arg(long)]
        input: Option<String>,
    },
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
        Commands::Api { endpoint, method, fields, headers, jq, raw, input } => {
            api::run(endpoint, method, fields, headers, jq, raw, input).await
        }
        Commands::Auth { command } => auth::run(command).await,
        Commands::Pr { command } => pr::run(command).await,
        Commands::Repo { command } => repo::run(command).await,
        Commands::Search { command } => search::run(command).await,
    }
}
