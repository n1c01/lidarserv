use std::sync::mpsc::{Receiver, Sender};
use anyhow::Error;
use crate::cli::AppOptions;
use crate::color_threads::ros::{Command, Transform};
use anyhow::Result;

pub(crate) fn ros_thread(app_options: AppOptions, command_rx: Receiver<Command>, other_tx: Sender<Transform>) -> Result<()> {
    todo!()
}