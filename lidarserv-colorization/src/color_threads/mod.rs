use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use pasture_core::containers::{BorrowedBuffer, VectorBuffer};
use std::fmt;
use std::time::Duration;

pub(crate) mod collect_colorization_data;
pub(crate) mod processing_frustum;
pub(crate) mod ros;
pub(crate) mod send_frustum;
pub(crate) mod status;
pub(crate) mod cross_thread_functionality;
pub (crate) mod colorization;

impl fmt::Debug for ImageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageData")
            .field("image_vec_len", &self.image.len()) // Avoid printing full bytes
            .field("width", &self.width)
            .field("height", &self.height)
            .field("timestamp", &self.timestamp)
            .field("sequence", &self.sequence)
            .field("encoding", &self.encoding)
            .field("image_id_and_frustum", &self.image_id_and_frustum)
            .finish()
    }
}

pub struct ImageData {
    pub image: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub timestamp: Duration,
    pub sequence: u32,
    pub encoding: String,
    pub image_id_and_frustum: ImageIdAndFrustum, //Image identifier and frustum query that can be used to query the image without the image data
}

impl ImageData {
    pub(crate) fn clone(&self) -> ImageData {
        ImageData {
            image: self.image.clone(),
            width: self.width,
            height: self.height,
            timestamp: self.timestamp,
            sequence: self.sequence,
            encoding: self.encoding.clone(),
            image_id_and_frustum: ImageIdAndFrustum {
                image_id: self.image_id_and_frustum.image_id,
                frustum: self.image_id_and_frustum.frustum.clone(),
            },
        }
    }
}


impl fmt::Debug for ImageIdAndFrustum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageIdentifier")
            .field("id", &self.image_id)
            .field("frustum", &self.frustum)
            .finish()
    }
}
pub struct ImageIdAndFrustum {
    pub image_id: u64,
    pub frustum: ViewFrustumQuery,
}

pub struct ImageIdAndVectorBuffer {
    pub image_id: u64,
    pub vector_buffer: VectorBuffer,
    pub image_complete: bool,
}

#[derive(Debug)]
pub struct ColorizationData {
    pub image_data: ImageData,
    pub point_data: VectorBuffer, //todo! validate type
}
