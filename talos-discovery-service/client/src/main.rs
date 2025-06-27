mod chat;
mod client;

use clap::command;
use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::client::Client;

#[derive(Parser, Debug)]
#[command(name = "MyApp")]
#[command(about = "A CLI tool", long_about = None)]
struct Cli {
    /// Server address
    #[arg(long, default_value = "127.0.0.1")]
    address: String,

    /// Server port
    #[arg(long, default_value = "3000")]
    port: u16,

    /// ClusterID
    #[arg(long, default_value = "my-cluster-id")]
    cluster_id: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start a chat session
    Chat {
        /// Chatroom
        #[arg(long)]
        chatroom: String,

        /// Username for chat
        #[arg(long)]
        username: String,

        /// Optional password
        #[arg(long)]
        password: Option<String>,
    },

    /// Upload a file
    Upload {
        /// Path to the file
        #[arg(long)]
        _filepath: String,
    },

    /// Download a file
    Download {
        /// Path to the file
        #[arg(long)]
        _filepath: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_args = Cli::parse();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false))
        .init();

    Client::new(cli_args).await?.execute().await
}
