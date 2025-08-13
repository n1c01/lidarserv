use std::fmt;
use std::time::Duration;
use lidarserv_common::query::view_frustum::ViewFrustumQuery;

pub(crate) mod processing_frustum;
pub(crate) mod ros;
mod send_frustum;
pub(crate) mod status;



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

