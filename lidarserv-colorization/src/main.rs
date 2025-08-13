use crate::color_threads::processing_frustum;
use crate::color_threads::processing_frustum::process_frustum_thread;
use crate::color_threads::send_frustum::send_frustum_thread;
use crate::color_threads::ros::ros_thread;
use crate::color_threads::status::{status_thread, Status};
use anyhow::Result;
use anyhow::{Context, Error};
use clap::Parser;
use cli::AppOptions;
use log::{debug, error, info};
use rosrust::api::resolve::get_unused_args;
use std::fmt::{Debug, Display};
use std::process::ExitCode;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::{mpsc, Arc};
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

mod cli;
mod color_threads;
mod init_colorize;
mod point_cloud_colorizer;

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
fn run(args: AppOptions) -> Result<(), Error> {
    //install the signal handler
    //preparing transmitters and receivers for stoping the program when Strg+c is pressed
    let (stop_status_tx, stop_status_rx) = mpsc::channel();
    let (stop_processing_frustum_tx, stop_processing_frustum_rx) = mpsc::channel();
    let (stop_ros_read_tx, stop_ros_read_rx) = mpsc::channel();
    let (stop_lidarserv_query_tx, stop_lidarserv_query_rx) = tokio::sync::broadcast::channel(1); //only one message sent therefore capacity one
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
        .expect("Failed to initialize Ctrl+C Handler");
        info!("Press Ctrl+C to exit.");
    }

    //ROS read connection Thread
    //sender: images from ROS
    //sender: positions of the camera from ROS
    // ROS Thread, that reads out the images and the positions of the camera
    let (commands_tx, commands_rx) = mpsc::channel();
    let (image_data_tx, image_data_rx) = mpsc::channel(); //Channel for the image data of the camera.
    let status = Arc::new(Status::default());
    let status1 = Arc::clone(&status);
    let join_ros = {
        let exit_tx = exit_tx.clone();
        let args1 = args.clone();
        thread::spawn(move || {
            ros_thread(args1, commands_rx, image_data_tx, status1).log_error();
            exit_tx.send(()).ok()
        })
    };

    //Processing frustum Thread
    //Processing Thread, that processes the camera position and calculates the frustum
    let (image_id_and_frustum_data_tx, image_id_and_frustum_data_rx) = mpsc::channel();
    let status2 = Arc::clone(&status);
    let args2 = args.clone();

    let join_processing_frustum = {
        thread::spawn(move || {
            process_frustum_thread(args2, image_data_rx, image_id_and_frustum_data_tx, status2).log_error();
        })
    };

    //LidarServ query Thread
    //TODO: LidarServ Thread, that queries the frustum to retrieve the points from the lidarserv server
    let (points_tx, points_rx) = mpsc::channel(); //todo!(rename more precise)
    let status3 = Arc::clone(&status);
    let args3 = args.clone();
    let join_lidarserv_query = {
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(send_frustum_thread(args3, image_id_and_frustum_data_rx,points_tx,status3)).log_error();
        })
    };

    //LidarServ answer Thread
    //TODO: LidarServ Thread, that recieves the points from the lidarserv server and sends them to the colorization thread
    let join_lidarserv_answer = {
        thread::spawn(move || {
            //todo!("lidarserv answer thread")
        })
    };

    //Processing colorization Thread
    //TODO: Processing Thread, that colorizes the points
    let join_colorization = {
        thread::spawn(move || {
            //todo!("colorization thread")
            //init_colorize();
        })
    };

    //LidarServ store Thread
    //TODO: LidarServ Thread, that sends the processed points to the lidarserv server
    let join_lidarserv_store = {
        thread::spawn(move || {
            //todo!("lidarserv store thread")
        })
    };

    //Status Thread
    let join_status = {
        let status = Arc::clone(&status);
        thread::spawn(move || {
            status_thread(status, stop_status_rx);
        })
    };

    // wait for exit (user pressed ctrl+c, or one of the thread terminated unexpectedly)
    exit_rx.recv().unwrap();
    let term = console::Term::stdout();
    if !term.features().is_attended() {
        info!("Shutting down...");
    } else {
        term.write_line("[⏹] Shutting down...").unwrap();
        term.move_cursor_up(1).ok();
    }
    status.shutdown.store(true, Ordering::Relaxed);

    //stop ROS read connection Thread
    commands_tx.send(color_threads::ros::Command::Exit).ok();
    join_ros.join().unwrap();

    //stop Processing frustum Thread
    join_processing_frustum.join().unwrap();

    //stop LidarServ query Thread
    join_lidarserv_query.join().unwrap();

    //stop LidarServ answer Thread
    join_lidarserv_answer.join().unwrap();

    //stop Processing colorization Thread
    join_colorization.join().unwrap();

    //stop LidarServ store Thread
    join_lidarserv_store.join().unwrap();

    //stop status Thread
    join_status.join().unwrap();

    // Be polite.
    info!("Bye. 👋");

    // We are done.
    Ok(())
}

trait LogErrors {
    type Ok;
    fn log_error(self) -> Option<Self::Ok>;
}

impl<T, E> LogErrors for Result<T, E>
where
    E: Display + Debug,
{
    type Ok = T;

    fn log_error(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{e}");
                debug!("{e:?}");
                None
            }
        }
    }
}
