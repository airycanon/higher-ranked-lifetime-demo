use anyhow::Result;
use clap::{Parser, Subcommand};
use demo::cmd::forward::Args as ForwardArgs;
use demo::cmd::reverse::Args as ReverseArgs;
use demo::cmd::{forward, reverse};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(long_about = "run forward proxy")]
    ForwardProxy(ForwardArgs),
    #[command(long_about = "run reverse proxy")]
    ReverseProxy(ReverseArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ForwardProxy(args) => forward::run(args).await,
        Commands::ReverseProxy(args) => reverse::run(args).await,
    }
}
