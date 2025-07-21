use std::process::ExitCode;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use anyhow::Context;
use clap::Parser;
use cli::AppOptions;
use rosrust::api::resolve::get_unused_args;
use log::{debug, error, info};
use tokio::sync::broadcast;

mod cli;
mod point_cloud_colorizer;
mod init_colorize;
mod ros;

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
    //install the signal handler 
    //preparing transmitters and receivers for stoping the program when Strg+c is pressed
    let (stop_status_tx, stop_status_rx) = mpsc::channel();
    let (stop_processing_frustum_tx, stop_processing_frustum_rx) = mpsc::channel();
    let (stop_ros_read_tx, stop_ros_read_rx) = mpsc::channel();
    let (stop_lidarserv_query_tx, stop_lidarserv_query_rx) = tokio::sync::broadcast::channel(1); //TODO: Think about channeltype
    let (stop_lidarserv_answer_tx, stop_lidarserv_answer_rx) = mpsc::channel(); 
    let (stop_processing_colorization_tx, stop_processing_colorization_rx) = mpsc::channel();
    let (stop_lidarserv_write_tx, stop_lidarserv_write_rx) = mpsc::channel();
    
    
    let (exit_tx, exit_rx) = channel();
    {
        let exit_tx = exit_tx.clone();
        let mut first_ctrlc = true;
        ctrlc::set_handler(move || {
            if first_ctrlc {
                exit_tx.send(()).ok();
                first_ctrlc = false;
            } else {
                stop_status_tx.send(()).ok();
                stop_processing_frustum_tx.send(()).ok();
                stop_ros_read_tx.send(()).ok();
                stop_lidarserv_query_tx.send(()).ok();
                stop_lidarserv_answer_tx.send(()).ok();
                stop_processing_colorization_tx.send(()).ok();
                stop_lidarserv_write_tx.send(()).ok();
            }
        })
            .context("Failed to install Ctrl+C signal handler.")?;
        info!("Press Ctrl+C to exit.");
    }
    
    
    //ROS read connection Thread
    //sender: images from ROS
    //sender: positions of the camera from ROS
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
    
    //Status Thread
    
    return Ok(());
}