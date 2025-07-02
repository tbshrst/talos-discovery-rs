mod cluster;
mod service;

use clap::Parser;
use discovery_api::{cluster_server::ClusterServer, tonic::transport::Server};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::service::DiscoveryService;

#[derive(Parser, Debug, Clone)]
#[clap(version = "1.0", next_line_help = true)]
pub struct Config {
    // Listen port
    #[clap(long, env = "PORT", default_value = "3000")]
    pub port: u16,

    // Garbage collection interval
    #[clap(long, env = "GC_INTERVAL", default_value = "60")]
    pub gc_interval: u16,

    // Backup path
    #[clap(long, env = "BACKUP_PATH")]
    pub backup_path: Option<String>,

    // Backup interval
    #[clap(long, env = "BACKUP_INTERVAL", default_value = "600")]
    pub backup_interval: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false))
        .init();

    let discovery_service = ClusterServer::new(
        DiscoveryService::new(config.gc_interval, config.backup_path, config.backup_interval).await?,
    );
    let addr = format!("0.0.0.0:{}", config.port).parse().unwrap();

    tracing::info!("Starting Talos Discovery Service gRPC server: {}", addr);
    Server::builder()
        .add_service(discovery_service)
        .serve(addr)
        .await
        .unwrap();

    Ok(())
}
