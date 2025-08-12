use crate::cli::AppOptions;
use crate::color_threads::ros::ros1::messages::sensor_msgs::Image;
use crate::color_threads::ros::{Command, ImageData};
use crate::color_threads::status::Status;
use anyhow::Result;
use anyhow::{anyhow, Error};
use log::{debug, info, trace, warn};
use std::sync::atomic::Ordering;
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

    //todo!("process multiple images");

    let image_topic = app_options.image_topics[0].clone();
    let status1 = Arc::clone(&status);

    let image_callback = move |msg: messages::sensor_msgs::Image| {
        status1.nr_rx_msg_image.fetch_add(1, Ordering::Relaxed); //add 1 more image message to the counter
        debug!("image message header: {:?}", msg.header);
        image_tx.send(parse_image_message(msg)).ok();
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

fn parse_image_message(msg: Image) -> ImageData {
    ImageData {
        image: msg.data,
        width: msg.width,
        height: msg.height,
        timestamp: Duration::new(msg.header.stamp.sec as u64, msg.header.stamp.nsec),
        sequence: msg.header.seq,
        encoding: msg.encoding,
    }
}

mod messages {
    rosrust::rosmsg_include!(sensor_msgs / Image);
}
