use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::t4_colorization::point_cloud_colorizer::PointCloudColorizer;
use crate::color_threads::{t4_colorization::ColorizationData};
use image::{DynamicImage, RgbaImage};
use log::{debug, warn};
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub(crate) fn thread_4_managing_colorization(
    _args: AppOptions, //todo! check if it can be removed
    stop_token: CancellationToken,
    colorization_data_rx: mpsc::Receiver<ColorizationData>,
    _colorized_points: mpsc::Sender<ColorizationData>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    loop {
        //handle stop signal
        if stop_token.is_cancelled() {
            debug!("thread_4_managing_colorization: Stop signal received");
            break;
        }

        let colorization_data = match colorization_data_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(data) => {
                debug!("thread_4_managing_colorization: colorization data received");
                status.t4_managing_colorization_thread_current_image_id.store(data.image_data.image_id_and_frustum.image_id,Ordering::Relaxed);
                data
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                //skip to check cancellation token
                //debug!("thread_4_managing_colorization: timeout");
                continue;
            }
            Err(error) => {
                warn!("image_data_rx error: {:?}", error);
                return Ok(());
            }
        };

        let point_cloud_colorizer = PointCloudColorizer {
            //todo use real colorizer
            frustum: colorization_data.image_data.image_id_and_frustum.frustum,
            dynamic_image: vec_to_dynamic_image_raw(colorization_data.image_data.image,
                                                    colorization_data.image_data.width,
                                                    colorization_data.image_data.height),
        };
        debug!("thread_4_managing_colorization: colorization started");
        let colorized_points = match point_cloud_colorizer.colorize(colorization_data.point_data){
            Ok(data) => {
                debug!("thread_4_managing_colorization: colorization done");
                data
            }
            Err(error) => {
                debug!("thread_4_managing_colorization: colorization failed, with error {:?}", error);
                return Ok(());
            }
        };
        debug!("point data, after colorization: {:?}",colorized_points);
    }
    Ok(())
}


fn vec_to_dynamic_image_raw(pixels: Vec<u8>, width: u32, height: u32) -> DynamicImage {
    // Assuming RGBA8 pixel format (4 channels per pixel)
    let img = RgbaImage::from_raw(width, height, pixels)
        .expect("Invalid buffer length for given dimensions");
    DynamicImage::ImageRgba8(img)
}