use std::sync::{mpsc, Arc};
use std::time::Duration;
use log::debug;
use pasture_core::containers::{BorrowedBuffer, VectorBuffer};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use lidarserv_server::net::client::writing_clients::write::{Connect, WriteClient};
use lidarserv_server::net::client::writing_clients::update::Update;
use crate::cli::AppOptions;
use crate::color_threads::status::Status;


pub async fn thread_5_send_points(
    args: AppOptions,
    stop_token: CancellationToken,
    colorized_points_rx: mpsc::Receiver<VectorBuffer>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
    let mut update_client = WriteClient::connect((args.host.as_str(), args.port), &mut shutdown_rx).await?;
    loop{
        if stop_token.is_cancelled() {
            debug!("Stop signal received");
            break;
        }
        //debug!("Send Points Thread: Waiting for points");
        let colorized_points_vector = match colorized_points_rx.recv_timeout(Duration::new(1, 0)){
            Ok(data) => {
                status.t5_send_points_thread_nr_received_points.fetch_add(data.len() as u64, std::sync::atomic::Ordering::Relaxed);
                status.t5_send_points_thread_nr_received_nodes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                data
            }
            Err(_) => {
                continue;
            }
        };
        debug!("Send Points Thread: storing points on server");
        update_client.update_points_global_coordinates (&colorized_points_vector).await?
        // todo check if global coordinates is right
        
    }
    //shutdown client
    shutdown_tx.send(()).ok();
    debug!("Send Points Thread: finished");

    Ok(())
}