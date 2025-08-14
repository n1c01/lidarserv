use std::sync::{mpsc, Arc};
use std::sync::mpsc::{Receiver, Sender};
use crate::cli::AppOptions;
use crate::color_threads::{ColorizationData, ImageIdAndFrustum, ImageIdAndVectorBuffer};
use crate::color_threads::status::Status;

pub(crate) fn collect_colorization_data_thread(
    args: AppOptions,
    points_rx: mpsc::Receiver<ImageIdAndVectorBuffer>,
    colorization_data_tx: mpsc::Sender<ColorizationData>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    loop {
        //loop loop woop woop
    }
}