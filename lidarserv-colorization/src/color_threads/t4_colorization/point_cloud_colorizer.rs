use image::{DynamicImage, GenericImageView, Pixel};
use las::{Color, Point};
use lidarserv_common::nalgebra::{
    Const, Isometry3, OMatrix, Perspective3, Point3, RowVector4, Vector2, Vector3, U4,
};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use log::{debug, warn};
use pasture_core::containers::{BorrowedBuffer, BorrowedMutBuffer, VectorBuffer};
use pasture_core::layout::attributes::{COLOR_RGB, POSITION_3D};

/// The picture struct holds a view frustum and a corresponding dynamic image
pub struct PointCloudColorizer {
    ///View frustum for the image
    pub(crate) frustum: ViewFrustumQuery,

    ///the image for the frustum
    pub(crate) dynamic_image: DynamicImage,
}

/// The implementation of the picture struct.
/// The picture struct is used to project a picture onto a point cloud
impl PointCloudColorizer {
    /// Colorizes a point cloud (cloud_reader) with the picture (self) and outputs it (cloud_writer)
    pub fn colorize(
        &self,
        mut vector_buffer: VectorBuffer,
    ) -> Result<VectorBuffer, &'static str> {

        //Get the projection matrix to transform the points to the picture frustum
        //debug!("PointCloudColorizer: Getting projection matrix");
        let view_projection = self.get_projection();

        //Iterate over pointcloud (colorize each point)
        if !vector_buffer.point_layout().has_attribute(&POSITION_3D) {
            return Err("Pointcloud does not have a position attribute");
        } else if !vector_buffer.point_layout().has_attribute(&COLOR_RGB) {
            return Err("Pointcloud does not have a color attribute");
        };

        //TODO! Use full position with right casting.
        let mut position: Vec<u8> = vec![0; POSITION_3D.size() as usize];
        let mut color_raw: Vec<u8> = vec![0; COLOR_RGB.size() as usize];
        for i in 0..vector_buffer.len() {
            //todo! check endianess
            //convert position to f64 then to Point struct
            BorrowedBuffer::get_attribute(&vector_buffer, &POSITION_3D, i, &mut position);
            let x = f64::from_le_bytes(position[0..8].try_into().unwrap());
            let y = f64::from_le_bytes(position[8..16].try_into().unwrap());
            let z = f64::from_le_bytes(position[16..24].try_into().unwrap());
            let point = Point { x, y, z, ..Default::default() };

            //convert the previous color to u16 then to Color struct
            BorrowedBuffer::get_attribute(&vector_buffer, &COLOR_RGB, i, &mut color_raw);
            let r = u16::from_le_bytes(color_raw[0..2].try_into().unwrap());
            let g = u16::from_le_bytes(color_raw[2..4].try_into().unwrap());
            let b = u16::from_le_bytes(color_raw[4..6].try_into().unwrap());
            //todo handle previous colors
            let _color = Color::new(r, g, b);

            /*
            debug!("position that is being colorized: {:?}", position);
            debug!("color before {:?}", color);
            debug!("point that is being colorized: {:?}", point);
             */

            let color_point = match self.process_point(&point, view_projection) {
                Ok(data) => {
                    data
                }
                Err("Position out of bounds (z-direction)") => {
                    warn!("Position out of bounds (z-direction)");
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            };
            let result_color: Vector3<u16> = match color_point.color {
                Some(c) => {
                    //debug!("point: [{:?},{:?},{:?}], has color: {:?}",point.x, point.y, point.z, c);
                    Vector3::new(c.red, c.green, c.blue)
                },
                None => {
                    //todo! handle differently e.g. by continuing
                    debug!("Point [{:?},{:?},{:?}] has no color, setting to white", point.x, point.y, point.z);
                    Vector3::new(255, 255, 255)
                },
            };
            unsafe {
                let color_bytes: &[u8] = std::slice::from_raw_parts(
                    &result_color as *const Vector3<u16> as *const u8,
                    size_of::<Vector3<u16>>(),
                );
                vector_buffer.set_attribute(&COLOR_RGB, i,color_bytes);
            }
        }
        Ok(vector_buffer)
    }

    fn process_point(
        &self,
        point: &Point,
        view_projection: OMatrix<f64, Const<4>, U4>,
    ) -> Result<Point, &'static str> {
        //Find xy position of point in view frustum of the picture
        let position= match self.find_xy(&point, view_projection) {
            Ok(p) => {
                p
            }
            Err("Position out of bounds (z-direction)") => {
                return Err("Position out of bounds (z-direction)");
            }
            Err(e) => {
                return Err(e);
            }
        };
        //Find the Pixel corresponding to the point
        let color:Color = match self.find_color(position) {
            Err(error) => {
                match error {
                    "Position out of bounds (x > picture)" => {
                        //Don't return Points that are not covered by the picture
                        //continue;
                        //TODO: in future (when only accessing relevant points) this should probably return a error
                        //TODO: remove testwise default color for points outside the picture
                        Color::new(0, 250, 0)
                    }
                    "Position out of bounds (x < 0)" => {
                        //Don't return Points that are not covered by the picture
                        //continue;
                        Color::new(0, 100, 0)
                    }
                    "Position out of bounds (y > picture)" => {
                        //Don't return Points that are not covered by the picture
                        //continue;
                        Color::new(0, 0, 250)
                    }

                    "Position out of bounds (y < 0)" => {
                        //Don't return Points that are not covered by the picture
                        //continue;
                        Color::new(0, 0, 100)
                    }
                    &_ => {
                        panic!("{:?}", error)
                    }
                }
            }
            Ok(c) => {
                debug!("\n\nreal color returned 🥳🎉: {:?}\n", c);
                c
            }
        };

