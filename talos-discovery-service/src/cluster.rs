use std::time::SystemTime;

use tokio::sync::mpsc::{self, Receiver};
use tonic::Status;
use tracing::error;

use crate::discovery::{self, WatchResponse};

pub(crate) type ClusterId = String;
type AffiliateId = String;

pub(crate) struct TalosCluster {
    _id: ClusterId,
    affiliates: Vec<Affiliate>,
    watch_broadcaster: tokio::sync::broadcast::Sender<WatchResponse>,
}

#[derive(Clone)]
struct Affiliate {
    id: AffiliateId, // part of 'message Affiliate'
    _expiration: SystemTime,
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
    pub async fn new_cluster_watcher(&self) -> Receiver<Result<WatchResponse, Status>> {
        let mut rx = self.watch_broadcaster.subscribe();
        let (tx, rx_stream) = mpsc::channel(128);

        let snapshot = self.get_affiliate_snapshot().await;
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

    async fn get_affiliate_snapshot(&self) -> WatchResponse {
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

    pub async fn _broadcast_cluster_snapshot(&self) {
        let snapshot = self.get_affiliate_snapshot().await;

        let _ = self
            .watch_broadcaster
            .send(snapshot)
            .inspect_err(|err| error!("{}", err));
    }
}
