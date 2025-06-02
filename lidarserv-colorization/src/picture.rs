use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, Pixel};
use las::{Color, Point, Reader, Writer};
use lidarserv_common::nalgebra::{ArrayStorage, Const, Isometry3, IsometryMatrix3, Matrix3, Matrix4, OMatrix, Perspective3, Point3, Rotation3, Vector2, Vector3, U4};
use lidarserv_common::query::view_frustum::ViewFrustumQuery;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct Picture {
    ///View frustum for the image
    frustum: ViewFrustumQuery,

    ///the image for the frustum
    dynamic_image: DynamicImage,
}

impl Picture {
    pub fn colorize(
        &self,
        mut cloud_reader: Reader,
        mut cloud_writer: Writer<BufWriter<File>>) -> Result<&'static str, &'static str> {

        let view_projection = self.get_projection();
        println!("view projection: {:?}", view_projection);

        //Iterate over Cloud reader
        cloud_reader.points().for_each(|point_result| {
            let point = point_result.unwrap_or_else(|e| panic!("Failed to read point: {}", e));

            //Find xy position of point in view frustum
            let position = self.find_xy(&point,view_projection).unwrap_or_else(|e| panic!("Failed to find xy position: {}", e));

            //Find Pixel with color to colorize each point
            let cast_position = Vector2::new(position.x as u32, position.y as u32);
            let color = self.find_color(cast_position).unwrap_or_else(|e| match e {
                "Position out of bounds" => {
                    Color::new(0, 50, 50)
                },
                _ => panic!("Failed to find color: {}", e)
            });

            //Colorize the point
            let colorized_point = self.colorize_point(&point, color).unwrap_or_else(|e| panic!("Failed to colorize point: {}", e));

            //write out the point
            cloud_writer.write_point(colorized_point).unwrap_or_else(|e| panic!("Failed to write point: {}", e))

        });

        //close the writer
        cloud_writer.close().unwrap_or_else(|e| panic!("Failed to close writer: {}", e));


        Ok("Hat vlt alles funktioniert")
    }

    fn get_projection(& self) -> OMatrix<f64,Const<4>,U4> {

        //TODO:check legal vector alignment
        /*
        let eye = self.frustum.camera_pos;
        let target = self.frustum.camera_pos + self.frustum.camera_dir;
        let up = self.frustum.camera_up;

        let view_transform = Isometry3::look_at_rh(&eye, &target, &up);
        let aspect_ratio = self.dynamic_image.width() as f64 / self.dynamic_image.height() as f64;
        let proj_frustum = Perspective3::new(aspect_ratio, self.frustum.fov_y, self.frustum.z_near, self.frustum.z_far);

        let view_projection_matrix = proj_frustum.as_matrix() * view_transform.to_matrix();
        let view_projection_matrix_inv = view_transform.inverse().to_matrix() * proj_frustum.inverse();
        //println!("view_projection_matrix: {:?}", view_projection_matrix);
         */

        let view_proj:OMatrix<f64,Const<4>,U4> = OMatrix::new_translation(&Vector3::new(-2000.,-1000.,0.));

        view_proj
    }


    ///Finds xy position for a point inside a view frustum,
    /// by calculating a projection of the view frustum and then applying the projection to the point.
    fn find_xy(&self,point: &Point, view_projection: OMatrix<f64,Const<4>,U4>) -> Result<Vector2<f64>,&'static str> {
        //TODO: Do checks

        //TODO: Check if point is in bounding box


        let projected_point = view_projection.transform_point(&Point3::new(point.x, point.y, point.z));
        
        
      /*
        println!("new point __________________________________________________________________________");
        println!("Das ist meine Position X: {:?}, Y {:?}, Z {:?}", point.x, point.y, point.z);
        println!("Das ist meine Position projiziert X: {:?}, Y {:?}, Z {:?}", projected_point.x, projected_point.y, projected_point.z);
       */
        
        Ok(Vector2::new(projected_point.x, projected_point.y))
    }

    fn find_color(&self,position:Vector2<u32>) -> Result<Color,&'static str> {
        //TODO: Do checks
        if position.x >= self.dynamic_image.width() || position.y >= self.dynamic_image.height() {
            return Err("Position out of bounds")
        }
        
        let pixel_color = self.dynamic_image.get_pixel(position.x, position.y).to_rgb().0;
        Ok(Color::new(pixel_color[0] as u16, pixel_color[1] as u16,pixel_color[2] as u16))
    }

    fn colorize_point(&self, point: &Point, color: Color) -> Result<Point,&'static str> {
        //TODO: Do checks
        //println!("Das ist meine Farbe {:?}", color);
        //let color = Color::new(point.x as u16 % 255 , point.y as u16 % 255 ,point.z as u16 % 255 );
        Ok(Point {
            x:point.x,
            y:point.y,
            z:point.z,
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

    pub fn example_picture(path: &Path) -> Picture {
        let window_size = Vector2::new(1000., 1000.);

        let frustum = ViewFrustumQuery {
            camera_pos: Point3::new(0., 0., 0.),
            camera_dir: Vector3::new(1., 1., 0.),
            camera_up: Vector3::new(0., 0., 1.),
            fov_y: 0.1,
            z_near: 10.0,
            z_far: 100.0,
            window_size: Vector2::new(1000., 1000.),
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

        Picture {
            frustum,
            dynamic_image,
        }
    }
}

