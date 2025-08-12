use std::sync::{mpsc, Arc};
use std::sync::atomic::Ordering;
use log::{debug, warn};
use tokio::sync::broadcast;
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use pasture_core::containers::{BorrowedBuffer, VectorBuffer};
use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use lidarserv_server::{
    index::query::Query,
    net::client::viewer::{PartialResult, QueryConfig, ViewerClient,NodeUpdate},
};

pub async fn send_frustum_thread(
    args: AppOptions,
    frustum_data_rx: mpsc::Receiver<ViewFrustumQuery>, //receiver to get the View Frustum Query for each image
    point_data_tx: mpsc::Sender<VectorBuffer>, //sender to send the points to the viewer.
    //point_data_complete_tx: mpsc::Sender<>,
    status: Arc<Status>,
)-> anyhow::Result<()> {
    debug!("Send Frustum Thread: Started");
    // connect to viewerClient
    let (_shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
    let mut client =
        ViewerClient::connect((args.host.as_str(), args.port), &mut shutdown_rx).await?;
    //loop to wait for new frustums to query.
    loop {
        let frustum = frustum_data_rx.recv()?;
        let frustum_query = Query::ViewFrustum(frustum);
        //todo! wait for reader before next query is sent.
        //send query
        client
            .write
            .query_oneshot(
                frustum_query,
                &QueryConfig {
                    point_filtering: false,
            },
            ).await?;
        //loop to receive all the parts of the view frustum query.
        loop{
            let update:PartialResult<VectorBuffer> = client
                .read
                .receive_update_global_coordinates(&mut shutdown_rx)
                .await?;


            match update {
                PartialResult::DeleteNode(_) => warn!("Received unexpected DeleteNode message."),
                PartialResult::UpdateNode(update) => {
                    status
                        .frustum_query_received_points
                        .fetch_add(update.points.len() as u64, Ordering::Relaxed);
                    status.frustum_query_received_nodes.fetch_add(1, Ordering::Relaxed);
                    point_data_tx.send(update.points)?
                }
                PartialResult::Complete => {

                    debug!("Received Complete message.");
                    break;
                },
            }
            //todo: send mark done.
        }



        //todo: send data.
    }
    debug!("Send Frustum Thread: finished");
}