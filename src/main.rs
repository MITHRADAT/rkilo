use std::{io::{self, Read}};
use libc;

mod logger;
mod config;
mod common;

use config::Config;
use common::*;

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

enum ProcessKeyResult {
    Continue,
    Quit
}

fn ctrl_key(c: u8) -> u8 {
    c & 0x1f
}
