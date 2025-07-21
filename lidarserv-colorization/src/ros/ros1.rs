use std::sync::mpsc::{Receiver, Sender};
use crate::cli::AppOptions;
use crate::ros::{Command, Transform};

pub(crate) fn ros_thread(app_options: AppOptions, command_rx: Receiver<Command>, other_tx: Sender<Transform>) -> Result<()> {
    todo!()
}