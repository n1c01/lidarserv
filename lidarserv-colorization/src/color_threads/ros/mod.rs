use std::sync::{mpsc, Arc};
use crate::cli::AppOptions;
use anyhow::Result;

mod ros1;

pub fn ros_thread(
    args: AppOptions,
    commands_rx: mpsc::Receiver<Command>,
    image_data_tx: mpsc::Sender<ImageData>
) -> Result<()> {
    // in the future this could call either ros1 or ros2
    // (once we add support for ros2)
    ros1::ros_thread(args, commands_rx, image_data_tx)
}

pub enum Command {
    Exit,
}

pub struct ImageData {
    pub image: Vec<u8>,
    pub timestamp: u64,
}