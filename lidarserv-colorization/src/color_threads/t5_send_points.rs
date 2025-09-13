use std::sync::{mpsc, Arc};
use std::time::Duration;
use log::debug;
use pasture_core::containers::{BorrowedBuffer, VectorBuffer};
use tokio_util::sync::CancellationToken;
use crate::cli::AppOptions;
use crate::color_threads::status::Status;

pub fn thread_5_send_points(
    _args: AppOptions,
    stop_token: CancellationToken,
    colorized_points_rx: mpsc::Receiver<VectorBuffer>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    loop{
        if stop_token.is_cancelled() {
            debug!("thread_4_managing_colorization: Stop signal received");
            break;
        }

        let _colorized_points_vecbuf = match colorized_points_rx.recv_timeout(Duration::new(1, 0)){
            Ok(data) => {
                status.t5_send_points_thread_nr_received_points.fetch_add(data.len() as u64, std::sync::atomic::Ordering::Relaxed);
                status.t5_send_points_thread_nr_received_nodes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                data
            }
            Err(_) => {
                continue;
            }
        };
        //todo construct client

        //todo send points to server

    }



    Ok(())
}