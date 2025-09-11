use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::{ImageIdAndFrustum, ImageIdAndVectorBuffer};
use lidarserv_server::{
    index::query::Query,
    net::client::viewer::{PartialResult, QueryConfig, ViewerClient},
};
use log::{debug, warn};
use pasture_core::containers::{BorrowedBuffer, MakeBufferFromLayout, VectorBuffer};
use pasture_core::layout::PointLayout;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

pub async fn thread_2_send_frustum(
    args: AppOptions,
    stop_token: CancellationToken,
    image_id_and_frustum_data_rx: mpsc::Receiver<ImageIdAndFrustum>, //Receiver to get the View Frustum Query for each image
    point_data_tx: mpsc::Sender<ImageIdAndVectorBuffer>, //Sender to send the points to the viewer.
    //point_data_complete_tx: mpsc::Sender<>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    debug!("Send Frustum Thread: Started");
    // connect to viewerClient
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
    let mut client = ViewerClient::connect((args.host.as_str(), args.port), &mut shutdown_rx).await?;
    //loop to wait for new frustums to query.
    loop {
        //handle stop signal
        if stop_token.is_cancelled() {
            debug!("Send Frustum Thread: Stop signal received");
            break;
        }
        let image_id_and_frustum = match image_id_and_frustum_data_rx.recv_timeout(Duration::new(1,0)) {
            Ok(data) => {
                status.t2_send_frustum_thread_current_image_id.store(data.image_id, Ordering::Relaxed);
                data
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(error) => {
                warn!("image_id_and_frustum_data_rx error: {:?}", error);
                return Ok(());
            }
        };
        let frustum = image_id_and_frustum.frustum;
        let current_image_id = image_id_and_frustum.image_id;
        //debug!("image_id {:?}: start the query", current_image_id);

        let frustum_query = Query::ViewFrustum(frustum);
        //send query
        debug!("image_id {:?}: Send query: {:?}", current_image_id, frustum_query);
        client
            .write
            .query_oneshot( //todo! change to query for permanant updates
                frustum_query,
                &QueryConfig {
                    point_filtering: false,
                },
            )
            .await?;
        //loop to receive all the parts of the view frustum query.
        loop {
            let update = client
                .read
                .receive_update_global_coordinates(&mut shutdown_rx)
                .await?;
            match update {
                PartialResult::DeleteNode(_) => warn!("Received unexpected DeleteNode message."),
                PartialResult::UpdateNode(update) => {
                    //debug!("Send Frustum Thread: image_id {:?}: Received UpdateNode message, sending points", current_image_id);
                    status
                        .t2_send_frustum_thread_nr_received_points
                        .fetch_add(update.points.len() as u64, Ordering::Relaxed);
                    status
                        .t2_send_frustum_thread_nr_received_nodes
                        .fetch_add(1, Ordering::Relaxed);
                    match point_data_tx.send(ImageIdAndVectorBuffer {
                        image_id: current_image_id,
                        vector_buffer: update.points,
                        image_complete: false,
                    }) {
                        Ok(_) => {
                            //debug!("Send Frustum Thread: image_id {:?}: points sent", current_image_id);
                        }
                        Err(error) => {
                            warn!("image_id {:?}: error sending points: {:?}", current_image_id, error);
                            return Ok(());
                        }
                    }
                    //debug!("Send Frustum Thread: image_id {:?}: points sent fully done", current_image_id);
                }
                PartialResult::Complete => {
                    let layout = PointLayout::default(); // empty layout
                    debug!("image_id {:?}: Received complete message, sending image complete buffer",current_image_id);
                    point_data_tx.send(ImageIdAndVectorBuffer {
                        image_id: current_image_id,
                        vector_buffer: VectorBuffer::new_from_layout(layout),
                        image_complete: true,
                    })?;
                    break; //break as last points of frustum are received, due to channel properties order is guaranteed
                }
            }

        }
    }
    debug!("Send Frustum Thread: Shutting down viewerClient");
    shutdown_tx.send(()).ok();
    debug!("Send Frustum Thread: finished");
    Ok(())
}
/*
#[cfg(test)]
mod tests {
    //todo! do test properly
    use super::thread_2_send_frustum;
    use crate::cli::AppOptions;
    use crate::color_threads::status::Status;
    use crate::color_threads::{ImageIdAndFrustum, ImageIdAndVectorBuffer};
    use anyhow::Result;
    use clap::Parser;
    use lidarserv_common::nalgebra::{Point3, Vector2, Vector3};
    use lidarserv_common::query::view_frustum::ViewFrustumQuery;
    use lidarserv_server::index::builder::build;
    use lidarserv_server::index::settings::IndexSettings;
    use lidarserv_server::net::server::serve;
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::Duration;
    use tokio::runtime::Builder;
    use tokio_util::sync::CancellationToken;

    fn start_test_server(host: &str, port: u16) -> (tokio::sync::broadcast::Sender<u32>, thread::JoinHandle<()>) {
        use lidarserv_common::geometry::coordinate_system::CoordinateSystem;
        use lidarserv_common::geometry::grid::GridHierarchy;
        use lidarserv_server::common::geometry::grid::LodLevel;
        use pasture_core::layout::PointLayout;
        use lidarserv_common::nalgebra::Vector3;
        use pasture_core::layout::attributes::POSITION_3D;
        use lidarserv_common::index::priority_function::TaskPriorityFunction;

        // Create a temporary data directory for the index
        let tmp_dir = std::env::temp_dir().join(format!("lidarserv_test_index_{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir).expect("create temp data dir");

        // Minimal settings for an empty index with a simple coordinate system and only position attribute
        let point_layout = PointLayout::from_attributes(&[POSITION_3D]);
        let coordinate_system = CoordinateSystem::from_las_transform(Vector3::new(1.0, 1.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        let node_hierarchy = GridHierarchy::new(16); // root node size 2^16 in local units
        let point_hierarchy = GridHierarchy::new(8); // sampling grid shift
        let max_lod = LodLevel::from_level(4);

        let settings = IndexSettings {
            use_metrics: false,
            node_hierarchy,
            point_hierarchy,
            coordinate_system,
            max_lod,
            max_bogus_inner: 0,
            max_bogus_leaf: 0,
            enable_compression: false,
            max_cache_size: 10_000,
            priority_function: TaskPriorityFunction::NrPoints,
            num_threads: 2,
            point_layout,
            attribute_indexes: vec![],
        };
        let index = build(settings, &tmp_dir).expect("Failed to build index");

        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel::<u32>(1);
        let host_str = host.to_string();
        let handle = thread::spawn(move || {
            let rt = Builder::new_multi_thread().enable_all().build().unwrap();
            rt.block_on(async move {
                let _ = serve((host_str.as_str(), port), index, shutdown_rx).await;
            });
        });
        (shutdown_tx, handle)
    }

    fn far_away_frustum() -> ViewFrustumQuery {
        ViewFrustumQuery {
            camera_pos: Point3::new(1.0e9, 1.0e9, 1.0e9),
            camera_dir: Vector3::new(1.0, 0.0, 0.0),
            camera_up: Vector3::new(0.0, 0.0, 1.0),
            fov_y: 1.0,
            z_near: 0.1,
            z_far: 1.0e6,
            window_size: Vector2::new(1920.0, 1080.0),
            max_distance: 1.0e6,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_thread_2_send_frustum_completes_without_points() -> Result<()> {
        let host = "127.0.0.1";
        let port: u16 = 45679;
        let (server_shutdown, server_thread) = start_test_server(host, port);
        std::thread::sleep(Duration::from_millis(200));

        let (frustum_tx, frustum_rx) = mpsc::channel::<ImageIdAndFrustum>();
        let (points_tx, points_rx) = mpsc::channel::<ImageIdAndVectorBuffer>();
        let status = Arc::new(Status::default());
        let stop_token = CancellationToken::new();

        let args = AppOptions::parse_from(["test", "--host", host, "--port", &port.to_string()]);

        let status_clone = Arc::clone(&status);
        let stop_token_child = stop_token.clone();
        let fut = thread_2_send_frustum(
            args,
            stop_token_child,
            frustum_rx,
            points_tx,
            status_clone,
        );
        let task_handle = tokio::spawn(async move { fut.await.unwrap() });

        let image_id = 42u64;
        frustum_tx
            .send(ImageIdAndFrustum {
                image_id,
                frustum: far_away_frustum(),
            })
            .unwrap();

        let mut received_non_complete = 0usize;
        let mut completed = false;
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while std::time::Instant::now() < deadline {
            match points_rx.recv_timeout(Duration::from_millis(200)) {
                Ok(msg) => {
                    assert_eq!(msg.image_id, image_id);
                    if msg.image_complete {
                        completed = true;
                        break;
                    } else {
                        received_non_complete += 1;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(e) => panic!("Unexpected channel error: {e:?}"),
            }
        }

        assert!(completed, "Did not receive image_complete within timeout");
        assert_eq!(received_non_complete, 0, "Expected no node updates for far-away frustum");

        use std::sync::atomic::Ordering;
        assert_eq!(status.t2_send_frustum_thread_current_image_id.load(Ordering::Relaxed), image_id);
        assert_eq!(status.t2_send_frustum_thread_nr_received_nodes.load(Ordering::Relaxed), 0);
        assert_eq!(status.t2_send_frustum_thread_nr_received_points.load(Ordering::Relaxed), 0);

        stop_token.cancel();
        let _ = server_shutdown.send(1);
        let _ = tokio::time::timeout(Duration::from_secs(5), task_handle).await;
        std::thread::sleep(Duration::from_millis(200));
        let _ = server_thread.join();

        Ok(())
    }
}

 */
