use std::{fmt, time::SystemTime};

use chrono::{DateTime, Utc};
use tokio::sync::{
    broadcast,
    mpsc::{self, Receiver},
};
use tonic::Status;
use tracing::error;

use crate::discovery::{self, WatchResponse};

pub(crate) type ClusterId = String;
type AffiliateId = String;

pub(crate) struct TalosCluster {
    id: ClusterId,
    affiliates: Vec<Affiliate>,
    watch_broadcaster: tokio::sync::broadcast::Sender<WatchResponse>,
}

#[derive(Clone)]
struct Affiliate {
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
    pub async fn _new(cluster_id: ClusterId) -> Self {
        let (tx, _) = broadcast::channel(128);

        Self {
            id: cluster_id,
            affiliates: vec![],
            watch_broadcaster: tx,
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

    pub async fn _broadcast_affiliate_states(&self) {
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
            .into_iter()
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

        for affiliate in &self.affiliates {
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
