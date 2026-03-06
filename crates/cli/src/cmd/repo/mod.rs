pub mod clone;
pub mod create;
pub mod list;
pub mod view;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum RepoCommands {
    /// List repositories in a workspace or project
    List {
        /// Workspace (Cloud) or project key (Server/DC). Defaults to configured workspace.
        owner: Option<String>,
        /// Maximum number of repositories to list
        #[arg(short = 'L', long, default_value = "30")]
        limit: u32,
        /// Filter by visibility: public or private
        #[arg(long, value_name = "public|private")]
        visibility: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// View a repository
    View {
        /// Repository in WORKSPACE/REPO or PROJECT/REPO format. Defaults to current repo.
        repo: Option<String>,
        /// Open repository in browser
        #[arg(short = 'w', long)]
        web: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Clone a repository locally
    Clone {
        /// Repository in WORKSPACE/REPO or PROJECT/REPO format
        repo: String,
        /// Local directory to clone into (defaults to repo slug)
        directory: Option<String>,
        /// Extra arguments forwarded to git clone
        #[arg(last = true)]
        git_args: Vec<String>,
    },
    /// Create a new repository
    Create {
        /// Repository name (slug)
        #[arg(short = 'n', long)]
        name: Option<String>,
        /// Description
        #[arg(short = 'd', long)]
        description: Option<String>,
        /// Workspace (Cloud) or project key (Server/DC). Defaults to configured workspace.
        #[arg(short = 'w', long)]
        workspace: Option<String>,
        /// Make the repository public (default: private)
        #[arg(long, conflicts_with = "private")]
        public: bool,
        /// Make the repository private (default)
        #[arg(long, conflicts_with = "public")]
        private: bool,
        /// Clone the repository locally after creation
        #[arg(long)]
        clone: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(command: RepoCommands) -> anyhow::Result<()> {
    match command {
        RepoCommands::List { owner, limit, visibility, json } => {
            list::run(owner, limit, visibility, json).await
        }
        RepoCommands::View { repo, web, json } => view::run(repo, web, json).await,
        RepoCommands::Clone { repo, directory, git_args } => {
            clone::run(repo, directory, git_args).await
        }
        RepoCommands::Create { name, description, workspace, public, private: _, clone, json } => {
            create::run(name, description, workspace, public, clone, json).await
        }
    }
}
