use clap::Parser;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use discovery_api::{HelloRequest, cluster_client::ClusterClient};

#[derive(Parser, Debug, Clone)]
#[clap(version = "1.0", author = "genua GmbH", next_line_help = true)]
pub struct Config {
    // Listen port
    #[clap(long, env = "ADDRESS", default_value = "127.0.0.1")]
    pub address: String,

    // Target port
    #[clap(long, env = "PORT", default_value = "3000")]
    pub port: u16,
}

#[tokio::main]
async fn main() {
    let config = Config::parse();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false))
        .init();

    let addr = format!("http://{}:{}", config.address, config.port);

    let mut client = ClusterClient::connect(addr).await.unwrap();
    info!("connected");
    let _ = client
        .hello(HelloRequest {
            cluster_id: "1337".to_string(),
            client_version: "1338".to_string(),
        })
        .await;
}
