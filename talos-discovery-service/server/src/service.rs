use std::{collections::HashMap, net::IpAddr, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use crate::cluster::{Affiliate, ClusterId, TalosCluster};
use discovery_api::{
    self, cluster_server::Cluster, AffiliateDeleteRequest, AffiliateDeleteResponse, AffiliateUpdateRequest,
    AffiliateUpdateResponse, HelloRequest, HelloResponse, ListRequest, ListResponse, WatchRequest, WatchResponse,
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

    async fn get_or_create_cluster<'a>(
        &self,
        clusters: &'a mut HashMap<ClusterId, TalosCluster>,
        cluster_id: ClusterId,
    ) -> &'a mut TalosCluster {
        if clusters.contains_key(&cluster_id) {
            return self.get_cluster(clusters, cluster_id).await.unwrap();
        }
        info!("Creating new cluster with ID {}", cluster_id.clone());
        clusters.insert(cluster_id.clone(), TalosCluster::new(cluster_id.clone()));
        self.get_cluster(clusters, cluster_id).await.unwrap()
    }

    async fn run_gc_loop(&self) {
        let self_clone = self.clone();

        tokio::task::spawn(async move {
            let mut gc_interval = time::interval(self_clone.gc_interval);

            info!("Garbage collector started");
            loop {
                gc_interval.tick().await;
                self_clone.run_gc().await;
            }
        });
    }

    async fn run_gc(&self) {
        debug!("run_gc");

        let mut clusters = self.clusters.lock().await;
        for cluster in clusters.values_mut() {
            cluster.run_gc().await;
        }

        let before_len = clusters.len();
        clusters.retain(|_, cluster| !cluster.has_affiliates());

        info!(
            "GC clusters, removed clusters: {}, remaining clusters: {}",
            before_len - clusters.len(),
            clusters.len()
        );

        clusters
            .iter()
            .for_each(|(_, cluster)| debug!("{}", cluster.to_string()));
    }

    async fn update_clusters(
        &self,
        request: AffiliateUpdateRequest,
    ) -> Result<Response<AffiliateUpdateResponse>, Status> {
        let mut clusters = self.clusters.lock().await;
        let cluster_id = request.cluster_id.clone();
        match clusters.get_mut(&cluster_id) {
            Some(existing_cluster) => existing_cluster.add_affiliate(&request).await?,
            None => {
                info!("Creating new cluster with ID {}", cluster_id.clone());
                let mut cluster = TalosCluster::new(cluster_id.clone());
                cluster.add_affiliate(&request).await?;
                clusters.insert(cluster_id, cluster);
            }
        };
        Ok(Response::new(AffiliateUpdateResponse {}))
    }
}

#[tonic::async_trait]
impl Cluster for DiscoveryService {
    type WatchStream = ReceiverStream<Result<WatchResponse, Status>>;

    async fn hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloResponse>, Status> {
        info!("Cluster node request: Hello ({})", request.remote_addr().unwrap().ip());

        let socket = request
            .remote_addr()
            .ok_or(Status::invalid_argument("Couldn't parse IP address"))
            .inspect_err(|err| debug!("{}", err.to_string()))?;

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
        info!("Cluster node request: Watch ({})", request.remote_addr().unwrap().ip());

        let request = request.into_inner();
        let cluster_id = request.cluster_id;

        // XXX: custom extension
        if cluster_id.len() > TalosCluster::MAX_IDENTIFIER_LENGTH {
            return Err(Status::invalid_argument("maximum identifier length exceeded"));
        }

        let mut clusters = self.clusters.lock().await;
        let cluster = self.get_or_create_cluster(&mut clusters, cluster_id.clone()).await;

        let watch_stream = cluster.subscribe().await;

        Ok(Response::new(ReceiverStream::new(watch_stream)))
    }

    async fn affiliate_update(
        &self,
        request: Request<AffiliateUpdateRequest>,
    ) -> Result<Response<AffiliateUpdateResponse>, Status> {
        info!(
            "Cluster node request: AffiliateUpdate ({})",
            request.remote_addr().unwrap().ip()
        );

        let request = request.into_inner();

        // XXX: custom extension
        if request.cluster_id.len() > TalosCluster::MAX_IDENTIFIER_LENGTH
            || request.affiliate_id.len() > TalosCluster::MAX_IDENTIFIER_LENGTH
        {
            return Err(Status::invalid_argument("maximum identifier length exceeded"));
        }

        // XXX: custom extension
        if let Some(affiliate_data) = &request.affiliate_data {
            if affiliate_data.len() > TalosCluster::MAX_PAYLOAD_LENGTH {
                return Err(Status::invalid_argument("maximum payload length exceeded"));
            }
        }

        // XXX: custom extension
        for endpoint in &request.affiliate_endpoints {
            if endpoint.len() > TalosCluster::MAX_PAYLOAD_LENGTH {
                return Err(Status::invalid_argument("maximum payload length exceeded"));
            }
        }

        // XXX: custom extension
        if let Some(ttl) = &request.ttl {
            if ttl.seconds <= 0 || ttl.seconds as u64 > TalosCluster::MAX_TTL_DURATION.as_secs() {
                return Err(Status::invalid_argument("maximum TTL exceeded"));
            }
        }

        self.update_clusters(request).await
    }

    async fn affiliate_delete(
        &self,
        request: Request<AffiliateDeleteRequest>,
    ) -> Result<Response<AffiliateDeleteResponse>, Status> {
        info!(
            "Cluster node request: AffilliateDelete ({})",
            request.remote_addr().unwrap().ip()
        );

        let request = request.into_inner();
        let cluster_id = request.cluster_id;
        let affiliate_id = request.affiliate_id;

        let mut clusters = self.clusters.lock().await;
        let cluster = self
            .get_cluster(&mut clusters, cluster_id.clone())
            .await
            .ok_or(Status::not_found(format!("Cluster ID {} not found", cluster_id)))
            .inspect_err(|err| error!("{}", err.to_string()))?;

        match cluster.get_affiliate(&affiliate_id).await {
            Some(_) => {
                cluster.delete_affiliate(&affiliate_id).await;
                cluster.broadcast_affiliate_states().await;

                info!("Deleted affiliate ID {} from cluster {}", affiliate_id, cluster_id);
            }
            None => debug!("Affiliate ID {} doesn't exist in cluster {}", affiliate_id, cluster_id),
        }

        Ok(Response::new(AffiliateDeleteResponse {}))
    }

    async fn list(&self, request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        info!("Cluster node request: List ({})", request.remote_addr().unwrap().ip());

        let request = request.into_inner();
        let cluster_id = request.cluster_id;

        let mut clusters = self.clusters.lock().await;
        let cluster = self
            .get_cluster(&mut clusters, cluster_id.clone())
            .await
            .ok_or(Status::not_found(format!("Cluster ID {} not found", cluster_id)))
            .inspect_err(|err| error!("{}", err.to_string()))?;

        let affiliates = cluster
            .get_affiliates()
            .await
            .into_iter()
            .cloned()
            .map(Affiliate::into)
            .collect::<Vec<discovery_api::Affiliate>>();

        Ok(Response::new(ListResponse { affiliates }))
    }
}
