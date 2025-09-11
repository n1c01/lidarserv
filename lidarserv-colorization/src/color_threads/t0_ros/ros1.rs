use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use crate::color_threads::t0_ros::ros1::messages::sensor_msgs::Image;
use crate::color_threads::t0_ros::Command;
use crate::color_threads::{ImageData, ImageIdAndFrustum};
use anyhow::anyhow;
use anyhow::Result;
use lidarserv_common::nalgebra::{Point3, Vector2};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use log::{debug, info};
use pasture_core::nalgebra::Vector3;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub(crate) fn ros_thread(
    app_options: AppOptions,
    commands_rx: Receiver<Command>,
    image_tx: Sender<ImageData>,
    status: Arc<Status>,
) -> Result<()> {
    // ROS init
    info!("Connecting to ROS master...");
    if rosrust::try_init_with_options("lidarserv_color", false).is_err() {
        return Err(anyhow!("Failed to connect to ROS master."));
    }
    info!("Connected 'lidarserv_color' to ROS master.");

    //todo!("process multiple image topics");
    let image_topic = app_options.image_topics[0].clone();
    let status1 = Arc::clone(&status);
    let image_id: AtomicU64 = Default::default();

    let image_callback = move |msg: messages::sensor_msgs::Image| {
        status1.nr_received_images.fetch_add(1, Ordering::Relaxed); //add 1 more image message to the counter
        let current_image_id = image_id.fetch_add(1, Ordering::Relaxed);
        status.t0_ros_current_image_id.store(current_image_id, Ordering::Relaxed);

        debug!(
            "image_id {:?} \nMessage header: {:?} ",
            current_image_id, msg.header,
        );
        image_tx
            .send(parse_image_message(msg, current_image_id))
            .ok();
    };
    let _image_subscriber = match rosrust::subscribe(&image_topic, 100, image_callback) {
        Ok(s) => s,
        Err(_) => {
            return Err(anyhow!(
                "Failed to subscribe to image topic `{}`.",
                image_topic
            ));
        }
    };

    info!("Subscribed to image topics.");

    // Control thread (for exiting)
    thread::spawn(move || {
        for cmd in commands_rx {
            match cmd {
                Command::Exit => {
                    debug!("ros1 thread: rosrust is shut down");
                    rosrust::shutdown()
                }
            }
        }
    });

    // ROS event loop
    rosrust::spin();
    Ok(())
}

pub(crate) fn parse_image_message(msg: Image, image_id: u64) -> ImageData {
    ImageData {
        image: msg.data,
        width: msg.width,
        height: msg.height,
        encoding: msg.encoding,
        timestamp: Duration::new(msg.header.stamp.sec as u64, msg.header.stamp.nsec),
        sequence: msg.header.seq,
        image_id_and_frustum: ImageIdAndFrustum {
            image_id: image_id,
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
        },
    }
}

mod messages {
    rosrust::rosmsg_include!(sensor_msgs / Image);
}
