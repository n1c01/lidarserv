use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::{colorization::ColorizationData, ImageData, ImageIdAndVectorBuffer};
use image::{DynamicImage, ImageReader};
use log::debug;
use std::sync::{mpsc, Arc};
use tokio_util::sync::CancellationToken;

pub(crate) fn managing_colorization_thread(
    args: AppOptions,
    stop_token: CancellationToken,
    colorization_data_rx: mpsc::Receiver<ColorizationData>,
    colorized_points: mpsc::Sender<ColorizationData>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    loop {
        //handle stop signal
        if stop_token.is_cancelled() {
            debug!("process_frustum_thread: Stop signal received");
            break;
        }
        let mut dynamic_image = DynamicImage::new_rgb8(100, 100);

        let colorization_data = colorization_data_rx.recv()?;
    }
    Ok(())
}
