use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::{ColorizationData, ImageData, ImageIdAndVectorBuffer};
use std::sync::{mpsc, Arc};
use log::debug;
use tokio::sync::broadcast::Receiver;
use tokio_util::sync::CancellationToken;
use crate::color_threads::cross_thread_functionality::check_stop_lidarserv_colorization;

pub(crate) fn collect_colorization_data_thread(
    args: AppOptions,
    stop_token: CancellationToken,
    points_rx: mpsc::Receiver<ImageIdAndVectorBuffer>,
    picture_data_rx: mpsc::Receiver<ImageData>,
    colorization_data_tx: mpsc::Sender<ColorizationData>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    loop {
        if stop_token.is_cancelled() {
            debug!("collect_colorization_data_thread: stop signal received");
            break;
        } //handle stop signal
        //loop loop woop woop
        //todo!("lidarserv answer thread")

        //receive points from the points_rx channel

        //receive picture data
        //let picture = picture_data_rx.recv()?;
        //safe picture data for processing

        //remove picture data once all the points are received.

        //maybe wait for collection of all the points for the picture

        //combine picture data and points to colorization data

        //send colorization data to the colorization_data_tx channel
    }
    Ok(())
}
