use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use anyhow::{anyhow, Error};
use crate::cli::AppOptions;
use crate::color_threads::ros::{Command, ImageData};
use anyhow::Result;
use log::{info, trace};
use crate::color_threads::status::Status;

pub(crate) fn ros_thread(app_options: AppOptions,
                         commands_rx: Receiver<Command>,
                         image_tx: Sender<ImageData>,
                         status: Arc<Status>,) -> Result<()> {
    // ROS init
    info!("Connecting to ROS master...");
    if rosrust::try_init_with_options("lidarserv-color", false).is_err() {
        return Err(anyhow!("Failed to connect to ROS master."));
    }
    info!("Connected to ROS master.");

    //todo: multiple image topics
    /*
    for image_topic in app_options.image_topics{
        info!(
        "Subscribing to image topic `{}`",
        image_topic);
    }
    */
    let image_topic = app_options.image_topics[0].clone();
    let status1 = Arc::clone(&status);

    let image_callback = move |msg: messages::sensor_msgs::Image| {
        trace!("image message: {msg:?}");
        status1.nr_rx_msg_tf.fetch_add(1, Ordering::Relaxed);
        /*
        for image in parse_image_message(msg, true) {
            transforms_tx_clone.send(tf).ok();
        }
         */
    };
    let _image_subscriber = match rosrust::subscribe(&image_topic, 100, image_callback) {
        Ok(s) => s,
        Err(_) => {
            return Err(anyhow!(
                "Failed to subscribe to image topic `{}`.",
                image_topic
            ))
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
/*
fn parse_image_message(p0: _, p1: bool) -> _ {
    todo!()
}

 */


mod messages {
    rosrust::rosmsg_include!(sensor_msgs / Image);
}