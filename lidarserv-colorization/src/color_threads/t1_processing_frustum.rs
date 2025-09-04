use crate::cli::AppOptions;
use crate::color_threads::cross_thread_functionality::check_stop_lidarserv_colorization;
use crate::color_threads::status::Status;
use crate::color_threads::t0_ros::Command;
use crate::color_threads::{ImageData, ImageIdAndFrustum};
use anyhow::{anyhow, Error};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use log::{debug, error, info, warn};
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::Receiver;
use tokio_util::sync::CancellationToken;

pub fn thread_1_process_frustum(
    args: AppOptions,
    stop_token: CancellationToken,
    image_data_rx: mpsc::Receiver<ImageData>,
    image_data_bypass_tx: mpsc::Sender<ImageData>,
    frustum_data_tx: mpsc::Sender<ImageIdAndFrustum>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    debug!("process_frustum_thread: is started");

    //loop waiting for image data to extract the frustum from it.
    loop {
        //handle stop signal
        if stop_token.is_cancelled() {
            debug!("process_frustum_thread: Stop signal received");
            break;
        }

        //receiving image from ros input thread (using timeout to release blocking and check cooperative cancellation)
        let image_data = match image_data_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(data) => data,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                //skip to check cancellation token
                //debug!("process_frustum_thread: timeout");
                continue;
            }
            Err(error) => {
                warn!("image_data_rx error: {:?}", error);
                return Ok(());
            }
        };
        image_data_bypass_tx.send(image_data.clone()).ok();
        //Here waiting for more points could be impelmented. (e.g. wayting a fixed amout of time.)
        thread::sleep(Duration::from_secs(3));

        status.nr_process_frustum_in.fetch_add(1, Ordering::Relaxed);
        debug!(
            "image_id: {:?} \nThe frustum {:?}",
            image_data.image_id_and_frustum.image_id, image_data.image_id_and_frustum.frustum
        );
        frustum_data_tx.send(image_data.image_id_and_frustum).ok(); //sending image to lidarserv frustum query thread
        status
            .nr_process_frustum_out
            .fetch_add(1, Ordering::Relaxed);
    }
    debug!("process_frustum_thread: is finished");
    Ok(())
}
