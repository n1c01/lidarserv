use std::collections::HashMap;
use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::{ColorizationData, ImageData, ImageIdAndVectorBuffer};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::vec;
use log::{debug, error, warn};
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
    let mut image_data_map= HashMap::new();

    loop {
        //handle stop signal
        if stop_token.is_cancelled() {
            debug!("collect_colorization_data_thread: stop signal received");
            break;
        }
        //loop loop woop woop
        //todo!("lidarserv answer thread")

        //receive picture data, but timeout after 1 second (for cooperative cancellation to work)
        let image_data = match picture_data_rx.recv_timeout(Duration::new(1, 0)) {
            Ok(data) => {data}
            Err(_) => {
                // Timeout therefore skip to the next loop iteration
                continue;
            }
        };
        let image_id = image_data.image_id_and_frustum.image_id;
        let mut colorization_data = ColorizationData{
            image_data,
            point_data: Vec::new(),
        };
        let insertion_return = image_data_map.insert(
            image_id,
            colorization_data
        );
        if insertion_return.is_some() {
            error!("collect_colorization_data_thread: image_id isnt unique");
        } else {
            debug!("collect_colorization_data_thread: image with id {:?} inserted into the hashmap",image_id);
        }


        //receive points from the points_rx channel
        let image_id_and_vec_buff = match points_rx.recv_timeout(Duration::new(1, 0)) {
            Ok(data) => {data}
            Err(_) => {
                debug!("collect_colorization_data_thread: timeout (point data)");
                continue;
            }
        };

        debug!("collect_colorization_data_thread: state of hashmap {:?}", image_data_map);



        //safe picture data for processing

        //remove picture data once all the points are received.

        //maybe wait for collection of all the points for the picture

        //combine picture data and points to colorization data

        //send colorization data to the colorization_data_tx channel
    }
    Ok(())
}
