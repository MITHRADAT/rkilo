use std::io::{self, Write};
use super::logger::Logger;

pub fn flush() {
    io::stdout().flush().unwrap();
}

pub fn clean_screen() {
    print!("\x1b[2J"); //clear the screen
    print!("\x1b[H"); //reposition the cursor
}

pub fn die(reason: DieReason) -> ! {
    clean_screen();
    flush();
    match reason {
        DieReason::Panic(msg) => {
            Logger::log(format!("die by panic:\r\n{}", msg).as_str());
            panic!("{}", msg)
        },
        DieReason::FFI(msg) => {
            Logger::log(format!("die by ffi:\r\n{}", msg).as_str());
            panic!("by foreign function interface: {}", msg)
        }
    }
}

pub enum DieReason {
    Panic(String),
    FFI(String)
}
