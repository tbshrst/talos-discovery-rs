mod cluster;
mod service;
mod discovery {
    tonic::include_proto!("sidero.discovery.server");
}

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{discovery::cluster_server::ClusterServer, service::DiscoveryService};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false))
        .init();

    let discovery_service = ClusterServer::new(DiscoveryService::new().await);
    let addr = "0.0.0.0:3000".parse().unwrap();

    tracing::info!("Starting Talos Discovery Service gRPC server: {}", addr);
    tonic::transport::Server::builder()
        .add_service(discovery_service)
        .serve(addr)
        .await
        .unwrap();
}
