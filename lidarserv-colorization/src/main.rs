mod picture;
use crate::picture::Picture;

use image::{GenericImageView, Pixel};
use las::point::Format;
use las::{Builder, Reader, Writer};
use std::io::BufWriter;
use std::path::Path;

fn main() {
    //resize image
    let path_picture = Path::new(
        "C:/Users/User/OneDrive/Dokumente/Informatikstudium_TUD/6_Semester/Bachelorarbeit/Daten/test_bilder/farbverlauf.jpg",
    );
    let path_cloud_out = Path::new(
        "C:/Users/User/OneDrive/Dokumente/Informatikstudium_TUD/6_Semester/Bachelorarbeit/Daten/test_clouds/example_color_out.las",
    );
    let path_cloud_in = Path::new(
        "C:/Users/User/OneDrive/Dokumente/Informatikstudium_TUD/6_Semester/Bachelorarbeit/Daten/test_clouds/example_gen_out.las",
    );

    //let img = read_resize_picture(picture_height, picture_width, path_picture);

    let picture = Picture::example_picture(path_picture);
    picture
        .colorize(
            create_las_reader(path_cloud_in),
            create_las_writer(path_cloud_out),
        )
        .expect("TODO: panic message");

    /*

    let img = read_resize_picture(picture_height, picture_width, path_picture);




    //create point cloud handlers
    let mut reader = create_las_reader(path_cloud_in);
    let mut writer = create_las_writer(path_cloud_out);


    //read each point and project color from image
    reader.points().for_each(|point| {
        let unwrapped_point = point.unwrap();

        let colorization_result = colorize_point(&unwrapped_point, &img);
        match colorization_result {
            Ok(colorized_point) => {
                writer.write_point(colorized_point).unwrap_or_else(|e| panic!("Failed to write point: {}", e))
            },
            Err("Point out of bounds") => {}, //Points out of bound is expected here, when image not showing all points
            Err(e) => println!("Error: {}", e)
        }
    });

    //close writer
    writer.close().unwrap_or_else(|e| panic!("Failed to close file: {}", e));



     */
}

fn create_las_reader(path: &Path) -> Reader {
    Reader::from_path(path).unwrap_or_else(|e| panic!("Failed to read file: {}", e))
}
fn create_las_writer(path: &Path) -> las::Writer<BufWriter<std::fs::File>> {
    let mut builder = Builder::from((1, 4));
    builder.point_format = Format::new(2).unwrap();
    Writer::from_path(path, builder.into_header().unwrap())
        .unwrap_or_else(|e| panic!("Failed to write file: {}", e))
}

/*
fn read_resize_picture(picture_width: u32, picture_height: u32, path: &Path) -> DynamicImage {
    ImageReader::open(path)
        .unwrap_or_else(|e| panic!("Failed to read image: {}", e))
        .decode()
        .unwrap_or_else(|e| panic!("Failed to decode image: {}", e))
        .resize(picture_width, picture_height, FilterType::Nearest)
}


//Hier muss für jeden Punkt entschieden werden welche Farbe er aufgrund eines Bildes bekommt.
fn colorize_point(point: &Point, img: &DynamicImage) -> Result<Point, &'static str>{
    if point.x <= 0. || point.x >= img.width() as f64 {
        //println!("Dimensions Point to wide: point x {:?}, img width: {:?}", point.x, img.width());
        return Err("Point out of bounds"); }
    if point.y <= 0. || point.y >= img.height() as f64 {
        //println!("Dimensions Point to wide: point y {:?}, img width: {:?}", point.y, img.height());
        return Err("Point out of bounds"); }

    let pixel = img.get_pixel(point.x as u32, point.y as u32);
    let color_cannels = pixel.to_rgb();
    let color:Color = Color{red: color_cannels.0[0] as u16,green:color_cannels.0[1] as u16,blue:color_cannels.0[2] as u16};
    let opt_color = Some(color);


    Ok(Point {
        x:point.x,
        y:point.y,
        z:9.,
        intensity: 9,
        color: opt_color,
        ..Default::default()
    })
}

 */
