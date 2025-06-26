use tonic::transport::Channel;
use tracing::info;

use discovery_api::{HelloRequest, cluster_client::ClusterClient};

use crate::chat::Chat;
use crate::{Cli, Commands};

pub(crate) struct Client {
    command: ClientCommand,
    client: ClusterClient<Channel>,
    cluster: Cluster,
}

struct Cluster {
    cluster_id: String,
    affiliate_id: String,
    address: String,
    port: u16,
}
enum ClientCommand {
    Chat(Chat),
    Upload,
    Download,
}

impl Client {
    pub async fn new(cli_args: Cli) -> anyhow::Result<Self> {
        let command = match cli_args.command {
            Commands::Chat {
                chatroom,
                username,
                password,
            } => ClientCommand::Chat(Chat::new(chatroom, username, password)),
            Commands::Upload { filepath } => ClientCommand::Upload,
            Commands::Download { filepath } => ClientCommand::Download,
        };

        let cluster_id = {
            match &command {
                ClientCommand::Chat(chat) => Self::hash(&chat.chatroom),
                _ => cli_args.cluster_id,
            }
        };
        let affiliate_id = Self::hash(&"my-going-to-be-random-id".to_string());

        let client = Self::connect(cli_args.address.clone(), cli_args.port, cluster_id.clone()).await?;

        Ok(Self {
            command,
            client,
            cluster: Cluster {
                cluster_id,
                affiliate_id,
                address: cli_args.address,
                port: cli_args.port,
            },
        })
    }

    async fn connect(address: String, port: u16, cluster_id: String) -> anyhow::Result<ClusterClient<Channel>> {
        let addr = format!("http://{}:{}", address, port);
        let mut client = ClusterClient::connect(addr).await?;
        info!("connected");

        let _ = client
            .hello(HelloRequest {
                cluster_id,
                client_version: "v1.9.2".to_string(), // ofc we are legit
            })
            .await?;

        Ok(client)
    }

    pub async fn execute(self) -> anyhow::Result<()> {
        match self.command {
            ClientCommand::Chat(chat) => chat.execute().await,
            ClientCommand::Upload => todo!(),
            ClientCommand::Download => todo!(),
        }

        Ok(())
    }

    pub fn hash(plain: &String) -> String {
        sha256::digest(plain)
    }
}
