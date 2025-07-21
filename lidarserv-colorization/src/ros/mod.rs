use std::sync::{mpsc, Arc};
use crate::cli::AppOptions;

mod ros1;

pub fn ros_thread(
    args: AppOptions,
    commands_rx: mpsc::Receiver<Command>,
    transforms_tx: mpsc::Sender<Transform>,
) -> anyhow::Result<()> {
    // in the future this could call either ros1 or ros2
    // (once we add support for ros2)
    ros1::ros_thread(args, commands_rx, transforms_tx)
}

pub enum Command {
    Exit,
}


pub struct Transform {
    //TODO:transformation
}