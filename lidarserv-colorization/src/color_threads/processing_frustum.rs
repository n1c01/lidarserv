use crate::cli::AppOptions;
use crate::color_threads::ros::Command;
use crate::color_threads::status::Status;
use crate::color_threads::{ImageData, ImageIdAndFrustum};
use anyhow::{anyhow, Error};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use log::{debug, error, info, warn};
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::Receiver;
use crate::color_threads::cross_thread_functionality::stop_lidarserv_colorization;

pub fn process_frustum_thread(
    args: AppOptions,
    mut stop_lidarserv_colorization_rx: Receiver<()>,
    image_data_rx: mpsc::Receiver<ImageData>,
    frustum_data_tx: mpsc::Sender<ImageIdAndFrustum>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    debug!("process_frustum_thread: is started");

    //loop waiting for image data to extract the frustum from it.
    loop {
        //todo! here waiting for more points could be impelmented. (e.g. wayting a fixed amout of time.)
        let image_data = match image_data_rx.recv() { //receiving image from ros input thread
            Ok(data) => data,
            Err(error) => {
                warn!("image_data_rx error: {:?}",error);
                return Ok(());
            }
        };
        status.nr_process_frustum_in.fetch_add(1, Ordering::Relaxed);
        debug!("image_id: {:?} \nThe frustum {:?}", image_data.image_id_and_frustum.image_id, image_data.image_id_and_frustum.frustum);
        frustum_data_tx.send(image_data.image_id_and_frustum).ok(); //sending image to lidarserv frustum query thread
        status
            .nr_process_frustum_out
            .fetch_add(1, Ordering::Relaxed);

        //handle stop signal
        if stop_lidarserv_colorization(&stop_lidarserv_colorization_rx) {
            debug!("process_frustum_thread: stop lidarserv colorization received");
            break;
        };

    }
    debug!("process_frustum_thread: is finished");
    Ok(())
}
