use crate::cli::AppOptions;
use crate::color_threads::ros::{Command, ImageData};
use crate::color_threads::status::Status;
use anyhow::{anyhow, Error};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use log::{debug, error, info};
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};

pub fn process_frustum_thread(
    args: AppOptions,
    image_data_rx: mpsc::Receiver<ImageData>,
    frustum_data_tx: mpsc::Sender<ViewFrustumQuery>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    debug!("process_frustum_thread: is started");

    //loop waiting for image data to extract the frustum from it.
    loop {
        //todo! here waiting for more points could be impelmented. (e.g. wayting a fixed amout of time.)
        let image_data = image_data_rx.recv()?; //receiving image from ros input thread
        status.nr_process_frustum_in.fetch_add(1, Ordering::Relaxed);
        debug!("process_frustum_thread: The image {:?}", image_data);
        frustum_data_tx.send(image_data.frustum).ok(); //sending image to lidarserv frustum query thread
        status
            .nr_process_frustum_out
            .fetch_add(1, Ordering::Relaxed);

        //todo! make stopable
    }
    debug!("process_frustum_thread: is finished");
}
