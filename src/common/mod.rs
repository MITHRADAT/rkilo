use std::{io::{self, Write}, process};
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
        },
        DieReason::Quit => {
            process::exit(0)
        }
    }
}

pub fn ctrl_key(c: u8) -> u8 {
    c & 0x1f
}

pub enum DieReason {
    Panic(String),
    FFI(String),
    Quit
}

pub enum Key {
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
    BackSpace  ,
    Quit       ,
    ESC        ,
    Save       ,
    Enter      ,
}
