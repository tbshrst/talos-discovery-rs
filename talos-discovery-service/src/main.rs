use tonic::Status;

use crate::discovery::cluster_server::Cluster;
use tokio_stream::wrappers::ReceiverStream;
pub mod discovery {
    tonic::include_proto!("sidero.discovery.server");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let discovery_service = discovery::cluster_server::ClusterServer::new(ClusterService {});
    let addr = "0.0.0.0:3000".parse().unwrap();

    tracing::info!("Starting Talos Discovery Service gRPC sever: {}", addr);

    tonic::transport::Server::builder()
        .add_service(discovery_service)
        .serve(addr)
        .await
        .unwrap();
}

pub struct ClusterService;

#[tonic::async_trait]
impl Cluster for ClusterService {
    type WatchStream = ReceiverStream<Result<discovery::WatchResponse, Status>>;

    async fn hello(
        &self,
        request: tonic::Request<discovery::HelloRequest>,
    ) -> std::result::Result<tonic::Response<discovery::HelloResponse>, tonic::Status> {
        tracing::info!("{:#?}", request);
        tracing::info!("{:#?}", request.remote_addr());
        Err(tonic::Status::ok("ERROR".to_string()))
    }

    async fn list(
        &self,
        request: tonic::Request<discovery::ListRequest>,
    ) -> std::result::Result<tonic::Response<discovery::ListResponse>, tonic::Status> {
        tracing::info!("{:#?}", request);
        tracing::info!("{:#?}", request.remote_addr());
        Err(tonic::Status::ok("ERROR".to_string()))
    }

    async fn affiliate_update(
        &self,
        request: tonic::Request<discovery::AffiliateUpdateRequest>,
    ) -> std::result::Result<tonic::Response<discovery::AffiliateUpdateResponse>, tonic::Status>
    {
        tracing::info!("{:#?}", request);
        tracing::info!("{:#?}", request.remote_addr());
        Err(tonic::Status::ok("ERROR".to_string()))
    }

    async fn affiliate_delete(
        &self,
        request: tonic::Request<discovery::AffiliateDeleteRequest>,
    ) -> std::result::Result<tonic::Response<discovery::AffiliateDeleteResponse>, tonic::Status>
    {
        tracing::info!("{:#?}", request);
        tracing::info!("{:#?}", request.remote_addr());
        Err(tonic::Status::ok("ERROR".to_string()))
    }

    async fn watch(
        &self,
        request: tonic::Request<discovery::WatchRequest>,
    ) -> std::result::Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        tracing::info!("{:#?}", request);
        tracing::info!("{:#?}", request.remote_addr());
        Err(tonic::Status::ok("ERROR".to_string()))
    }
}
