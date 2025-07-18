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

fn run(args: AppOptions){
    //ROS connection Thread
    
    //Processing frustum Thread
    
    //LidarServ query Thread
    
    //LidarServ answer Thread
    
    //Processing colorization Thread
    //init_colorize();
    
    //LidarServ save Thread

}