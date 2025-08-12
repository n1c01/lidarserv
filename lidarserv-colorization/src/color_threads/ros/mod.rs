use crate::cli::AppOptions;
use crate::color_threads::status::Status;
use anyhow::Result;
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

impl fmt::Debug for ImageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageData")
            .field("image_vec_len", &self.image.len()) // Avoid printing full bytes
            .field("width", &self.width)
            .field("height", &self.height)
            .field("idenfifier", &self.identifier)
            .field("encoding", &self.encoding)
            .field("frustum", &self.frustum)
            .finish()
    }
}

pub struct ImageData {
    pub image: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub encoding: String,
    pub identifier: ImageIdentifier,
    pub frustum: ViewFrustumQuery,
}
impl fmt::Debug for ImageIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageIdentifier")
            .field("id", &self.id)
            .field("timestamp", &self.timestamp)
            .field("sequence", &self.sequence)
            .finish()
    }
}
pub struct ImageIdentifier {
    pub id: u64,
    pub timestamp: Duration,
    pub sequence: u32,
}
