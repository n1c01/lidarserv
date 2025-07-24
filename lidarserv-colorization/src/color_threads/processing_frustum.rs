use std::sync::{mpsc, Arc};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use crate::cli::AppOptions;
use crate::color_threads::ros::{Command, ImageData};
use crate::color_threads::status::Status;

pub fn process_frustum_thread(
    args: AppOptions,
    image_data_rx: mpsc::Receiver<ImageData>,
    frustum_data_tx: mpsc::Sender<ViewFrustumQuery>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    todo!("turn Image Data into ViewFrustumQuery")
}
/*
Query::ViewFrustum(ViewFrustumQuery {
                    camera_pos,
                    camera_dir,
                    camera_up: vector![0.0, 0.0, 1.0],
                    fov_y: FRAC_PI_4,
                    z_near,
                    z_far,
                    window_size: camera_matrix.window_size,
                    max_distance: args.point_distance, //vermutlich sehr niedrig wählen
                }
 */