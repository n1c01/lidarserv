use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::{t4_colorization::ColorizationData, ImageData, ImageIdAndVectorBuffer};
use log::{debug, error};
use pasture_core::containers::BorrowedBuffer;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub(crate) fn thread_3_collect_colorization_data(
    _args: AppOptions, //todo! check if it can be removed completely
    stop_token: CancellationToken,
    points_rx: mpsc::Receiver<ImageIdAndVectorBuffer>,
    picture_data_rx: mpsc::Receiver<ImageData>,
    colorization_data_tx: mpsc::Sender<ColorizationData>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    let mut image_data_map: HashMap<u64, ImageData> = HashMap::new();

    let mut counter = 0;
    let mut image_data_counter = 0;
    let mut node_data_counter = 0;
    let mut node_complete_message_counter = 0;
    loop {
        debug!("collect_colorization_data_thread: \nloop count: {:?}, \nrecv image count: {:?},\nrecv node complete count: {:?}, \nrecv node count: {:?}",
            counter,image_data_counter,node_complete_message_counter, node_data_counter);
        counter += 1;
        //handle stop signal
        if stop_token.is_cancelled() {
            debug!("collect_colorization_data_thread: stop signal received");
            break;
        }
        //loop loop woop woop

        //receive picture data, but timeout after 1 second (for cooperative cancellation to work)
        match picture_data_rx.recv_timeout(Duration::new(1, 0)) {
            Ok(data) => {
                //safe picture data for processing
                status.t3_collect_colorization_data_thread_current_image_id.store(data.image_id_and_frustum.image_id,Ordering::Relaxed);
                safe_image_to_hashmap(data, &mut image_data_map);
                image_data_counter+=1;
            }
            Err(_) => { /*Timeout therefore nothing happens here*/ }
        };

        //receive points from the points_rx channel
        //debug!("collect_colorization_data_thread: waiting for points");
        let image_id_and_vec_buff = match points_rx.recv_timeout(Duration::new(1, 0)) {
            Ok(data) => {
                if data.image_complete {
                    //remove picture data once all the points are received.
                    debug!(
                        "collect_colorization_data_thread: image with id {:?} complete",
                        data.image_id
                    );
                    image_data_map.remove(&data.image_id);
                    node_complete_message_counter += 1;
                    continue;
                } else {
                    debug!(
                        "collect_colorization_data_thread: received points for image with id: {:?}",
                        data.image_id
                    );
                    status.t3_collect_colorization_data_thread_nr_received_points.fetch_add(data.vector_buffer.len() as u64, Ordering::Relaxed);
                    status.t3_collect_colorization_data_thread_nr_received_nodes.fetch_add(1, Ordering::Relaxed);
                    node_data_counter += 1;
                    data
                }
            }
            Err(_) => {
                //debug!("collect_colorization_data_thread: timeout (point data)");
                continue;
            }
        };

        //todo!("fix this seems to be not reachable");
        debug!("collect_colorization_data_thread: received points");
        let image_data = match image_data_map.get(&image_id_and_vec_buff.image_id) {
            None => {
                debug!("collect_colorization_data_thread: image with id: {:?} not in hashmap",&image_id_and_vec_buff.image_id);
                return Err(anyhow::anyhow!("collect_colorization_data_thread: image_id not found"));
                //todo handle problem by using a queue pop vecbuff with cloud checking if image is ready else push again to the end.
            }
            Some(data) => data ,
        };

        //debug!("collect_colorization_data_thread: state of hashmap {:?}",image_data_map );

        //todo maybe wait for collection of all the points for the picture (not at the moment)

        //combine picture data and points to colorization data
        let image_data = image_data.clone();
        let colorization_data = ColorizationData {
            image_data,
            point_data: image_id_and_vec_buff.vector_buffer,
        };

        //send colorization data to the colorization_data_tx channel
        debug!("collect_colorization_data_thread: sending data: image_id: {:?}, Vectorbuffer size: {:?}",
            colorization_data.image_data.image_id_and_frustum.image_id,
            colorization_data.point_data.len()
        );
        colorization_data_tx.send(colorization_data).unwrap_or_else(|e| {
            debug!("collect_colorization_data_thread: error sending colorization data: {:?}", e);
        })

    }

    Ok(())
}

fn safe_image_to_hashmap(image_data: ImageData, image_data_map: &mut HashMap<u64, ImageData>) {
    let image_id = image_data.image_id_and_frustum.image_id;
    let insertion_return = image_data_map.insert(image_id, image_data);
    if insertion_return.is_some() {
        error!("collect_colorization_data_thread: image_id isnt unique");
    } else {
        debug!(
            "collect_colorization_data_thread: image with id {:?} inserted into the hashmap",
            image_id
        );
    }
}
