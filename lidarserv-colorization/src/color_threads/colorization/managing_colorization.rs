use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::{colorization::ColorizationData, ImageData, ImageIdAndVectorBuffer};
use image::{DynamicImage, ImageReader};
use log::{debug, warn};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use lidarserv_common::nalgebra::{Point3, Vector2, Vector3};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use crate::color_threads::colorization::point_cloud_colorizer::PointCloudColorizer;

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

        let colorization_data = match colorization_data_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(data) => {
                debug!("process_frustum_thread: colorization data received");
                data
            },
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
        
        let point_cloud_colorizer = PointCloudColorizer{
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
        point_cloud_colorizer.colorize(colorization_data).expect("TODO: panic message");

        
    }
    Ok(())
}
