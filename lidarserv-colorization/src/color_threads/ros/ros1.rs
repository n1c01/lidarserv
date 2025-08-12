use crate::cli::AppOptions;
use crate::color_threads::ros::ros1::messages::sensor_msgs::Image;
use crate::color_threads::ros::{Command, ImageData, ImageIdentifier};
use crate::color_threads::status::Status;
use anyhow::Result;
use anyhow::{anyhow, Error};
use log::{debug, info, trace, warn};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use lidarserv_common::query::view_frustum::ViewFrustumQuery;

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
    let image_id:AtomicU64 = Default::default();

    let image_callback = move |msg: messages::sensor_msgs::Image| {
        status1.nr_received_images.fetch_add(1, Ordering::Relaxed); //add 1 more image message to the counter
        let current_image_id = image_id.fetch_add(1, Ordering::Relaxed);
        debug!("image message header: {:?} of image_id: {:?}", msg.header, current_image_id);
        image_tx.send(parse_image_message(msg,current_image_id)).ok();
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
                Command::Exit => rosrust::shutdown(),
            }
        }
    });

    // ROS event loop
    rosrust::spin();
    Ok(())
}

pub(crate) fn parse_image_message(msg: Image, image_id: u64) -> ImageData {
    //todo: add positional data! 
    ImageData {
        image: msg.data,
        width: msg.width,
        height: msg.height,
        encoding: msg.encoding,
        identifier: ImageIdentifier{
            id: image_id,
            timestamp: Duration::new(msg.header.stamp.sec as u64, msg.header.stamp.nsec),
            sequence: msg.header.seq,
        },
        frustum: ViewFrustumQuery {
            camera_pos: Default::default(),
            camera_dir: Default::default(),
            camera_up: Default::default(),
            fov_y: 0.0,
            z_near: 0.0,
            z_far: 0.0,
            window_size: Default::default(),
            max_distance: 0.0,
        },
    }
}

mod messages {
    rosrust::rosmsg_include!(sensor_msgs / Image);
}
