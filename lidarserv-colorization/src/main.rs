use std::process::ExitCode;
use clap::Parser;
use cli::AppOptions;
use rosrust::api::resolve::get_unused_args;
use log::{debug, error, info};

mod cli;
mod point_cloud_colorizer;
mod init_colorize;


fn main() -> ExitCode {
    // arg parsing
    let args = AppOptions::parse_from(get_unused_args());

    // logger
    simple_logger::init_with_level(args.log_level).unwrap();

    // run
    match run(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            error!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(args: AppOptions) -> Result<(),String>{
    //install signal handler
    
    //ROS connection Thread
    //TODO: ROS Thread, that reads out the images and the positions of the camera
    
    //Processing frustum Thread
    //TODO: Processing Thread, that processes the camera position and calculates the frustum
    
    //LidarServ query Thread
    //TODO: LidarServ Thread, that queries the frustum to retrieve the points from the lidarserv server
    
    //LidarServ answer Thread
    //TODO: LidarServ Thread, that recieves the points from the lidarserv server and sends them to the colorization thread
    
    //Processing colorization Thread
    //TODO: Processing Thread, that colorizes the points
    //init_colorize();
    
    //LidarServ save Thread
    //TODO: LidarServ Thread, that sends the processed points to the lidarserv server
    
    return Ok(());
}