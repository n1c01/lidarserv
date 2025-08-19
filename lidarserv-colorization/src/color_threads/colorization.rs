use std::sync::{mpsc, Arc};
use log::debug;
use tokio_util::sync::CancellationToken;
use crate::cli::AppOptions;
use crate::color_threads::{ColorizationData, ImageData, ImageIdAndVectorBuffer};
use crate::color_threads::status::Status;

pub fn colorization_thread(
    args: AppOptions,
    stop_token: CancellationToken,
    colorization_data_rx: mpsc::Receiver<ColorizationData>,
    colorized_points: mpsc::Sender<ColorizationData>,
    status: Arc<Status>,
)-> anyhow::Result<()> {
    loop {
        //handle stop signal
        if stop_token.is_cancelled() {
            debug!("process_frustum_thread: Stop signal received");
            break;
        }



    }
    Ok(())
}