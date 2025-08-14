use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError},
        Arc,
    },
    thread,
    time::Duration,
};

use console::{style, Key};
use log::info;

/// Holds status information printed regularly.
#[derive(Debug, Default)]
pub struct Status {
    pub paused: AtomicBool,
    pub shutdown: AtomicBool,
    pub nr_received_images: AtomicU64, //Number of received image messages.
    pub nr_process_frustum_in: AtomicU64, // Number of received frustum queries.
    pub nr_process_frustum_out: AtomicU64, //Number of processed frustum queries.
    pub nr_sent_queries: AtomicU64,    //Number of sent queries.
    pub frustum_query_received_points: AtomicU64, //Number of received points
    pub frustum_query_received_nodes: AtomicU64, //Number of received nodes
}

pub fn status_thread(status: Arc<Status>, shutdown_rx: mpsc::Receiver<()>) {
    let mut buffer1: i64 = 0; // signed integers, because we use relaxed ordering for the atomic counters, so we could observe the increment of the counter that removes messages from the buffer before the one that inserts messages into the buffer.
    let mut buffer2: i64 = 0;
    let mut all_stopped_prev = false;

    {
        let status = Arc::clone(&status);
        thread::spawn(move || control_thread(status));
    }

    while let Err(RecvTimeoutError::Timeout) = shutdown_rx.recv_timeout(Duration::from_secs(1)) {
        //todo!(make output clearer for usecase)
        let nr_received_images = status.nr_received_images.swap(0, Ordering::Relaxed);
        let nr_process_frustum_in = status.nr_process_frustum_in.swap(0, Ordering::Relaxed);
        let nr_process_frustum_out = status.nr_process_frustum_out.swap(0, Ordering::Relaxed);
        let nr_tx_msg_query = status.nr_sent_queries.swap(0, Ordering::Relaxed);
        let paused = status.paused.load(Ordering::Relaxed);
        let shutdown = status.shutdown.load(Ordering::Relaxed);
        buffer1 += nr_received_images as i64;
        buffer1 -= nr_process_frustum_in as i64;
        buffer2 += nr_process_frustum_out as i64;
        buffer2 -= nr_tx_msg_query as i64;

        let state_part = if shutdown {
            "[⏹]"
        } else if paused {
            "[⏸︎]"
        } else {
            "[⏵]"
        };

        let mut all_stopped = paused || shutdown;
        let stop_reason = if shutdown {
            "shut down"
        } else if paused {
            "paused"
        } else {
            ""
        };
        let rx_part = if all_stopped && nr_received_images == 0 {
            stop_reason.to_string()
        } else {
            all_stopped = false;
            format!("{:3} msg/s", nr_received_images, )
        };
        let process_part = if all_stopped && buffer1 == 0 && nr_process_frustum_out == 0 {
            stop_reason.to_string()
        } else {
            all_stopped = false;
            format!(
                "queue: {:2} msg | {:3} msg/s",
                buffer1, nr_process_frustum_out,
            )
        };
        let tx_part = if all_stopped && buffer2 == 0 && nr_tx_msg_query == 0 {
            stop_reason.to_string()
        } else {
            all_stopped = false;
            format!("queue: {:2} msg | {:3} msg/s", buffer2, nr_tx_msg_query)
        };
        if !all_stopped || !all_stopped_prev {
            println!(
                "{}[{} {}] [{} {}] [{} {}]",
                state_part,
                style("RX").bold(),
                rx_part,
                style("PROCESS").bold(),
                process_part,
                style("TX").bold(),
                tx_part
            );
        }
        if all_stopped && shutdown {
            break;
        }
        all_stopped_prev = all_stopped;
    }
}

pub fn control_thread(status: Arc<Status>) {
    let term = console::Term::stdout();
    if !term.features().is_attended() {
        return;
    }
    info!("Press space to pause / unpause.");

    loop {
        match term.read_key() {
            Ok(Key::Char(' ')) => {
                let paused = !status.paused.fetch_not(Ordering::Relaxed);
                if paused {
                    term.write_line("[⏸︎] PAUSE").unwrap();
                    term.move_cursor_up(1).ok();
                } else {
                    term.write_line("[⏵] RESUME").unwrap();
                    term.move_cursor_up(1).ok();
                }
            }
            Ok(Key::Unknown) => return,
            Ok(_) => (),
            Err(_) => return,
        }
    }
}
