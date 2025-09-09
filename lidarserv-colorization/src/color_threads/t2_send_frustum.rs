use crate::cli::AppOptions;
use crate::color_threads::cross_thread_functionality::check_stop_lidarserv_colorization;
use crate::color_threads::status::Status;
use crate::color_threads::{ImageIdAndFrustum, ImageIdAndVectorBuffer};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use lidarserv_server::{
    index::query::Query,
    net::client::viewer::{NodeUpdate, PartialResult, QueryConfig, ViewerClient},
};
use log::{debug, warn};
use pasture_core::containers::{BorrowedBuffer, MakeBufferFromLayout, VectorBuffer};
use pasture_core::layout::PointLayout;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio_util::sync::CancellationToken;

pub async fn thread_2_send_frustum(
    args: AppOptions,
    stop_token: CancellationToken,
    image_id_and_frustum_data_rx: mpsc::Receiver<ImageIdAndFrustum>, //Receiver to get the View Frustum Query for each image
    point_data_tx: mpsc::Sender<ImageIdAndVectorBuffer>, //Sender to send the points to the viewer.
    //point_data_complete_tx: mpsc::Sender<>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    debug!("Send Frustum Thread: Started");
    // connect to viewerClient
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
    let mut client = ViewerClient::connect((args.host.as_str(), args.port), &mut shutdown_rx).await?;
    //loop to wait for new frustums to query.
    loop {
        //handle stop signal
        if stop_token.is_cancelled() {
            debug!("Send Frustum Thread: Stop signal received");
            break;
        }
        let image_id_and_frustum = match image_id_and_frustum_data_rx.recv_timeout(Duration::new(1,0)) {
            Ok(data) => {
                status.t2_send_frustum_thread_current_image_id.store(data.image_id, Ordering::Relaxed);
                data
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(error) => {
                warn!("image_id_and_frustum_data_rx error: {:?}", error);
                return Ok(());
            }
        };
        let frustum = image_id_and_frustum.frustum;
        let current_image_id = image_id_and_frustum.image_id;
        //debug!("image_id {:?}: start the query", current_image_id);

        let frustum_query = Query::ViewFrustum(frustum);
        //send query
        debug!("image_id {:?}: Send query", current_image_id);
        client
            .write
            .query_oneshot(
                frustum_query,
                &QueryConfig {
                    point_filtering: false,
                },
            )
            .await?;
        //loop to receive all the parts of the view frustum query.
        loop {
            let update = client
                .read
                .receive_update_global_coordinates(&mut shutdown_rx)
                .await?;
            match update {
                PartialResult::DeleteNode(_) => warn!("Received unexpected DeleteNode message."),
                PartialResult::UpdateNode(update) => {
                    //debug!("Send Frustum Thread: image_id {:?}: Received UpdateNode message, sending points", current_image_id);
                    status
                        .t2_send_frustum_thread_nr_received_points
                        .fetch_add(update.points.len() as u64, Ordering::Relaxed);
                    status
                        .t2_send_frustum_thread_nr_received_nodes
                        .fetch_add(1, Ordering::Relaxed);
                    match point_data_tx.send(ImageIdAndVectorBuffer {
                        image_id: current_image_id,
                        vector_buffer: update.points,
                        image_complete: false,
                    }) {
                        Ok(_) => {
                            //debug!("Send Frustum Thread: image_id {:?}: points sent", current_image_id);
                        }
                        Err(error) => {
                            warn!("image_id {:?}: error sending points: {:?}", current_image_id, error);
                            return Ok(());
                        }
                    }
                    //debug!("Send Frustum Thread: image_id {:?}: points sent fully done", current_image_id);
                }
                PartialResult::Complete => {
                    let layout = PointLayout::default(); // empty layout
                    debug!("image_id {:?}: Received complete message, sending image complete buffer",current_image_id);
                    point_data_tx.send(ImageIdAndVectorBuffer {
                        image_id: current_image_id,
                        vector_buffer: VectorBuffer::new_from_layout(layout),
                        image_complete: true,
                    })?;
                    break; //break as last points of frustum are received, due to channel properties order is guaranteed
                }
            }

        }
    }
    debug!("Send Frustum Thread: Shutting down viewerClient");
    shutdown_tx.send(()).ok();
    debug!("Send Frustum Thread: finished");
    Ok(())
}
