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
use tokio_util::sync::CancellationToken;
use crate::color_threads::colorization::colorization_thread;
use crate::init_colorize::init_colorize;

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

    let stop_source = CancellationToken::new();
    let child_token2 = stop_source.child_token();
    let child_token3 = stop_source.child_token();
    let child_token4 = stop_source.child_token();
    let child_token5 = stop_source.child_token();

    let (stop_status_tx, stop_status_rx) = channel();

    let (exit_tx, exit_rx) = channel();
    {
        let exit_tx = exit_tx.clone();
        ctrlc::set_handler(move || {
            debug!("Ctrl+C pressed.");
            stop_source.cancel();
            debug!("cooperative cancellation done");
            exit_tx.send(()).ok();
            debug!("exit message send");
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
                child_token2,
                image_data_rx,
                image_data_bypass_tx,
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
                child_token3,
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
            collect_colorization_data_thread(
                args4,
                child_token4,
                points_rx,
                image_data_bypass_rx,
                colorization_data_tx,
                status4)
                .log_error();
            exit_tx4.send(()).ok()
        })
    };

    //Processing colorization Thread
    let exit_tx5 = exit_tx.clone();
    let status5 = Arc::clone(&status);
    let args5 = args.clone();
    let (colorized_data_tx, colorized_data_rx) = mpsc::channel();

    let join_colorization = {
        thread::spawn(move || {
            //TODO: Processing colorization Thread, that colorizes the points
            colorization_thread(
                args5,
                child_token5,
                colorization_data_rx,
                colorized_data_tx,
                status5
            ).log_error();
            exit_tx5.send(()).ok()
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

    status.shutdown.store(true, Ordering::Relaxed);

    //todo: think about terminating threads forcfully after x amount of time. (e.g. 10 seconds)
    debug!("stopping ros");
    //stop ROS read connection Thread
    commands_tx.send(color_threads::ros::Command::Exit).ok();
    debug!("joining thread ros");
    join_ros.join().unwrap();

    //stop Processing frustum Thread
    debug!("joining thread processing frustum");
    join_processing_frustum.join().unwrap();

    //stop LidarServ query Thread
    debug!("joining thread lidarserv query");
    join_lidarserv_query.join().unwrap();

    //stop LidarServ answer Thread
    debug!("joining thread lidarserv answer");
    join_lidarserv_answer.join().unwrap();

    //stop Processing colorization Thread
    debug!("joining thread colorization");
    join_colorization.join().unwrap();

    //stop LidarServ store Thread
    debug!("joining thread lidarserv store");
    join_lidarserv_store.join().unwrap();

    //stop status Thread
    debug!("stopping status thread");
    stop_status_tx.send(()).ok();
    debug!("joining thread status");
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
