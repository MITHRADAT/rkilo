use std::{io::{self, Read}, env, fs, cmp, process};

mod logger;
mod config;
mod common;
mod cursor;

use common::*;
use cursor::*;
use config::Config;

static mut TEXT: Vec<String> = Vec::new();

fn main() {
    init();
    
    if let Some(path) = env::args().nth(1) {
        read_file(&path)
    }
    
    loop {
        refresh_screen();
        process_keypress();
    }
}

fn init() {
    Config::get().enable_raw_mode();
}

fn end() {
    Config::get().disable_raw_mode();
}

#[allow(static_mut_refs)]
fn read_file(path: &str) {
    fs::read_to_string(path).unwrap_or_else(|err| {
        end();
        die(DieReason::Panic(err.to_string()))
    })
        .lines()
        .for_each(|line| unsafe { TEXT.push(line.to_string()) })
}

#[allow(static_mut_refs)]
fn draw_rows() {
    let config = Config::get();
    let rows = config.screen_rows() as usize;
    let cols = config.screen_cols() as usize;
    let y_offset = unsafe { CURSOR.y_offset as usize };
    let file_line_count = unsafe { TEXT.len() };
    
    for screen_row in 0..rows {
        let file_row = y_offset + screen_row;
        if file_row < file_line_count {
            unsafe { print!("{}", &TEXT[file_row]) }
        } else if file_line_count < rows {
            print!("~");
            
            //welcome message
            if file_line_count == 0 && screen_row == (rows / 3) {
                let mut welcome = "kilo editor written in rust -- version 0.0.1";
                let welcome_len = cmp::min(welcome.len(), cols);
                welcome = &welcome[..welcome_len];
                let padding = (cols - 1 - welcome_len) / 2;
                for _ in 0..padding { print!(" ") }
                print!("{}", welcome);
            }
        }
        
        print!("\x1b[K"); //clear line
        if screen_row < rows - 1 {
            print!("\r\n");
        }
    }
}

fn scroll() {
    unsafe {
        if CURSOR.y < CURSOR.y_offset {
            CURSOR.y_offset = CURSOR.y
        }
        
        let screen_rows = Config::get().screen_rows();
        if CURSOR.y >= CURSOR.y_offset + screen_rows {
            CURSOR.y_offset = CURSOR.y - screen_rows + 1;
        }
    }
}

fn refresh_screen() {
    scroll();
    print!("\x1b[?25l"); //hide the cursor
    print!("\x1b[H"); //reposition the cursor
    draw_rows();
    unsafe {
        let cursor_position = format!("\x1b[{};{}H", CURSOR.y - CURSOR.y_offset + 1, CURSOR.x + 1);
        print!("{}", cursor_position);
    }
    print!("\x1b[?25h"); //show the cursor
    flush();
}

fn process_keypress() {
    let input = read_key();
    match input {
        Key::ArrowUp   |
        Key::ArrowDown |
        Key::ArrowLeft |
        Key::ArrowRight => { move_cursor(input) },
        Key::Quit       => { clean_screen(); flush(); end(); process::exit(0) },
        Key::PageUp     => { for _ in 0..distance_from_top()    { move_cursor(Key::ArrowUp)   } },
        Key::PageDown   => { for _ in 0..distance_from_bottom() { move_cursor(Key::ArrowDown) } },
        Key::Home       => { unsafe { CURSOR.x = Config::get().screen_zero()     } },
        Key::End        => { unsafe { CURSOR.x = Config::get().screen_cols() - 1 } },
        _               => {}
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
                },
            }
        }
    }

    let mut buff = [0u8; 1];
    let byte     = read_byte(&mut buff);
    
    if byte == ctrl_key(b'q') { return Key::Quit }
    if byte == b'\x1b' {
        let mut seq = [0u8; 3];
        match read_byte(&mut seq[0..1]) {
            b'[' =>  {
                match read_byte(&mut seq[1..2]) {
                    b'1' => { if read_byte(&mut seq[2..3]) == b'~' { return Key::Home     } return Key::ESC },
                    b'3' => { if read_byte(&mut seq[2..3]) == b'~' { return Key::Delete   } return Key::ESC },
                    b'4' => { if read_byte(&mut seq[2..3]) == b'~' { return Key::End      } return Key::ESC },
                    b'5' => { if read_byte(&mut seq[2..3]) == b'~' { return Key::PageUp   } return Key::ESC },
                    b'6' => { if read_byte(&mut seq[2..3]) == b'~' { return Key::PageDown } return Key::ESC },
                    b'7' => { if read_byte(&mut seq[2..3]) == b'~' { return Key::Home     } return Key::ESC },
                    b'8' => { if read_byte(&mut seq[2..3]) == b'~' { return Key::End      } return Key::ESC },
                    b'A' => return Key::ArrowUp,
                    b'B' => return Key::ArrowDown,
                    b'C' => return Key::ArrowRight,
                    b'D' => return Key::ArrowLeft,
                    b'F' => return Key::End,
                    b'H' => return Key::Home,
                    _    => return Key::ESC,
                }
            },
            b'O' => {
                match read_byte(&mut seq[1..2]) {
                    b'H' => return Key::Home,
                    b'F' => return Key::End,
                    _    => return Key::ESC
                }
            },
            _ => return Key::ESC
        }
    }

    Key::Char(byte)
}

#[allow(static_mut_refs)]
fn move_cursor(key: Key) {
    let config = Config::get();
    unsafe {
        match key {
            Key::ArrowUp    => { if CURSOR.y > config.screen_zero()  { CURSOR.y -= 1 } },
            Key::ArrowDown  => { if CURSOR.y < TEXT.len() as u16 - 1 { CURSOR.y += 1 } },
            Key::ArrowLeft  => { if CURSOR.x > config.screen_zero()  { CURSOR.x -= 1 } },
            Key::ArrowRight => { if CURSOR.x < config.screen_cols()  { CURSOR.x += 1 } },
            _               => {}
        }
    }
}

fn ctrl_key(c: u8) -> u8 {
    c & 0x1f
}

enum Key {
    Char(u8)   ,
    ArrowUp    ,
    ArrowDown  ,
    ArrowRight ,
    ArrowLeft  ,
    Home       ,
    End        ,
    PageUp     ,
    PageDown   ,
    Delete     ,
    Quit       ,
    ESC        ,
}
