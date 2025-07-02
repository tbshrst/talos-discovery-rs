use discovery_api::{
    self,
    cluster_server::Cluster,
    tonic::{async_trait, Request, Response, Status},
    AffiliateDeleteRequest, AffiliateDeleteResponse, AffiliateUpdateRequest, AffiliateUpdateResponse, HelloRequest,
    HelloResponse, ListRequest, ListResponse, WatchRequest, WatchResponse,
};
use std::{collections::HashMap, net::IpAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::io::AsyncWriteExt;
use tokio::{
    fs::{File, OpenOptions},
    sync::Mutex,
    time,
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info};

use crate::cluster::{Affiliate, ClusterId, TalosCluster};

#[derive(Clone)]
pub(crate) struct DiscoveryService {
    clusters: Arc<Mutex<HashMap<ClusterId, TalosCluster>>>,
    gc_interval: Duration,
    backup_path: Option<PathBuf>,
    backup_interval: Duration,
}

impl DiscoveryService {
    const BACKUP_FILE_NAME: &str = "discovery_service_backup.json";

    pub async fn new(gc_interval: u16, backup_path: Option<String>, backup_interval: u16) -> anyhow::Result<Self> {
        let backup_path = backup_path.map(|path| PathBuf::from(path).join(Self::BACKUP_FILE_NAME));

        let new = Self {
            clusters: Arc::new(Mutex::new(HashMap::new())),
            gc_interval: Duration::from_secs(gc_interval.into()),
            backup_path,
            backup_interval: Duration::from_secs(backup_interval.into()),
        };

        new.import_backup().await?;

        new.run_backup_loop().await;
        new.run_gc_loop().await;

        Ok(new)
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

    async fn run_backup_loop(&self) {
        if self.backup_path.is_none() {
            debug!("Backups deactivated");
            return;
        }

        let self_clone = self.clone();
        tokio::task::spawn(async move {
            let mut backup_interval = time::interval(self_clone.backup_interval);

            info!("Backup loop started");
            loop {
                backup_interval.tick().await;
                if let Err(err) = self_clone.export_backup().await {
                    error!("couldn't save backup: {}", err.to_string());
                    error!("stopping backup loop");
                    return;
                }
            }
        });
    }

    async fn export_backup(&self) -> anyhow::Result<()> {
        debug!("export_backup");

        let backup_path = {
            match &self.backup_path {
                Some(backup_path) => backup_path.as_path(),
                None => return Ok(()),
            }
        };
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(backup_path)
            .await?;

        let svc_clusters = self.clusters.lock().await;
        let svc_clusters = svc_clusters.values().collect::<Vec<_>>();

        let json = serde_json::to_string(&svc_clusters)?;
        file.write_all(json.as_bytes()).await?;
        file.write_u8(b'\n').await?;

        debug!("{} clusters backed up", svc_clusters.len());

        Ok(())
    }

    async fn import_backup(&self) -> anyhow::Result<()> {
        debug!("import_backup");

        let backup_path = {
            match &self.backup_path {
                Some(backup_path) if backup_path.exists() => backup_path.as_path(),
                _ => return Ok(()),
            }
        };
        let file = File::open(backup_path).await?.into_std().await;
        let reader = std::io::BufReader::new(file);

        let clusters: Vec<TalosCluster> = serde_json::from_reader(reader)?;
        info!("{} clusters restored", clusters.len());

        let mut svc_clusters = self.clusters.lock().await;
        for cluster in clusters {
            svc_clusters.insert(cluster.id.clone(), cluster);
        }

        Ok(())
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

#[async_trait]
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
            .ok_or(Status::not_found(format!("Cluster ID {cluster_id} not found")))
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
            .ok_or(Status::not_found(format!("Cluster ID {cluster_id} not found")))
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
