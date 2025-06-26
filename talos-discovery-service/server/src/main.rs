mod cluster;
mod service;
mod discovery {
    tonic::include_proto!("sidero.discovery.server");
}

use clap::Parser;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{discovery::cluster_server::ClusterServer, service::DiscoveryService};

#[derive(Parser, Debug, Clone)]
#[clap(version = "1.0", author = "genua GmbH", next_line_help = true)]
pub struct Config {
    // Listen port
    #[clap(long, env = "PORT", default_value = "3000")]
    pub port: u16,

    // Garbage collection interval
    #[clap(long, env = "GC_INTERVAL", default_value = "60")]
    pub gc_interval: u16,
}

#[tokio::main]
async fn main() {
    let config = Config::parse();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false))
        .init();

    let discovery_service = ClusterServer::new(DiscoveryService::new(config.gc_interval).await);
    let addr = format!("0.0.0.0:{}", config.port).parse().unwrap();

    tracing::info!("Starting Talos Discovery Service gRPC server: {}", addr);
    tonic::transport::Server::builder()
        .add_service(discovery_service)
        .serve(addr)
        .await
        .unwrap();
}
