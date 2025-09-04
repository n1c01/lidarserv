use console::{style, Key};
use log::info;
use std::fmt::format;
use std::ops::Add;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError},
        Arc,
    },
    thread,
    time::Duration,
};

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
    pub ros_thread_running: AtomicBool,
    pub process_frustum_thread_running: AtomicBool,
    pub send_frustum_thread_running: AtomicBool,
    pub collect_colorization_data_thread_running: AtomicBool,
    pub managing_colorization_thread_running: AtomicBool,
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

        let ros_thread_running = status.ros_thread_running.load(Ordering::Relaxed);
        let process_frustum_thread_running = status
            .process_frustum_thread_running
            .load(Ordering::Relaxed);
        let send_frustum_thread_running =
            status.send_frustum_thread_running.load(Ordering::Relaxed);
        let collect_colorization_data_thread_running = status
            .collect_colorization_data_thread_running
            .load(Ordering::Relaxed);
        let managing_colorization_thread_running = status
            .managing_colorization_thread_running
            .load(Ordering::Relaxed);



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
            format!("{:3} msg/s", nr_received_images,)
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

        let mut thread_states = String::new();
        check_or_cross(& mut thread_states, ros_thread_running,"t0_ros").expect("couldnt write status of ros thread");
        check_or_cross(& mut thread_states, process_frustum_thread_running,"t1_frustum").expect("couldnt write status of process_frustum_thread");
        check_or_cross(& mut thread_states, send_frustum_thread_running,"t2_send_f").expect("couldnt write status of send_frustum_thread");
        check_or_cross(& mut thread_states, collect_colorization_data_thread_running,"t3_collect_p").expect("couldnt write status of collect_colorization_data_thread");
        check_or_cross(& mut thread_states, managing_colorization_thread_running,"t4_colorize").expect("couldnt write status of managing_colorization_thread");


        if !all_stopped || !all_stopped_prev {
            println!(
                "{}[{}{}]",
                state_part,
                style("Thread states: ").bold(),
                thread_states,
                /*
                "{}[{} {}] [{} {}] [{} {}]",
                state_part,
                style("RX").bold(),
                rx_part,
                style("PROCESS").bold(),
                process_part,
                style("TX").bold(),
                tx_part

                 */
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

fn check_or_cross(thread_states: &mut String, thread_running: bool, thread_name:&str) -> anyhow::Result<()> {
    const CHECK: &str = "✓";
    const CROSS: &str = "✗";
    thread_states.push_str(thread_name);
    thread_states.push_str(":");
    thread_states.push_str(
        &style(if thread_running {
            CHECK
        } else {
            CROSS
        })
            .fg(if thread_running {
                console::Color::Green
            } else {
                console::Color::Red
            })
            .to_string());
    thread_states.push_str(", ");

    Ok(())
}
