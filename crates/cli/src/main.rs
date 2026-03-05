mod cmd;

use clap::Parser;

#[derive(Parser)]
#[command(name = "bb", about = "A gh-style CLI for Bitbucket")]
#[command(version, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: cmd::Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cmd::run(cli.command).await
}
