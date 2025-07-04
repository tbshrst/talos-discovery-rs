use chrono::{DateTime, Timelike, Utc};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
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
use tracing::{debug, error, info};

use discovery_api::{self, tonic::Status, AffiliateUpdateRequest, WatchResponse};

pub(crate) type ClusterId = String;
type AffiliateId = String;

pub(crate) struct TalosCluster {
    pub(crate) id: ClusterId,
    affiliates: HashMap<AffiliateId, Affiliate>,
    watch_broadcaster: Sender<WatchResponse>,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct Affiliate {
    // part of gRPC message Affiliate
    id: AffiliateId,
    // part of gRPC message Affiliate
    data: Vec<u8>,
    // part of gRPC message Affiliate
    endpoints: Vec<Vec<u8>>,
    expiration: SystemTime,
}

impl From<Affiliate> for discovery_api::Affiliate {
    fn from(val: Affiliate) -> Self {
        discovery_api::Affiliate {
            id: val.id,
            data: val.data,
            endpoints: val.endpoints,
        }
    }
}

impl TalosCluster {
    const BUFFER_SIZE: usize = 64;
    // XXX: custom extension
    pub const MAX_IDENTIFIER_LENGTH: usize = 256;
    // XXX: custom extension
    pub const MAX_PAYLOAD_LENGTH: usize = 512 * 1024;
    // XXX: custom extension
    pub const MAX_TTL_DURATION: Duration = Duration::from_secs(2 * 60 * 60); // 2 hours

    pub fn new(cluster_id: ClusterId) -> TalosCluster {
        TalosCluster {
            id: cluster_id,
            affiliates: HashMap::new(),
            watch_broadcaster: Sender::new(Self::BUFFER_SIZE),
        }
    }

    pub async fn convert_watch_response(&self, affiliates: Vec<&Affiliate>) -> discovery_api::WatchResponse {
        discovery_api::WatchResponse {
            affiliates: affiliates
                .into_iter()
                .cloned()
                .map(Affiliate::into)
                .collect::<Vec<discovery_api::Affiliate>>(),
            deleted: false,
        }
    }

    pub async fn subscribe(&self) -> Receiver<Result<WatchResponse, Status>> {
        let mut rx = self.watch_broadcaster.subscribe();
        let (tx, rx_stream) = mpsc::channel(Self::BUFFER_SIZE);

        let cluster_snapshot = self.get_affiliates().await;
        let watch_response = self.convert_watch_response(cluster_snapshot).await;
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
        self.send_affiliate_update(self.convert_watch_response(affiliate_states).await)
            .await;
    }

    async fn broadcast_deleted_affiliates(&self, expired: HashMap<AffiliateId, Affiliate>) {
        if expired.is_empty() {
            return;
        }
        let deleted_affiliates = expired
            .into_values()
            .map(Affiliate::into)
            .collect::<Vec<discovery_api::Affiliate>>();

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
        debug!("Removing affiliate ID {} from Cluster ID {}", affiliate_id, self.id);
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

impl Serialize for TalosCluster {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("TalosCluster", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("affiliates", &self.affiliates)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TalosCluster {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SerdeTalosCluster {
            id: ClusterId,
            affiliates: HashMap<AffiliateId, Affiliate>,
        }

        let helper = SerdeTalosCluster::deserialize(deserializer)?;

        Ok(Self {
            id: helper.id,
            affiliates: helper.affiliates,
            watch_broadcaster: Sender::new(Self::BUFFER_SIZE),
        })
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
                    data.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join("")
                )
            };
            let _ = write!(f, ", Encrypted data: {encrypted_data}");

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
                            .map(|b| format!("{b:02x}"))
                            .collect::<Vec<_>>()
                            .join("")
                    )
                };
                let _ = write!(f, ", Encrypted Endpoint: {encrypted_endpoint}");
            }
        }

        write!(f, "}}")
    }
}
