use crate::color_threads::ImageData;
use pasture_core::containers::VectorBuffer;

pub mod point_cloud_colorizer;
pub mod t4_managing_colorization;

#[derive(Debug)]
pub struct ColorizationData {
    pub image_data: ImageData,
    pub point_data: VectorBuffer, //todo! validate type
}
