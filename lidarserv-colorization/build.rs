use std::{env, path::PathBuf};

fn main() {
    let pkg_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest root"));
    let msg_path1 = pkg_path.join("ros1_common_msgs");
    println!(
        "cargo:rustc-env=ROSRUST_MSG_PATH={}",
        msg_path1.to_str().unwrap(),
    )
}
