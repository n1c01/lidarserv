mod point_cloud_colorizer;
mod init_colorize;

use crate::point_cloud_colorizer::PointCloudColorizer;
use crate::init_colorize::init_colorize;

use las::point::Format;
use las::{Builder, Reader, Writer};
use std::io::BufWriter;
use std::path::Path;

fn main() {
    //ROS connection Thread
    
    //Processing frustum Thread
    
    //LidarServ query Thread
    
    //LidarServ answer Thread
    
    //Processing colorization Thread
    //init_colorize();
    
    //LidarServ save Thread

}