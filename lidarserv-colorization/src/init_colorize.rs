use crate::color_threads::colorization::point_cloud_colorizer::PointCloudColorizer;
use las::point::Format;
use las::{Builder, Reader, Writer};
use std::io::BufWriter;
use std::path::Path;

pub fn init_colorize() {
    //resize image
    let path_picture = Path::new(
        //"C:/Users/User/OneDrive/Dokumente/Informatikstudium_TUD/6_Semester/Bachelorarbeit/Daten/test_bilder/tina.jpg",
        "C:/Users/User/OneDrive/Dokumente/Informatikstudium_TUD/6_Semester/Bachelorarbeit/Daten/test_bilder/Holzkirchen_DSC02437.JPG",
    );
    let path_cloud_out = Path::new(
        "C:/Users/User/OneDrive/Dokumente/Informatikstudium_TUD/6_Semester/Bachelorarbeit/Daten/test_clouds/example_color_out.las",
    );

    /*
    let path_projected_cloud_out = Path::new(
        "C:/Users/User/OneDrive/Dokumente/Informatikstudium_TUD/6_Semester/Bachelorarbeit/Daten/test_clouds/example_projection_out.las",
    );
     */
    let path_cloud_in = Path::new(
        //"C:/Users/User/OneDrive/Dokumente/Informatikstudium_TUD/6_Semester/Bachelorarbeit/Daten/test_clouds/example_gen_out.las",
        "C:/Users/User/OneDrive/Dokumente/Informatikstudium_TUD/6_Semester/Bachelorarbeit/Daten/test_clouds/example_gen_out.las",
    );

    //let img = read_resize_picture(picture_height, picture_width, path_picture);

    let picture = PointCloudColorizer::example_picture(path_picture);
    /*
    picture
        .colorize(
            create_las_reader(path_cloud_in),
            create_las_writer(path_cloud_out),
        )
        .expect("TODO: panic message");
        
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
