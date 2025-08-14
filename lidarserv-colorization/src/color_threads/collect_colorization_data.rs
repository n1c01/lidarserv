use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::{ColorizationData, ImageData, ImageIdAndVectorBuffer};
use std::sync::{mpsc, Arc};

pub(crate) fn collect_colorization_data_thread(
    args: AppOptions,
    points_rx: mpsc::Receiver<ImageIdAndVectorBuffer>,
    picture_data_rx: mpsc::Receiver<ImageData>,
    colorization_data_tx: mpsc::Sender<ColorizationData>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    loop {
        //loop loop woop woop
        //todo!("lidarserv answer thread")

        //receive points from the points_rx channel

        //receive picture data
        let picture = picture_data_rx.recv()?;
        //safe picture data for processing

        //remove picture data once all the points are received.

        //maybe wait for collection of all the points for the picture

        //combine picture data and points to colorization data

        //send colorization data to the colorization_data_tx channel
    }
}
