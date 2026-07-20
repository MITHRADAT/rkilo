use std::{io::{self, Read}, cmp};
// use libc;

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
    let config = Config::get();
    let rows = config.screen_rows();
    let cols = config.screen_cols() as usize;
    let message_row = rows / 3;

    //before welcome message
    for _ in 0..message_row {
        print!("~");
        print!("\x1b[K"); //clear line
        print!("\r\n");
    }

    //welcome message
    print!("~");
    let mut welcome = "kilo editor written in rust -- version 0.0.1";
    let welcome_len = cmp::min(welcome.len(), cols);
    welcome = &welcome[..welcome_len];
    let padding = (cols - 1 - welcome_len) / 2;
    for _ in 0..padding {
        print!(" ");
    }
    print!("{}", welcome);
    print!("\x1b[K"); //clear line
    print!("\r\n");

    //after welcome message
    for _ in message_row + 1..rows - 1 {
        print!("~");
        print!("\x1b[K"); //clear line
        print!("\r\n");
    }
    
    //last line
    print!("~");
    print!("\x1b[K"); //clear line
}

fn refresh_screen() {
    print!("\x1b[?25l"); //hide the cursor
    print!("\x1b[H"); //reposition the cursor
    draw_rows();
    print!("\x1b[H"); //reposition the cursor
    print!("\x1b[?25h"); //show the cursor
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
