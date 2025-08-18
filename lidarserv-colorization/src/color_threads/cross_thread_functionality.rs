use log::{debug, warn};
use rosrust::ros_debug_throttle;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::Receiver;

pub fn check_stop_lidarserv_colorization(
    stop_lidarserv_colorization_rx: &mut Receiver<()>,
) -> bool {
    debug!("check_stop_lidarserv_colorization: called");
    match stop_lidarserv_colorization_rx.try_recv() {
        Ok(_) => {
            debug!("stop_lidarserv_colorization: stop signal received");
            false
        }
        Err(TryRecvError::Closed) => {
            warn!("stop_lidarserv_colorization: stop signal channel closed");
            false
        }
        Err(TryRecvError::Empty) => {
            //meaning: stop signal hasn't been received yet.
            debug!("stop_lidarserv_colorization: stop signal channel empty, continue working");
            true
        }
        Err(TryRecvError::Lagged(_)) => {
            warn!("stop_lidarserv_colorization: stop signal channel lagged");
            false
        }
    }
}