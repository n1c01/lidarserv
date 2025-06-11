use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, Pixel};
use las::{Color, Point, Reader, Writer};
use lidarserv_common::nalgebra::{Const, Isometry3, OMatrix, Perspective3, Point3, Vector2, Vector3, U4};
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
        mut cloud_writer: Writer<BufWriter<File>>,
        mut proj_cloud_writer: Writer<BufWriter<File>> //TODO: remove when no more need for it
    ) -> Result<&'static str, &'static str> {

        let view_projection = self.get_projection();

        //Iterate over Cloud reader
        for point_result in cloud_reader.points(){
        //cloud_reader.points().for_each(|point_result| {
            let point = point_result.unwrap_or_else(|e| panic!("Failed to read point: {}", e));

            let position;
            let color;

            //Find xy position of point in view frustum
            match self.find_xy(&point,view_projection, &mut proj_cloud_writer) {
                Err(e) => {
                    match e {
                        //position is out of frame in relation to the camera (behind the camera)
                        "Position out of bounds (z-direction)" =>{
                            //skip points that are not represented by the picture
                            continue;
                        }
                        &_ => { panic!("Failed to find xy position: {:?}", e) }
                    }
                }
                Ok(p) => { position = p; }
            }

            //Find the Pixel with color to colorize each point
            match self.find_color(position) {
                Err(error) => {
                    match error {
                        "Position out of bounds (xy-direction)" => {
                            //Don't return Points that are not covered by the picture
                            //continue;
                            //TODO: in future (when only accessing relevant points) this should probably return a error
                            //TODO: remove testwise default color for points outside the picture
                            color = Color::new(20,50,0);
                        }
                        &_ => { panic!("{:?}", error) }
                    }
                }
                Ok(c) => { color = c; }
            }
            //Colorize the point
            let colorized_point = self.colorize_point(&point, color).unwrap_or_else(|e| panic!("Failed to colorize point: {}", e));

            //write out the point
            cloud_writer.write_point(colorized_point).unwrap_or_else(|e| panic!("Failed to write point: {}", e))

        }

        //close the writer
        cloud_writer.close().unwrap_or_else(|e| panic!("Failed to close writer: {}", e));        
        proj_cloud_writer.close().unwrap_or_else(|e| panic!("Failed to close projection writer: {}", e));





        Ok("Hat vlt alles funktioniert")
    }

    fn get_projection(& self) -> OMatrix<f64,Const<4>,U4> {

        //TODO:check legal vector alignment

        let eye = self.frustum.camera_pos;
        let target = self.frustum.camera_pos + self.frustum.camera_dir;
        let up = self.frustum.camera_up;

        let view_transform = Isometry3::look_at_rh(&eye, &target, &up);
        let aspect_ratio = self.dynamic_image.width() as f64 / self.dynamic_image.height() as f64;
        let proj_frustum = Perspective3::new(aspect_ratio, self.frustum.fov_y, self.frustum.z_near, self.frustum.z_far);

        let view_projection_matrix = proj_frustum.as_matrix() * view_transform.to_matrix();
        //let view_projection_matrix_inv = view_transform.inverse().to_matrix() * proj_frustum.inverse();
        println!("view_projection_matrix: {:?}", view_projection_matrix);

/*
        let translation:OMatrix<f64,Const<4>,U4> = OMatrix::new_translation(&Vector3::new(-1000.,-1000.,0.));
        let rotation:OMatrix<f64,Const<4>,U4> = OMatrix::new_rotation_wrt_point(Vector3::new(0.1,0.1,0.1),Point3::new(0.,0.,0.));
        let scaling:OMatrix<f64,Const<4>,U4> = OMatrix::new_scaling(0.5);

        let test_scale:OMatrix<f64,Const<4>,U4> = OMatrix::from_columns(& [
            Vector4::new(1.0f64, 0.0f64, 0.0f64, 0.0f64),
            Vector4::new(0.0f64,1.0f64,0.0f64,0.0f64),
            Vector4::new(0.0f64,0.0f64,1.0f64,0.0f64),
            Vector4::new(-100.0f64,-100.0f64,-100.0f64,1.0f64)]);


 */
        let view_proj = view_projection_matrix;

        println!("view projection: {:?}", view_proj);


        view_proj
    }


    ///Finds xy position for a point inside a view frustum,
    /// by calculating a projection of the view frustum and then applying the projection to the point.
    fn find_xy(&self,point: &Point, view_projection: OMatrix<f64,Const<4>,U4>, projection_cloud_writer: &mut Writer<BufWriter<File>>) -> Result<Vector2<f64>,&'static str> {
        //TODO: Do checks

        //TODO: Check if point is in bounding box


        let projected_point = view_projection.transform_point(&Point3::new(point.x, point.y, point.z));

        let saved_point = Point {
            x:point.x,
            y:point.y,
            z:point.z,
            intensity: 9,
            color: Some(Color::new(60,10,100)),
            ..Default::default()
        };

        if projected_point.z < 0. {
            return Err("Position out of bounds (z-direction)")
        }
        projection_cloud_writer.write_point(saved_point).unwrap_or_else(|e| panic!("Failed to write point: {}", e));



        /*
          println!("new point __________________________________________________________________________");
          println!("Das ist meine Position X: {:?}, Y {:?}, Z {:?}", point.x, point.y, point.z);
          println!("Das ist meine Position projiziert X: {:?}, Y {:?}, Z {:?}", projected_point.x, projected_point.y, projected_point.z);
         */

        Ok(Vector2::new(projected_point.x, projected_point.y))
    }

    fn find_color(&self,position:Vector2<f64>) -> Result<Color,&'static str> {
        //TODO: Do checks
        if position.x >= self.dynamic_image.width() as f64
            || position.y >= self.dynamic_image.height() as f64
            || position.x < 0.
            || position.y < 0.
        {
            return Err("Position out of bounds (xy-direction)")
        }


        //cast position to match image
        let position = Vector2::new(position.x as u32, position.y as u32);


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
            camera_pos: Point3::new(-1., -4., 0.),
            camera_dir: Vector3::new(2., -6., 0.1),
            camera_up: Vector3::new(0., 0., 1.),
            fov_y: 0.001,
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

        Picture {
            frustum,
            dynamic_image,
        }
    }
}

