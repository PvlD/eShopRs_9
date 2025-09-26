use std::env::{self, current_dir};

use stylers::build2;

fn main() {
    let mut cd = current_dir().unwrap().to_str().unwrap().to_string();
    let glob_root = cd.clone();

    cd.push_str("/..");

    if env::set_current_dir(&cd).is_ok() {
        build2(glob_root, Some(String::from("./style/main2.css")));
    } else {
        panic!("failed to change dir to {}", cd)
    }
}
