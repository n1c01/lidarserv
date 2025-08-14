use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use anyhow::Result;
use crate::color_threads::{ImageData};

use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use std::fmt;
use std::sync::{mpsc, Arc};
use std::time::Duration;

mod ros1;

pub fn ros_thread(
    args: AppOptions,
    commands_rx: mpsc::Receiver<Command>,
    image_data_tx: mpsc::Sender<ImageData>,
    status: Arc<Status>,
) -> Result<()> {
    // in the future this could call either ros1 or ros2
    // (once we add support for ros2)
    ros1::ros_thread(args, commands_rx, image_data_tx, status)
}


pub enum Command {
    Exit,
}

