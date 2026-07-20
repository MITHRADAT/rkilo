use std::{io::Write, path::Path, fs};
use chrono::Local;


pub struct Logger;

impl Logger {
    pub fn log(text: &str) {
        let file_name = Local::now().format("%Y.%m.%d.%H-%M").to_string();
        let log_dir_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("log");
        fs::create_dir_all(&log_dir_path).unwrap();
        let path = log_dir_path.join(file_name);
        let mut file =  fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)
            .unwrap();

        writeln!(file, "{}", text).unwrap();
    }
}
