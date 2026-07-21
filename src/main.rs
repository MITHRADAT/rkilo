use std::{io::{self, Read}, cmp, process};
// use libc;

mod logger;
mod config;
mod common;

use config::Config;
use common::*;

static mut CURSOR: Cursor = Cursor {
    x: 0,
    y: 0
};

struct Cursor {
    x: u16,
    y: u16
}
    
fn main() {
   init();
    
    loop {
        refresh_screen();
        process_keypress();
    }

    //end();
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
    unsafe {
        let cursor_position = format!("\x1b[{};{}H", CURSOR.y + 1, CURSOR.x + 1);
        print!("{}", cursor_position);
    }
    print!("\x1b[?25h"); //show the cursor
    flush();
}

fn process_keypress() {
    let input = read_key();
    match input {
        Key::Quit => {
            clean_screen();
            flush();
            end();
            process::exit(0)
        },
        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => move_cursor(input),
        _ => {}
    }
}

fn read_key() -> Key {
    let mut c = [0u8; 1];
    let mut stdin = io::stdin();
    loop {
        match stdin.read(&mut c) {
            Ok(1) => {
                if c[0] == b'\x1b' {
                    break
                }
                if c[0] == ctrl_key(b'q') {
                    return Key::Quit
                }
                return Key::Char(c[0])
            },
            Ok(_) => continue,
            Err(err) => {
                end();
                die(DieReason::Panic(err.to_string()))
            }
        }
    }
    let mut seq = [0u8; 3];
    match stdin.read(&mut seq[0..1]) {
        Ok(1) => {
            if seq[0] != b'[' {
                return Key::ESC
            }
        },
        Ok(_) => return Key::ESC,
        Err(err) => {
            end();
            die(DieReason::Panic(err.to_string()))
        }
    }
    
    match stdin.read(&mut seq[1..2]) {
        Ok(1) => {},
        Ok(_) => return Key::ESC,
        Err(err) => {
            end();
            die(DieReason::Panic(err.to_string()))
        }
    }

    match seq[1] {
        b'A' => return Key::ArrowUp, 
        b'B' => return Key::ArrowDown, 
        b'C' => return Key::ArrowRight, 
        b'D' => return Key::ArrowLeft,
        _    => {}
    }
    
    return Key::ESC
}

fn move_cursor(key: Key) {
    let config = Config::get();
    unsafe {
        match key {
            Key::ArrowUp    => { if CURSOR.y > config.screen_zero() { CURSOR.y -= 1 } },
            Key::ArrowDown  => { if CURSOR.y < config.screen_rows() { CURSOR.y += 1 } },
            Key::ArrowRight => { if CURSOR.x < config.screen_cols() { CURSOR.x += 1 } },
            Key::ArrowLeft  => { if CURSOR.x > config.screen_zero() { CURSOR.x -= 1 } },
            _               => {}
        }
    }
}

fn ctrl_key(c: u8) -> u8 {
    c & 0x1f
}

enum Key {
    Char(u8),
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
    Quit,
    ESC
}
