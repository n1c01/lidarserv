use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, Pixel};
use las::{Color, Point, Reader, Writer};
use lidarserv_common::nalgebra::{Const, Isometry3, OMatrix, Perspective3, Point3, RowVector4, Vector2, Vector3, U4};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;


/// The picture struct holds a view frustum and a corresponding dynamic image
pub struct PointCloudColorizer {
    ///View frustum for the image
    frustum: ViewFrustumQuery,

    ///the image for the frustum
    dynamic_image: DynamicImage,
    
}

/// The implementation of the picture struct.
/// The picture struct is used to project a picture onto a point cloud
impl PointCloudColorizer {
    /// Colorizes a point cloud (cloud_reader) with the picture (self) and outputs it (cloud_writer)
    pub fn colorize(&self,
                    mut cloud_reader: Reader,
                    mut cloud_writer: Writer<BufWriter<File>>,
    ) -> Result<&'static str, &'static str> {
        //Get the projection matrix to transform the points to the picture frustum
        let view_projection = self.get_projection();

        //Iterate over Cloud reader (colorize each point)
        for point_result in cloud_reader.points() {
            let point = point_result.unwrap_or_else(|e| panic!("Failed to read point: {}", e));

            let position;
            let color;

            //Find xy position of point in view frustum of the picture
            match self.find_xy(&point, view_projection) {
                Err(e) => {
                    match e {
                        //position is out of frame in relation to the camera (behind the camera)
                        //TODO: check if this is the case
                        "Position out of bounds (z-direction)" => {
                            //skip points that are not represented by the picture
                            continue;
                        }
                        &_ => {
                            panic!("Failed to find xy position Error: {:?}", e)
                        }
                    }
                }
                Ok(p) => {
                    position = p;
                }
            }

            //Find the Pixel corresponding to the point
            match self.find_color(position) {
                Err(error) => {
                    match error {
                        "Position out of bounds (x > picture)" => {
                            //Don't return Points that are not covered by the picture
                            //continue;
                            //TODO: in future (when only accessing relevant points) this should probably return a error
                            //TODO: remove testwise default color for points outside the picture
                            color = Color::new(0, 250, 0);
                        }
                        "Position out of bounds (x < 0)" => {
                            //Don't return Points that are not covered by the picture
                            //continue;
                            color = Color::new(0, 100, 0);
                        }
                        "Position out of bounds (y > picture)" => {
                            //Don't return Points that are not covered by the picture
                            //continue;
                            color = Color::new(0, 0, 250);
                        }
                        
                        "Position out of bounds (y < 0)" => {
                            //Don't return Points that are not covered by the picture
                            //continue;
                            color = Color::new(0, 0, 100);
                        }
                        &_ => {
                            panic!("{:?}", error)
                        }
                    }
                }
                Ok(c) => {
                    color = c;
                }
            }
            //Colorize the point
            let colorized_point = self
                .colorize_point(&point, color)
                .unwrap_or_else(|e| panic!("Failed to colorize point: {}", e));

            //write out the point
            cloud_writer
                .write_point(colorized_point)
                .unwrap_or_else(|e| panic!("Failed to write point: {}", e))
        }

        //close the writer
        cloud_writer
            .close()
            .unwrap_or_else(|e| panic!("Failed to close writer: {}", e));
        
        Ok("Done")
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

        let view_projection_matrix: OMatrix<f64, Const<4>, U4> = proj_frustum.as_matrix() * view_transform.to_matrix();
        let view_projection_matrix_inv = proj_frustum.inverse() * view_transform.inverse().to_matrix() ;
        println!("view_projection_matrix: {:?}", view_projection_matrix);

        let translation:OMatrix<f64,Const<4>,U4> = OMatrix::new_translation(&Vector3::new(-1000.,-1000.,0.));
        let rotation:OMatrix<f64,Const<4>,U4> = OMatrix::new_rotation_wrt_point(Vector3::new(0.1,0.1,0.1),Point3::new(0.,0.,0.));
        let scaling:OMatrix<f64,Const<4>,U4> = OMatrix::new_scaling(0.5);

        let test_scale:OMatrix<f64,U4,Const<4>> = OMatrix::from_rows(& [
            RowVector4::new(-0.083455190734908813, -0.99480626956417306, -0.05827278245644181, 6.2100728800843541 ),
            RowVector4::new(0.88182502907790672, -0.046488086288672265, -0.46927974165199771, 16.725405857514271 ),
            RowVector4::new(0.46413343903574611, -0.090550228431698312, 0.88112473969343208, -58.819855654855907 ),
            RowVector4::new(0., 0., 0., 1.)]);
       
        /*
        let test_scale:OMatrix<f64,U4,Const<4>> = OMatrix::from(
            [-0.083455190734908813, -0.99480626956417306, -0.05827278245644181, 6.2100728800843541,
            0.88182502907790672, -0.046488086288672265, -0.46927974165199771, 16.725405857514271,
            0.46413343903574611, -0.090550228431698312, 0.88112473969343208, -58.819855654855907,
            0., 0., 0., 1.]);
            
         */
        /*
        let test_scale:OMatrix<f64,U4,Const<4>> = OMatrix::from_columns(& [
            Vector4::new(-0.083455190734908813, -0.99480626956417306, -0.05827278245644181, 6.2100728800843541 ),
            Vector4::new(0.88182502907790672, -0.046488086288672265, -0.46927974165199771, 16.725405857514271 ),
            Vector4::new(0.46413343903574611, -0.090550228431698312, 0.88112473969343208, -58.819855654855907 ),
            Vector4::new(0., 0., 0., 1.)]);
         */
        println!("test_scale: {:?}", test_scale);



        //Holzkirchen_DSC02437 matrix:
        // -0.083455190734908813 -0.99480626956417306 -0.05827278245644181 6.2100728800843541
        // 0.88182502907790672 -0.046488086288672265 -0.46927974165199771 16.725405857514271
        // 0.46413343903574611 -0.090550228431698312 0.88112473969343208 -58.819855654855907
        // 0 0 0 1
        let view_proj = test_scale;

        //println!("view projection: {:?}", view_proj);

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
        let x = projected_point.x.min(100000.).max(-100000.);
        let y = projected_point.y.min(100000.).max(-100000.);
        let z = projected_point.z.min(100000.).max(-100000.);

        //println!("X: {:?}, Y: {:?}, Z: {:?}", x, y, z);

        let saved_point = Point {
            x,
            y,
            z,
            intensity: 9,
            color: Some(Color::new(60, 10, 100)),
            ..Default::default()
        };

        /*
        if projected_point.z < 0. {
            return Err("Position out of bounds (z-direction)");
        }   
        */
        /*
        projection_cloud_writer
            .write_point(saved_point)
            .unwrap_or_else(|e| panic!("Failed to write point: {}", e));
            
         */

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
        if position.x >= self.dynamic_image.width() as f64{
            return Err("Position out of bounds (x > picture)");
        }
        else if position.x < 0.
        {
            return Err("Position out of bounds (x < 0)");
        }
        if position.y >= self.dynamic_image.height() as f64
        {
            return Err("Position out of bounds (y > picture)");
        }
        else if position.y < 0.
        {
            return Err("Position out of bounds (y < 0)");
        }

        //cast position to match image
        let position = Vector2::new(position.x as u32, position.y as u32);

        let pixel_color = self
            .dynamic_image
            .get_pixel(position.x, position.y)
            .to_rgb()
            .0;
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
        /*
        let mut colorized_point = point.clone();
        colorized_point.color = Some(color);
        Ok(colorized_point)

         */
    }
    
    /// Creates a picture struct from a path to a picture
    /// # attributes
    /// - path: The path to the picture
    /// # returns
    /// The picture struct
    /// # errors
    /// - "Failed to read image"
    /// - "Failed to decode image"

    pub fn example_picture(path: &Path) -> PointCloudColorizer {
        let window_size = Vector2::new(1000., 1000.);

        let frustum = ViewFrustumQuery {
            camera_pos: Point3::new(-1., -4., 0.),
            camera_dir: Vector3::new(2.5, -3., -2.3),
            camera_up: Vector3::new(0., 0., 1.),
            fov_y: 0.003,
            z_near: 0.0,
            z_far: 1000.0,
            window_size,
            max_distance: 1000.0,
        };
        let dynamic_image = ImageReader::open(path)
            .unwrap_or_else(|e| panic!("Failed to read image: {}", e))
            .decode()
            .unwrap_or_else(|e| panic!("Failed to decode image: {}", e))
            .resize(
                window_size.x as u32,
                window_size.y as u32,
                FilterType::Gaussian,
            );

        PointCloudColorizer {
            frustum,
            dynamic_image,
        }
    }
}
