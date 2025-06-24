use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use tokio::{
    sync::{mpsc, Mutex},
    time,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::discovery::{
    cluster_server::{Cluster, ClusterServer},
    AffiliateDeleteRequest, AffiliateDeleteResponse, AffiliateUpdateRequest,
    AffiliateUpdateResponse, HelloRequest, HelloResponse, ListRequest, ListResponse, WatchRequest,
    WatchResponse,
};

pub mod discovery {
    tonic::include_proto!("sidero.discovery.server");
}

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

type ClusterId = String;
type AffiliateId = String;

#[derive(Clone)]
pub struct DiscoveryService {
    clusters: Arc<Mutex<HashMap<ClusterId, TalosCluster>>>,
    gc_interval: Duration,
}

struct TalosCluster {
    _id: ClusterId,
    _affiliates: Vec<Affilliates>,
    watch_broadcaster: tokio::sync::broadcast::Sender<WatchResponse>,
}

struct Affilliates {
    _id: AffiliateId,
    _expiration: SystemTime,
}

impl DiscoveryService {
    async fn new() -> Self {
        let new = Self {
            clusters: Arc::new(Mutex::new(HashMap::new())),
            gc_interval: Duration::from_secs(60),
        };

        new.run_gc_loop().await;

        new
    }

    async fn run_gc_loop(&self) {
        let self_clone = self.clone();

        tokio::task::spawn(async move {
            let mut gc_interval = time::interval(self_clone.gc_interval);

            info!("garbage collector started");
            loop {
                gc_interval.tick().await;
                self_clone.run_gc().await;
            }
        });
    }

    async fn run_gc(&self) {
        debug!("run_gc");
        let mut _clusters = self.clusters.lock().await;
    }
}

#[tonic::async_trait]
impl Cluster for DiscoveryService {
    type WatchStream = ReceiverStream<Result<WatchResponse, Status>>;

    async fn hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        info!(
            "cluster node request: Hello ({})",
            request.remote_addr().unwrap().ip()
        );
        debug!("{:?}", request);

        let socket = request
            .remote_addr()
            .ok_or(Status::invalid_argument("couldn't parse IP address"))
            .inspect_err(|err| error!("{}", err.to_string()))?;

        let ip = match socket.ip() {
            IpAddr::V4(ipv4) => ipv4.octets().to_vec(),
            IpAddr::V6(ipv6) => ipv6.octets().to_vec(),
        };

        Ok(Response::new(HelloResponse {
            redirect: None,
            client_ip: ip,
        }))
    }

    async fn watch(
        &self,
        request: Request<WatchRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        info!(
            "cluster node request: Watch ({})",
            request.remote_addr().unwrap().ip()
        );
        debug!("{:?}", request);

        let request = request.into_inner();
        let clusters = self.clusters.lock().await;
        let cluster_id = request.cluster_id;

        let cluster = clusters
            .get(&cluster_id)
            .ok_or(Status::not_found(format!(
                "cluster with ID {} not found",
                cluster_id
            )))
            .inspect_err(|err| error!("{}", err.to_string()))?;

        let mut rx = cluster.watch_broadcaster.subscribe();
        let (tx, rx_stream) = mpsc::channel(16);

        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if tx.send(Ok(msg)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx_stream)))
    }

    async fn affiliate_update(
        &self,
        request: Request<AffiliateUpdateRequest>,
    ) -> std::result::Result<Response<AffiliateUpdateResponse>, Status> {
        info!(
            "cluster node request: AffiliateUpdate ({})",
            request.remote_addr().unwrap().ip()
        );
        debug!("{:?}", request);
        let mut _clusters = self.clusters.lock().await;
        unimplemented!();
    }

    async fn affiliate_delete(
        &self,
        request: Request<AffiliateDeleteRequest>,
    ) -> Result<Response<AffiliateDeleteResponse>, Status> {
        info!(
            "cluster node request: AffilliateDelete ({})",
            request.remote_addr().unwrap().ip()
        );
        debug!("{:?}", request);
        let mut _clusters = self.clusters.lock().await;
        unimplemented!();
    }

    async fn list(&self, request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        info!(
            "cluster node request: List ({})",
            request.remote_addr().unwrap().ip()
        );
        debug!("{:?}", request);
        let mut _clusters = self.clusters.lock().await;
        unimplemented!();
    }
}
