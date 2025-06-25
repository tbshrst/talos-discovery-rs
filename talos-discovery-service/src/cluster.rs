use chrono::{DateTime, Utc};
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

use tonic::{Response, Status};
use tracing::{error, info};

use crate::discovery::{self, AffiliateUpdateRequest, AffiliateUpdateResponse, WatchResponse};

pub(crate) type ClusterId = String;
type AffiliateId = String;

pub(crate) struct TalosCluster {
    id: ClusterId,
    affiliates: HashMap<AffiliateId, Affiliate>,
    watch_broadcaster: tokio::sync::broadcast::Sender<WatchResponse>,
}

#[derive(Clone)]
pub(crate) struct Affiliate {
    id: AffiliateId, // part of 'message Affiliate'
    expiration: SystemTime,
    data: Vec<u8>,           // part of 'message Affiliate'
    endpoints: Vec<Vec<u8>>, // part of 'message Affiliate'
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

impl TalosCluster {
    pub fn new(cluster_id: ClusterId) -> TalosCluster {
        TalosCluster {
            id: cluster_id,
            affiliates: HashMap::new(),
            watch_broadcaster: Sender::new(16),
        }
    }
    pub async fn subscribe(&self) -> Receiver<Result<WatchResponse, Status>> {
        let mut rx = self.watch_broadcaster.subscribe();
        let (tx, rx_stream) = mpsc::channel(128);

        let snapshot: WatchResponse = self.get_affiliates().await;
        let _ = tx
            .send(Ok(snapshot))
            .await
            .inspect_err(|err| error!("{}", err));

        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Err(err) = tx.send(Ok(msg)).await {
                    error!("{}", err);
                    break;
                }
            }
        });

        rx_stream
    }

    pub async fn add_affiliate(
        &mut self,
        request: &AffiliateUpdateRequest,
    ) -> Result<Response<AffiliateUpdateResponse>, Status> {
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
        let affiliate_id = affiliate.id.clone();
        self.affiliates.insert(affiliate.id.clone(), affiliate);
        info!(
            "Added affiliate: {}\nNumber of affiliates: {}",
            affiliate_id,
            self.affiliates.len()
        );

        self.broadcast_affiliate_states().await;
        Ok(Response::new(AffiliateUpdateResponse {}))
    }

    pub async fn broadcast_affiliate_states(&self) {
        let snapshot = self.get_affiliates().await;

        let _ = self
            .watch_broadcaster
            .send(snapshot)
            .inspect_err(|err| error!("{}", err));
    }

    async fn get_affiliates(&self) -> WatchResponse {
        let affiliates = self
            .affiliates
            .clone()
            .into_values()
            .map(Affiliate::into)
            .collect::<Vec<discovery::Affiliate>>();

        WatchResponse {
            affiliates,
            deleted: false,
        }
    }
}

impl fmt::Display for TalosCluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = write!(f, "Cluster: {}", self.id);

        for affiliate in self.affiliates.values() {
            let _ = write!(f, "\taffiliate id: {}", affiliate.id);

            let _ = write!(
                f,
                "\t\texpiration id: {}",
                DateTime::<Utc>::from(affiliate.expiration).to_rfc3339()
            );

            let encrypted_data = {
                let mut data = &affiliate.data[..];

                if data.len() > 64 {
                    data = &data[..64];
                }

                format!("{}..", str::from_utf8(data).unwrap())
            };
            let _ = write!(f, "\t\tencrypted data: {}", encrypted_data);

            for endpoint in &affiliate.endpoints {
                let encrypted_endpoint = {
                    let mut endpoints = &endpoint[..];

                    if endpoints.len() > 64 {
                        endpoints = &endpoints[..64];
                    }

                    format!("{}..", str::from_utf8(endpoints).unwrap())
                };
                let _ = write!(f, "\t\tencrypted endpoitn: {}", encrypted_endpoint);
            }
        }

        write!(f, "")
    }
}
