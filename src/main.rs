use std::{io::{self, Read, Write}};
use libc;

mod logger;
mod config;

use logger::Logger;
use config::Config;

fn main() {
   init();
    
    loop {
        refresh_screen();
        match process_keypress() {
            ProcessKeyResult::Continue => (),
            ProcessKeyResult::Quit => break
        }
    }

    end();
}

fn init() {
    Config::get().enable_raw_mode();
}

fn end() {
    Config::get().disable_raw_mode();
}

fn draw_rows() {
    for _ in 0..Config::get().screen_rows() {
        print!("~\r\n");
    }
}

fn refresh_screen() {
    clean_screen();
    draw_rows();
    print!("\x1b[H"); //reposition the cursor
    flush();
}

fn clean_screen() {
    print!("\x1b[2J"); //clear the screen
    print!("\x1b[H"); //reposition the cursor
}

fn flush() {
    io::stdout().flush().unwrap();
}

fn process_keypress() -> ProcessKeyResult {
    let input = read_key();
    if input == ctrl_key(b'q') {
        clean_screen();
        flush();
        return ProcessKeyResult::Quit
    }
    ProcessKeyResult::Continue
}

fn read_key() -> u8 {
    let mut c = [0u8; 1];
    let mut stdin = io::stdin();
    loop {
        match stdin.read(&mut c) {
            Ok(1) => return c[0],
            Ok(_) => continue,
            Err(err) => {
                end();
                die(DieReason::Panic(err.to_string()))
            }
        }
    }
}

fn die(reason: DieReason) -> ! {
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

enum DieReason {
    Panic(String),
    FFI(String)
}

enum ProcessKeyResult {
    Continue,
    Quit
}

fn ctrl_key(c: u8) -> u8 {
    c & 0x1f
}
