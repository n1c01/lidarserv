use std::sync::{mpsc, Arc};
use std::time::Duration;
use log::debug;
use pasture_core::containers::VectorBuffer;
use tokio_util::sync::CancellationToken;
use crate::cli::AppOptions;
use crate::color_threads::status::Status;

pub fn thread_5_send_points(
    args: AppOptions, 
    stop_token: CancellationToken,
    colorized_points_rx: mpsc::Receiver<VectorBuffer>,
    status: Arc<Status>,
) -> anyhow::Result<()> {
    loop{
        if stop_token.is_cancelled() {
            debug!("thread_4_managing_colorization: Stop signal received");
            break;
        }
        
        let colorized_points_vecbuf = match colorized_points_rx.recv_timeout(Duration::new(1, 0)){
            Ok(data) => {
                data
            }
            Err(error) => {
                continue;
            }
        };
        //construct client 
        
        //send points to server
    
    }
    
    
    
    Ok(())
}