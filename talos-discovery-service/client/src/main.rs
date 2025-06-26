use clap::command;
use clap::{Parser, Subcommand};
use tonic::transport::Channel;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use discovery_api::{HelloRequest, cluster_client::ClusterClient};

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
        filepath: String,
    },

    /// Download a file
    Download {
        /// Path to the file
        #[arg(long)]
        filepath: String,
    },
}

// async fn chat_loop(username: String, password: Option<String>) {}

impl Commands {
    pub async fn execute(&self) {
        match self {
            Commands::Chat {
                chatroom,
                username,
                password,
            } => {
                println!("Chatting as {} with password {:?}", username, password);
            }
            Commands::Upload { filepath } => {
                println!("Uploading file: {}", filepath);
            }
            Commands::Download { filepath } => {
                println!("Downloading file: {}", filepath);
            }
        }
    }
}

struct Cluster {
    cluster_id: String,
    affiliate_id: String,
    address: String,
    port: u16,
}

struct Client {
    command: ClientCommand,
    client: ClusterClient<Channel>,
    cluster: Cluster,
}

enum ClientCommand {
    Chat(Chat),
    Upload,
    Download,
}

struct Chat {
    chatroom: String,
    username: String,
    password: Option<String>,
}

impl Chat {
    fn new(chatroom: String, username: String, password: Option<String>) -> Self {
        Self {
            chatroom,
            username,
            password,
        }
    }
}

impl Client {
    async fn new(cli_args: Cli) -> anyhow::Result<Self> {
        let command = match cli_args.command {
            Commands::Chat {
                chatroom,
                username,
                password,
            } => ClientCommand::Chat(Chat::new(chatroom, username, password)),
            Commands::Upload { filepath } => ClientCommand::Upload,
            Commands::Download { filepath } => ClientCommand::Download,
        };

        let client = Self::connect(cli_args.address.clone(), cli_args.port).await?;

        Ok(Self {
            command,
            client,
            cluster: Cluster {
                cluster_id: "todo!()".to_string(),
                affiliate_id: "todo!()".to_string(),
                address: cli_args.address,
                port: cli_args.port,
            },
        })
    }

    async fn connect(address: String, port: u16) -> anyhow::Result<ClusterClient<Channel>> {
        let addr = format!("http://{}:{}", address, port);
        let mut client = ClusterClient::connect(addr).await?;
        info!("connected");

        let _ = client
            .hello(HelloRequest {
                cluster_id: "1337".to_string(),
                client_version: "1338".to_string(),
            })
            .await?;

        Ok(client)
    }

    pub async fn execute(self) -> anyhow::Result<()> {
        println!("EXEC CHAT");
        todo!()
    }

    pub async fn _hash(plain: String) -> String {
        sha256::digest(plain)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_args = Cli::parse();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false))
        .init();

    Ok(Client::new(cli_args).await?.execute().await?)
}
