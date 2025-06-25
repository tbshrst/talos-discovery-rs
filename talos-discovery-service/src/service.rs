use std::{collections::HashMap, net::IpAddr, sync::Arc, time::Duration};

use tokio::{sync::Mutex, time};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use crate::{
    cluster::{ClusterId, TalosCluster},
    discovery::{
        cluster_server::Cluster, AffiliateDeleteRequest, AffiliateDeleteResponse, AffiliateUpdateRequest,
        AffiliateUpdateResponse, HelloRequest, HelloResponse, ListRequest, ListResponse, WatchRequest, WatchResponse,
    },
};

#[derive(Clone)]
pub(crate) struct DiscoveryService {
    clusters: Arc<Mutex<HashMap<ClusterId, TalosCluster>>>,
    gc_interval: Duration,
}

impl DiscoveryService {
    pub async fn new(gc_interval: u16) -> Self {
        let new = Self {
            clusters: Arc::new(Mutex::new(HashMap::new())),
            gc_interval: Duration::from_secs(gc_interval.into()),
        };

        new.run_gc_loop().await;

        new
    }

    async fn get_cluster<'a>(
        &self,
        clusters: &'a mut HashMap<ClusterId, TalosCluster>,
        cluster_id: ClusterId,
    ) -> Option<&'a mut TalosCluster> {
        clusters.get_mut(&cluster_id)
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

        let mut clusters = self.clusters.lock().await;
        clusters.values_mut().for_each(|cluster| cluster.run_gc());

        let before_len = clusters.len();
        clusters.retain(|_, cluster| !cluster.is_empty());

        info!(
            "GC clusters, removed clusters: {}, remaining clusters: {}",
            before_len - clusters.len(),
            clusters.len()
        );

        clusters
            .iter()
            .for_each(|(_, cluster)| debug!("{}", cluster.to_string()));
    }
}

#[tonic::async_trait]
impl Cluster for DiscoveryService {
    type WatchStream = ReceiverStream<Result<WatchResponse, Status>>;

    async fn hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloResponse>, Status> {
        info!("cluster node request: Hello ({})", request.remote_addr().unwrap().ip());
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

    async fn watch(&self, request: Request<WatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        info!("cluster node request: Watch ({})", request.remote_addr().unwrap().ip());
        debug!("{:?}", request);

        let request = request.into_inner();
        let cluster_id = request.cluster_id;

        let mut clusters = self.clusters.lock().await;
        let cluster = self
            .get_cluster(&mut clusters, cluster_id.clone())
            .await
            .ok_or(Status::not_found(format!("cluster with ID {} not found", cluster_id)))
            .inspect_err(|err| error!("{}", err.to_string()))?;

        let watch_stream = cluster.subscribe().await;

        Ok(Response::new(ReceiverStream::new(watch_stream)))
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
        let mut clusters = self.clusters.lock().await;
        let request = request.into_inner();

        if let Some(cluster) = clusters.get_mut(&request.cluster_id) {
            return cluster.add_affiliate(&request).await;
        }
        info!("Creating new cluster with id {}", request.cluster_id.clone());
        let mut cluster = TalosCluster::new(request.cluster_id.clone());
        let res = cluster.add_affiliate(&request).await;
        clusters.insert(request.cluster_id.clone(), cluster);

        res
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

        let request = request.into_inner();
        let cluster_id = request.cluster_id;
        let affiliate_id = request.affiliate_id;

        let mut clusters = self.clusters.lock().await;
        let cluster = self
            .get_cluster(&mut clusters, cluster_id.clone())
            .await
            .ok_or(Status::not_found(format!("cluster with ID {} not found", cluster_id)))
            .inspect_err(|err| error!("{}", err.to_string()))?;

        match cluster.get_affiliate(&affiliate_id).await {
            Some(_) => {
                cluster.delete_affiliate(&affiliate_id).await;
                cluster.broadcast_affiliate_states().await;

                info!("deleted affiliate {} from cluster {}", affiliate_id, cluster_id);
            }
            None => debug!("affiliate {} doesn't exist in cluster {}", affiliate_id, cluster_id),
        }

        Ok(Response::new(AffiliateDeleteResponse {}))
    }

    async fn list(&self, request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        info!("cluster node request: List ({})", request.remote_addr().unwrap().ip());
        debug!("{:?}", request);
        let mut _clusters = self.clusters.lock().await;
        unimplemented!();
    }
}
