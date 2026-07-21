use std::{io::{self, Read}, cmp, process};
// use libc;

mod logger;
mod config;
mod common;
mod cursor;

use common::*;
use cursor::*;
use config::Config;
    
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
        Key::PageUp => {
            for _ in 0..distance_from_top() {
                move_cursor(Key::ArrowUp)
            }
        },
        Key::PageDown => {
            for _ in 0..distance_from_bottom() {
                move_cursor(Key::ArrowDown)
            }
        },
        _ => {}
    }
}

fn read_key() -> Key {
    fn read_byte(buff: &mut [u8]) -> u8 {
        let mut stdin = io::stdin();
        loop {
            match stdin.read(buff) {
                Ok(1) => return buff[0],
                Ok(_) => continue,
                Err(err) => {
                    end();
                    die(DieReason::Panic(err.to_string()))
                }
            }
        }
    }

    let mut buff = [0u8; 1];
    let byte = read_byte(&mut buff);
    if byte == ctrl_key(b'q') {
        return Key::Quit
    }
    if byte == b'\x1b' {
        let mut seq = [0u8; 3];
        if read_byte(&mut seq[0..1]) == b'[' {
            match read_byte(&mut seq[1..2]) {
                b'A' => return Key::ArrowUp, 
                b'B' => return Key::ArrowDown, 
                b'C' => return Key::ArrowRight, 
                b'D' => return Key::ArrowLeft,
                b'5' => {
                    if read_byte(&mut seq[1..2]) == b'~' { return Key::PageUp }
                    return Key::ESC
                },
                b'6' => {
                    if read_byte(&mut seq[1..2]) == b'~' { return Key::PageDown }
                    return Key::ESC
                },
                _    => return Key::ESC,
            }
        } else {
            return Key::ESC
        }
    } else {
        return Key::Char(byte)
    }
}

fn move_cursor(key: Key) {
    let config = Config::get();
    unsafe {
        match key {
            Key::ArrowUp    => { if CURSOR.y > config.screen_zero() { CURSOR.y -= 1 } },
            Key::ArrowDown  => { if CURSOR.y < config.screen_rows() { CURSOR.y += 1 } },
            Key::ArrowLeft  => { if CURSOR.x > config.screen_zero() { CURSOR.x -= 1 } },
            Key::ArrowRight => { if CURSOR.x < config.screen_cols() { CURSOR.x += 1 } },
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
    PageUp,
    PageDown,
    Quit,
    ESC
}
