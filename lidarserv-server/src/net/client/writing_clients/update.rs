use async_trait::async_trait;
use nalgebra::{Point3, Vector3};
use pasture_core::containers::{BorrowedBuffer, BorrowedBufferExt, BorrowedMutBufferExt, InterleavedBuffer, InterleavedBufferMut, MakeBufferFromLayout, OwningBuffer, VectorBuffer};
use pasture_core::layout::PointLayout;
use lidarserv_common::geometry::coordinate_system::{CoordinateSystem, CoordinateSystemError};
use lidarserv_common::geometry::position::{Component, WithComponentTypeOnce};
use lidarserv_common::tracy_client::span;
use crate::net::client::writing_clients::write::WriteClient;
use crate::net::LidarServerError;
use crate::net::protocol::messages::Header;

#[async_trait]
pub trait Update {
    async fn update_points_global_coordinates(&mut self, points: &VectorBuffer) -> Result<(), LidarServerError>;
    async fn update_points_local_coordinates(&mut self, points: &VectorBuffer, ) -> Result<(), LidarServerError>;
    async fn update_raw_point_data(&mut self, data: &[u8]) -> Result<(), LidarServerError>;

}

#[async_trait]
impl Update for WriteClient{
    async fn update_points_global_coordinates(&mut self, points: &VectorBuffer) -> Result<(), LidarServerError> {

        let global_position_attr = f64::position_attribute();
        let target_layout = PointLayout::from_attributes(&self.attributes);
        let mut target_buffer = VectorBuffer::new_from_layout(target_layout.clone());
        target_buffer.resize(points.len());
        for attribute in &self.attributes {
            if *attribute == global_position_attr {
                struct Wct<'a> {
                    src_buffer: &'a VectorBuffer,
                    target_buffer: &'a mut VectorBuffer,
                    coordinate_system: CoordinateSystem,
                }
                impl WithComponentTypeOnce for Wct<'_> {
                    type Output = Result<(), CoordinateSystemError>;

                    fn run_once<C: Component>(self) -> Self::Output {
                        let Self {
                            target_buffer,
                            src_buffer,
                            coordinate_system,
                        } = self;

                        let global_positions =
                            src_buffer.view_attribute::<Vector3<f64>>(&f64::position_attribute());
                        let mut local_positions = target_buffer
                            .view_attribute_mut::<C::PasturePrimitive>(&C::position_attribute());
                        for i in 0..src_buffer.len() {
                            let pos_global: Point3<f64> = global_positions.at(i).into();
                            let pos_local: Point3<C> =
                                coordinate_system.encode_position(pos_global)?;
                            local_positions.set_at(i, C::position_to_pasture(pos_local));
                        }
                        Ok(())
                    }
                }
                let result = Wct {
                    src_buffer: points,
                    target_buffer: &mut target_buffer,
                    coordinate_system: self.coordinate_system,
                }
                    .for_layout_once(&target_layout);
                if let Err(e) = result {
                    return Err(LidarServerError::Client(format!("{e}")));
                }
            } else {
                let Some(src_attr_member) = points.point_layout().get_attribute(attribute) else {
                    return Err(LidarServerError::Client(format!(
                        "Missing attribute: {attribute:?}"
                    )));
                };
                let dst_attr_member = target_buffer
                    .point_layout()
                    .get_attribute(attribute)
                    .expect("created like this")
                    .clone();
                let src_view = points.view_raw_attribute(src_attr_member);
                let mut dst_view = target_buffer.view_raw_attribute_mut(&dst_attr_member);
                for i in 0..points.len() {
                    dst_view[i].copy_from_slice(&src_view[i]);
                }
            }
        }

        self.update_points_local_coordinates(&target_buffer).await
    }

    async fn update_points_local_coordinates(&mut self, points: &VectorBuffer) -> Result<(), LidarServerError> {
        // check attributes
        for attr in &self.attributes {
            if !points.point_layout().has_attribute(attr) {
                return Err(LidarServerError::Client(format!(
                    "Missing point attribute {attr:?}"
                )));
            }
        }

        let mut data = Vec::new();
        let _s1 = span!("CaptureDeviceClient::insert_points_local_coordinates encode point data");
        if let Err(e) = self.codec.instance().write_points(points, &mut data) {
            return Err(LidarServerError::Client(format!("Encoding error: {e}")));
        }
        drop(_s1);
        self.update_raw_point_data(&data).await
    }

    async fn update_raw_point_data(&mut self, data: &[u8]) -> Result<(), LidarServerError> {
        self.connection
            .write_message(&Header::UpdatePoints, data)
            .await?;

        Ok(())
    }
}