use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::t4_colorization::point_cloud_colorizer::PointCloudColorizer;
use crate::color_threads::{t4_colorization::ColorizationData};
use image::{DynamicImage};
use lidarserv_common::nalgebra::{Point3, Vector2, Vector3};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use log::{debug, warn};
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub(crate) fn thread_4_managing_colorization(
    args: AppOptions,
    stop_token: CancellationToken,
    colorization_data_rx: mpsc::Receiver<ColorizationData>,
    colorized_points: mpsc::Sender<ColorizationData>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    loop {
        //handle stop signal
        if stop_token.is_cancelled() {
            debug!("thread_4_managing_colorization: Stop signal received");
            break;
        }
        let mut dynamic_image = DynamicImage::new_rgb8(100, 100);

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
            frustum: ViewFrustumQuery {
                //example frustum todo: add real frustum (positional data)
                camera_pos: Point3::new(-54.40324866531016, 2.3269014261665073, 10.108743731819478),
                camera_dir: Vector3::new(
                    -0.8632547306347013,
                    -0.374380434165962,
                    -0.3385713522295046,
                ),
                camera_up: Vector3::new(0.0, 0.0, 1.0),
                fov_y: 0.7853981633974483,
                z_near: 0.2985705572917801,
                z_far: 298570.5573180077,
                window_size: Vector2::new(500.0, 500.0),
                max_distance: 10.0,
            },
            dynamic_image,
        };
        debug!("thread_4_managing_colorization: colorization started");
        match point_cloud_colorizer.colorize(colorization_data){
            Ok(data) => {
                debug!("thread_4_managing_colorization: colorization done")
                //todo send colorized data
            }
            Err(error) => {
                debug!("thread_4_managing_colorization: colorization failed, with error {:?}", error);
                return Ok(());
            }
        }
    }
    Ok(())
}
