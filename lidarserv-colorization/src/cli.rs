use clap::Parser;

/// Connector that forwards point clouds from ROS into lidarserv.
///
/// The connection to ROS should be established automatically if you are within the ROS environment. If the connection fails, you can set the following ROS connection options:
///
///  - The master URL:
///     via command line: `__master:=http://localhost:11311/`
///     via environment variable: `ROS_MASTER_URI=http://localhost:11311/`
///  - The ros hostname or ip address:
///     via command line: `__hostname:=localhost` or `__ip:=127.0.0.1`
///     via environment variables: `ROS_HOSTNAME=localhost` or `ROS_IP=127.0.0.1`
///  - The ros namespace:
///     via command line: `__ns:=my_namespace`
///     via environment variable: `ROS_NAMESPACE=my_namespace`
///  - The node name:
///     via command line: `__name:=lidarserv`
#[derive(Debug, Parser, Clone)]
#[command(verbatim_doc_comment)]
pub struct AppOptions {
    /// Verbosity of the command line output.
    #[clap(long, default_value = "info")]
    pub log_level: log::Level,

    /// Vector of the ROS topics where the image messages will be published to.
    #[clap(long, default_values = &["/Cam1_Image","/Cam2_Image","/Cam3_Image"])]
    pub image_topics: Vec<String>,

    /// Name of the fixed coordinate frame that the lidar points will be
    /// transformed to before sending to the lidarserv server.
    // note: The default should probably be "map" according to REP-105 https://www.ros.org/reps/rep-0105.html
    #[clap(long, default_value = "camera_init")]
    pub world_frame: String,

    /// Hostname of the lidarserv server
    #[clap(long, default_value = "::1")]
    pub host: String,

    /// Port of the lidarserv server
    #[clap(long, default_value = "4567")]
    pub port: u16,
}
