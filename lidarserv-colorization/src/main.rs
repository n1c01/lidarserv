use crate::color_threads::collect_colorization_data::collect_colorization_data_thread;
use crate::color_threads::processing_frustum::process_frustum_thread;
use crate::color_threads::ros::ros_thread;
use crate::color_threads::send_frustum::send_frustum_thread;
use crate::color_threads::status::{status_thread, Status};
use crate::color_threads::{processing_frustum, ImageIdAndVectorBuffer};
use anyhow::Result;
use anyhow::{Context, Error};
use clap::Parser;
use cli::AppOptions;
use log::{debug, error, info};
use rosrust::api::resolve::get_unused_args;
use std::fmt::{Debug, Display};
use std::process::ExitCode;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Receiver, Sender};
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
    let (stop_lidarserv_colorization_tx, stop_lidarserv_colorization_rx1) =
        tokio::sync::broadcast::channel(1); //only one message sent therefore capacity one
    let stop_lidarserv_colorization_rx2 = stop_lidarserv_colorization_tx.subscribe();
    let stop_lidarserv_colorization_rx3 = stop_lidarserv_colorization_tx.subscribe();
    let stop_lidarserv_colorization_rx4 = stop_lidarserv_colorization_tx.subscribe();

    let (stop_status_tx, stop_status_rx) = channel();

    let (exit_tx, exit_rx) = channel();
    {
        let exit_tx = exit_tx.clone();
        let mut first_ctrlc = true;
        ctrlc::set_handler(move || {
            if first_ctrlc {
                exit_tx.send(()).ok();
                first_ctrlc = false;
            } else {
                stop_lidarserv_colorization_tx.send(()).ok();
                stop_status_tx.send(()).ok();
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
    let exit_tx1 = exit_tx.clone();
    let status1 = Arc::clone(&status);
    let join_ros = {
        let args1 = args.clone();
        thread::spawn(move || {
            ros_thread(args1, commands_rx, image_data_tx, status1).log_error();
            exit_tx1.send(()).ok()
        })
    };

    //Processing frustum Thread
    //Processing Thread, that processes the camera position and calculates the frustum
    let (image_id_and_frustum_data_tx, image_id_and_frustum_data_rx) = mpsc::channel();
    let (image_data_bypass_tx, image_data_bypass_rx) = mpsc::channel(); //Channel for the image data of the camera.
    let exit_tx2 = exit_tx.clone();
    let status2 = Arc::clone(&status);
    let args2 = args.clone();

    let join_processing_frustum = {
        thread::spawn(move || {
            process_frustum_thread(
                args2,
                stop_lidarserv_colorization_rx2,
                image_data_rx,
                image_id_and_frustum_data_tx,
                status2,
            )
            .log_error();
            exit_tx2.send(()).ok()
        })
    };

    //LidarServ query Thread
    //TODO: LidarServ Thread, that queries the frustum to retrieve the points from the lidarserv server
    let (points_tx, points_rx) = mpsc::channel(); //todo!(rename more precise)
    let exit_tx3 = exit_tx.clone();
    let status3 = Arc::clone(&status);
    let args3 = args.clone();
    let join_lidarserv_query = {
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(send_frustum_thread(
                args3,
                stop_lidarserv_colorization_rx3,
                image_id_and_frustum_data_rx,
                points_tx,
                status3,
            ))
            .log_error();
            exit_tx3.send(()).ok()
        })
    };

    //LidarServ answer Thread
    //TODO: LidarServ Thread, that recieves the points from the lidarserv server and sends them to the colorization thread
    let (colorization_data_tx, colorization_data_rx) = mpsc::channel();
    let exit_tx4 = exit_tx.clone();
    let status4 = Arc::clone(&status);
    let args4 = args.clone();
    let join_lidarserv_answer = {
        thread::spawn(move || {
            collect_colorization_data_thread(args4, points_rx, image_data_bypass_rx, colorization_data_tx, status4)
                .log_error();
            exit_tx4.send(()).ok()
        })
    };

    //Processing colorization Thread
    //TODO: Processing Thread, that colorizes the points
    let join_colorization = {
        thread::spawn(move || {
            //todo!("colorization thread")
            //init_colorize();
            //exit_tx.send(()).ok()
        })
    };

    //LidarServ store Thread
    //TODO: LidarServ Thread, that sends the processed points to the lidarserv server
    let join_lidarserv_store = {
        thread::spawn(move || {
            //todo!("lidarserv store thread")
            //exit_tx.send(()).ok()
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
