use console::{style, Key};
use log::info;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use tokio_util::sync::CancellationToken;
/// Holds status information printed regularly.
#[derive(Debug, Default)]
pub struct Status {
    pub paused: AtomicBool,
    pub shutdown: AtomicBool,
    pub nr_received_images: AtomicU64, //Number of received image messages.
    pub nr_process_frustum_in: AtomicU64, // Number of received frustum queries.
    pub nr_process_frustum_out: AtomicU64, //Number of processed frustum queries.

    pub t0_ros_thread_running: AtomicBool,
    pub t0_ros_current_image_id: AtomicU64,

    pub t1_process_frustum_thread_running: AtomicBool,
    pub t1_process_frustum_thread_current_image_id: AtomicU64,

    pub t2_send_frustum_thread_running: AtomicBool,
    pub t2_send_frustum_thread_current_image_id: AtomicU64,
    pub t2_send_frustum_thread_nr_received_points: AtomicU64, //Number of received points

    pub t3_collect_colorization_data_thread_running: AtomicBool,
    pub t3_collect_colorization_data_thread_current_image_id: AtomicU64,
    pub t3_collect_colorization_data_thread_nr_received_points: AtomicU64,
    pub t3_collect_colorization_data_thread_nr_received_nodes: AtomicU64,

    pub t4_managing_colorization_thread_running: AtomicBool,
    pub t4_managing_colorization_thread_current_image_id: AtomicU64,
    pub t2_send_frustum_thread_nr_received_nodes: AtomicU64, //Number of received nodes
}

pub fn status_thread(status: Arc<Status>, stop_token: CancellationToken){
    let stop_control_thread = stop_token.child_token();
    {
        let status = Arc::clone(&status);
        thread::spawn(move || control_thread(status,stop_control_thread));
    }

    while !stop_token.is_cancelled() {
        thread::sleep(Duration::from_millis(500));
        //todo!(make output clearer for usecase)
        let paused = status.paused.load(Ordering::Relaxed);
        let shutdown = status.shutdown.load(Ordering::Relaxed);

        let ros_thread_running = status.t0_ros_thread_running.load(Ordering::Relaxed);
        let process_frustum_thread_running = status
            .t1_process_frustum_thread_running
            .load(Ordering::Relaxed);
        let send_frustum_thread_running =
            status.t2_send_frustum_thread_running.load(Ordering::Relaxed);
        let collect_colorization_data_thread_running = status
            .t3_collect_colorization_data_thread_running
            .load(Ordering::Relaxed);
        let managing_colorization_thread_running = status
            .t4_managing_colorization_thread_running
            .load(Ordering::Relaxed);

        let t0_ros_current_image_id = Option::from(status.t0_ros_current_image_id.load(Ordering::Relaxed));

        let t1_process_frustum_thread_current_image_id = Option::from(status.t1_process_frustum_thread_current_image_id.load(Ordering::Relaxed));

        let t2_send_frustum_thread_current_image_id = Option::from(status.t2_send_frustum_thread_current_image_id.load(Ordering::Relaxed));
        let t2_send_frustum_thread_nr_received_points = Option::from(status.t2_send_frustum_thread_nr_received_points.load(Ordering::Relaxed));
        let t2_send_frustum_thread_nr_received_nodes = Option::from(status.t2_send_frustum_thread_nr_received_nodes.load(Ordering::Relaxed));

        let t3_collect_colorization_data_thread_current_image_id = Option::from(status.t3_collect_colorization_data_thread_current_image_id.load(Ordering::Relaxed));
        let t3_collect_colorization_data_thread_nr_received_points = Option::from(status.t3_collect_colorization_data_thread_nr_received_points.load(Ordering::Relaxed));
        let t3_collect_colorization_data_thread_nr_received_nodes = Option::from(status.t3_collect_colorization_data_thread_nr_received_nodes.load(Ordering::Relaxed));


        let t4_managing_colorization_thread_current_image_id = Option::from(status.t4_managing_colorization_thread_current_image_id.load(Ordering::Relaxed));

        let state_part = if shutdown {
            "[⏹]"
        } else if paused {
            "[⏸︎]"
        } else {
            "[⏵]"
        };

        let mut thread_states = String::new();
        check_or_cross(& mut thread_states,
                       ros_thread_running,
                       "t0_ros",
                       t0_ros_current_image_id,
                       None,
                       None)
            .expect("couldnt write status of ros thread");
        check_or_cross(& mut thread_states,
                       process_frustum_thread_running,
                       "t1_frustum",
                       t1_process_frustum_thread_current_image_id,
                       None,
                       None)
            .expect("couldnt write status of process_frustum_thread");
        check_or_cross(& mut thread_states,
                       send_frustum_thread_running,
                       "t2_send_f",
                       t2_send_frustum_thread_current_image_id,
                       t2_send_frustum_thread_nr_received_points,
                       t2_send_frustum_thread_nr_received_nodes)
            .expect("couldnt write status of send_frustum_thread");
        check_or_cross(& mut thread_states,
                       collect_colorization_data_thread_running,
                       "t3_collect_p",
                       t3_collect_colorization_data_thread_current_image_id,
                       t3_collect_colorization_data_thread_nr_received_points,
                       t3_collect_colorization_data_thread_nr_received_nodes)
            .expect("couldnt write status of collect_colorization_data_thread");
        check_or_cross(& mut thread_states,
                       managing_colorization_thread_running,
                       "t4_colorize",
                       t4_managing_colorization_thread_current_image_id,
                       None,
                       None)
            .expect("couldnt write status of managing_colorization_thread");


        println!(
            "{}[{}{}]\n",
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
}

pub fn control_thread(status: Arc<Status>,stop_token:CancellationToken) {
    let term = console::Term::stdout();
    if !term.features().is_attended() {
        return;
    }
    info!("Press space to pause / unpause.");

    loop {
        if stop_token.is_cancelled() {break;}
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

fn check_or_cross(thread_states: &mut String,
                  thread_running: bool, thread_name:&str,
                  thread_image_id:Option<u64>,
                  thread_point_count:Option<u64>,
                  thread_received_nodes:Option<u64>,
) -> anyhow::Result<()> {
    const CHECK: &str = "✓";
    const CROSS: &str = "✗";
    const MAX_THREAD_NAME_LENGTH: usize = 15;

    thread_states.push_str("\n");
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
    thread_states.push_str(&style(thread_name).bold().to_string());
    thread_states.push_str(":");
    if thread_name.len() < MAX_THREAD_NAME_LENGTH{
        thread_states.push_str(&" ".repeat(MAX_THREAD_NAME_LENGTH + 1 - thread_name.len()));
    }
    if thread_image_id.is_some() {
        thread_states.push_str("| image ID: ");
        thread_states.push_str(&thread_image_id.unwrap().to_string());
    }
    if thread_point_count.is_some() {
        thread_states.push_str("| Nr. of points: ");
        thread_states.push_str(&thread_point_count.unwrap().to_string());
    }
    if thread_received_nodes.is_some() {
        thread_states.push_str("| Nr. of received nodes: ");
        thread_states.push_str(&thread_received_nodes.unwrap().to_string());
    }
    Ok(())
}