        //Colorize the point
        let colorized_point = self
            .colorize_point(&point, color)
            .unwrap_or_else(|e| panic!("Failed to colorize point: {}", e));

        Ok(colorized_point)
    }

    ///Get the projection matrix for the picture
    /// # attributes
    /// - self: The picture struct
    ///
    /// # returns
    /// The projection matrix used to transform the points to the picture frustum
    fn get_projection(&self) -> OMatrix<f64, Const<4>, U4> {
        //TODO:check legal vector alignment

        let eye = self.frustum.camera_pos;
        let target = self.frustum.camera_pos + self.frustum.camera_dir;
        let up = self.frustum.camera_up;

        let view_transform = Isometry3::look_at_rh(&eye, &target, &up);
        let aspect_ratio = self.dynamic_image.width() as f64 / self.dynamic_image.height() as f64;
        let proj_frustum = Perspective3::new(
            aspect_ratio,
            self.frustum.fov_y,
            self.frustum.z_near,
            self.frustum.z_far,
        );

        let _view_projection_matrix: OMatrix<f64, Const<4>, U4> = proj_frustum.as_matrix() * view_transform.to_matrix();
        let _view_projection_matrix_inv = proj_frustum.inverse() * view_transform.inverse().to_matrix();

        let _translation: OMatrix<f64, Const<4>, U4> = OMatrix::new_translation(&Vector3::new(-1000., -1000., 0.));
        let _rotation: OMatrix<f64, Const<4>, U4> = OMatrix::new_rotation_wrt_point(Vector3::new(0.1, 0.1, 0.1), Point3::new(0., 0., 0.));
        let _scaling: OMatrix<f64, Const<4>, U4> = OMatrix::new_scaling(0.5);

        let test_scale: OMatrix<f64, U4, Const<4>> = OMatrix::from_rows(&[
            RowVector4::new(
                -0.083455190734908813,
                -0.99480626956417306,
                -0.05827278245644181,
                6.2100728800843541,
            ),
            RowVector4::new(
                0.88182502907790672,
                -0.046488086288672265,
                -0.46927974165199771,
                16.725405857514271,
            ),
            RowVector4::new(
                0.46413343903574611,
                -0.090550228431698312,
                0.88112473969343208,
                -58.819855654855907,
            ),
            RowVector4::new(0., 0., 0., 1.),
        ]);
        //todo change to real projection matrix
        let view_proj = test_scale;

        view_proj
    }

    /// Finds xy position for a point inside a view frustum,
    /// by calculating a projection of the view frustum and then applying the projection to the point.
    ///
    /// # attributes
    /// - self
    /// - The point of which the position should be found
    /// - THe projection matrix to project the point into the picture frustum
    /// # returns
    /// 2D vector with the position of the point in the picture
    fn find_xy(
        &self,
        point: &Point,
        view_projection: OMatrix<f64, Const<4>, U4>,
    ) -> Result<Vector2<f64>, &'static str> {
        //TODO: Do checks

        //TODO: Check if point is in bounding box

        let projected_point =
            view_projection.transform_point(&Point3::new(point.x, point.y, point.z));

        //xyz values cut at extreme high or low values
        let _x = projected_point.x.min(100000.).max(-100000.);
        let _y = projected_point.y.min(100000.).max(-100000.);
        let _z = projected_point.z.min(100000.).max(-100000.);

        Ok(Vector2::new(projected_point.x, projected_point.y))
    }

    /// finds the color of a point in the picture
    /// # attributes
    /// - self: The picture struct
    /// - position: The position of the point in the picture
    /// # returns
    /// The color of the point in the picture
    /// # errors
    /// - "Position out of bounds (x > picture)"
    /// - "Position out of bounds (x < 0)"
    /// - "Position out of bounds (y > picture)"
    fn find_color(&self, position: Vector2<f64>) -> Result<Color, &'static str> {
        //TODO: Do checks
        if position.x >= self.dynamic_image.width() as f64 {
            return Err("Position out of bounds (x > picture)");
        } else if position.x < 0. {
            return Err("Position out of bounds (x < 0)");
        }
        if position.y >= self.dynamic_image.height() as f64 {
            return Err("Position out of bounds (y > picture)");
        } else if position.y < 0. {
            return Err("Position out of bounds (y < 0)");
        }

        //cast position to match image
        let position = Vector2::new(position.x as u32, position.y as u32);

        let pixel_color = self
            .dynamic_image
            .get_pixel(position.x, position.y)
            .to_rgb()
            .0;
        debug!("pixel_color: {:?} of postition X: {:?}, Y: {:?}", pixel_color,position.x, position.y);
        Ok(Color::new(
            pixel_color[0] as u16,
            pixel_color[1] as u16,
            pixel_color[2] as u16,
        ))
    }

    /// Colorizes a point
    /// # attributes
    /// - self: The picture struct
    /// - point: The point to colorize
    /// - color: The color to colorize the point with
    /// # returns
    /// The colorized point
    fn colorize_point(&self, point: &Point, color: Color) -> Result<Point, &'static str> {
        //TODO: Do checks
        //println!("Das ist meine Farbe {:?}", color);
        //let color = Color::new(point.x as u16 % 255, point.y as u16 % 255, point.z as u16 % 255 );
        Ok(Point {
            x: point.x,
            y: point.y,
            z: point.z,
            intensity: 9,
            color: Some(color),
            ..Default::default()
        })
        //TODO: Fix so that cloneing is possible (right point format type)
    }
}
