use chrono::{DateTime, Timelike, Utc};
use std::{
    collections::HashMap,
    fmt,
    num::TryFromIntError,
    time::{Duration, SystemTime},
};
use tokio::sync::{
    broadcast::Sender,
    mpsc::{self, Receiver},
};
use tonic::Status;
use tracing::{debug, error, info};

use crate::discovery::{self, AffiliateUpdateRequest, WatchResponse};

pub(crate) type ClusterId = String;
type AffiliateId = String;

pub(crate) struct TalosCluster {
    id: ClusterId,
    affiliates: HashMap<AffiliateId, Affiliate>,
    watch_broadcaster: Sender<WatchResponse>,
}

#[derive(Clone)]
pub(crate) struct Affiliate {
    // part of gRPC message Affiliate
    id: AffiliateId,
    // part of gRPC message Affiliate
    data: Vec<u8>,
    // part of gRPC message Affiliate
    endpoints: Vec<Vec<u8>>,
    expiration: SystemTime,
}

impl From<Affiliate> for discovery::Affiliate {
    fn from(val: Affiliate) -> Self {
        discovery::Affiliate {
            id: val.id,
            data: val.data,
            endpoints: val.endpoints,
        }
    }
}

impl From<Vec<&Affiliate>> for discovery::WatchResponse {
    fn from(val: Vec<&Affiliate>) -> Self {
        Self {
            affiliates: val
                .into_iter()
                .cloned()
                .map(Affiliate::into)
                .collect::<Vec<discovery::Affiliate>>(),
            deleted: false,
        }
    }
}

impl TalosCluster {
    const BUFFER_SIZE: usize = 64;

    pub fn new(cluster_id: ClusterId) -> TalosCluster {
        TalosCluster {
            id: cluster_id,
            affiliates: HashMap::new(),
            watch_broadcaster: Sender::new(Self::BUFFER_SIZE),
        }
    }

    pub async fn subscribe(&self) -> Receiver<Result<WatchResponse, Status>> {
        let mut rx = self.watch_broadcaster.subscribe();
        let (tx, rx_stream) = mpsc::channel(Self::BUFFER_SIZE);

        let cluster_snapshot = self.get_affiliates().await;
        let watch_response: WatchResponse = cluster_snapshot.into();
        let _ = tx.send(Ok(watch_response)).await.inspect_err(|err| error!("{}", err));

        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Err(err) = tx.send(Ok(msg)).await {
                    debug!("{}", err);
                    break;
                }
            }
        });

        rx_stream
    }

    pub async fn broadcast_affiliate_states(&self) {
        let affiliate_states = self.get_affiliates().await;
        self.send_affiliate_update(affiliate_states.into()).await;
    }

    async fn broadcast_deleted_affiliates(&self, expired: HashMap<AffiliateId, Affiliate>) {
        if expired.is_empty() {
            return;
        }
        let deleted_affiliates = expired
            .into_values()
            .map(Affiliate::into)
            .collect::<Vec<discovery::Affiliate>>();

        let response = WatchResponse {
            affiliates: deleted_affiliates,
            deleted: true,
        };

        self.send_affiliate_update(response).await;
    }

    async fn send_affiliate_update(&self, response: WatchResponse) {
        if self.watch_broadcaster.receiver_count() == 0 {
            return;
        }
        let _ = self
            .watch_broadcaster
            .send(response)
            .inspect_err(|err| error!("{}", err));
    }

    pub fn has_affiliates(&self) -> bool {
        self.affiliates.is_empty()
    }

    pub async fn add_affiliate(&mut self, request: &AffiliateUpdateRequest) -> Result<(), Status> {
        let ttl = request
            .ttl
            .ok_or(Status::invalid_argument("Invalid TTL"))
            .inspect_err(|err| error!("{}", err.to_string()))?;

        let ttl = Duration::new(
            ttl.seconds
                .try_into()
                .map_err(|err: TryFromIntError| Status::invalid_argument(err.to_string()))
                .inspect_err(|err| error!("{}", err.to_string()))?,
            ttl.nanos
                .try_into()
                .map_err(|err: TryFromIntError| Status::invalid_argument(err.to_string()))
                .inspect_err(|err| error!("{}", err.to_string()))?,
        );

        let affiliate = Affiliate {
            id: request.affiliate_id.clone(),
            expiration: SystemTime::now() + ttl,
            endpoints: request.affiliate_endpoints.clone(),
            data: request.affiliate_data().to_vec(),
        };

        self.affiliates.insert(affiliate.id.clone(), affiliate);

        info!("Added affiliate: {}", request.affiliate_id,);
        info!("Number of affiliates: {}", self.affiliates.len());

        self.broadcast_affiliate_states().await;

        Ok(())
    }

    pub(crate) async fn get_affiliates(&self) -> Vec<&Affiliate> {
        self.affiliates.values().collect()
    }

    pub async fn get_affiliate(&self, affiliate_id: &AffiliateId) -> Option<&Affiliate> {
        self.affiliates.get(affiliate_id)
    }

    pub async fn delete_affiliate(&mut self, affiliate_id: &AffiliateId) -> Option<Affiliate> {
        debug!("Removing affiliate ID {}", affiliate_id);
        self.affiliates.remove(affiliate_id)
    }

    pub async fn run_gc(&mut self) {
        let expired = self
            .affiliates
            .clone()
            .into_iter()
            .filter(|(_, a)| SystemTime::now() > a.expiration)
            .collect::<HashMap<_, _>>();

        for exp in expired.values() {
            self.delete_affiliate(&exp.id).await;
        }

        info!(
            "GC for cluster {}: Removed {} affiliates. Remaining: {}",
            self.id,
            expired.len(),
            self.affiliates.len()
        );

        self.broadcast_deleted_affiliates(expired).await;
    }
}

impl fmt::Display for TalosCluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = write!(f, "{{ Cluster: {}", self.id);

        for affiliate in self.affiliates.values() {
            let _ = write!(f, ", Affiliate id: {}", affiliate.id);

            let _ = write!(
                f,
                ", Expiration: {}",
                DateTime::<Utc>::from(affiliate.expiration).with_nanosecond(0).unwrap()
            );

            let encrypted_data = {
                let mut data = &affiliate.data[..];

                if data.len() > 4 {
                    data = &data[..4];
                }

                format!(
                    "{}..",
                    data.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
                )
            };
            let _ = write!(f, ", Encrypted data: {}", encrypted_data);

            for endpoint in &affiliate.endpoints {
                let encrypted_endpoint = {
                    let mut endpoints = &endpoint[..];

                    if endpoints.len() > 4 {
                        endpoints = &endpoints[..4];
                    }

                    format!(
                        "{}..",
                        endpoints
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<Vec<_>>()
                            .join("")
                    )
                };
                let _ = write!(f, ", Encrypted Endpoint: {}", encrypted_endpoint);
            }
        }

        write!(f, "}}")
    }
}
