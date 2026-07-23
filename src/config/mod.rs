use std::{io::{self, Read}, sync::OnceLock, str, mem};
use super::{common::*, logger::Logger};


static CONFIG: OnceLock<Config> = OnceLock::new();
    
pub struct Config {
    screen_zero     : usize,
    screen_rows     : usize,
    screen_cols     : usize,
    original_termios: libc::termios,
}

impl Config {
    pub fn get() -> &'static Config {
        let config = CONFIG.get_or_init(|| {
            let (rows, cols) = window_size();
            let mut termios = mem::MaybeUninit::<libc::termios>::uninit();
            unsafe {
                if libc::tcgetattr(libc::STDIN_FILENO, termios.as_mut_ptr()) == -1 {
                    die(DieReason::FFI("tcgetattr Config::get()".to_string()))
                }
            }
            Config {
                screen_zero      : 0,
                screen_rows      : rows,
                screen_cols      : cols,
                original_termios : unsafe { termios.assume_init() },
            }
        });
        config
    }

    pub fn enable_raw_mode(&self) {
        let mut original_termios = self.original_termios;
        original_termios.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
        original_termios.c_oflag &= !libc::OPOST;
        original_termios.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
        original_termios.c_cflag |= libc::CS8;
        original_termios.c_cc[libc::VMIN]  = 0;
        original_termios.c_cc[libc::VTIME] = 1;
        unsafe {
            if libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSAFLUSH   ,
                &original_termios)
                == -1 {
                    die(DieReason::FFI(
                        "tcsetattr in Config::enable_raw_mode()".to_string()))
                }
        }
    }

    pub fn disable_raw_mode(&self) {
        unsafe {
            if libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSAFLUSH   ,
                &self.original_termios)
                == -1 {
                    die(DieReason::FFI(
                        "tcsetattr in Config::disable_raw_mode()".to_string()))
                }
        }
    }

    pub fn screen_zero(&self) -> usize { self.screen_zero }
    pub fn screen_rows(&self) -> usize { self.screen_rows }
    pub fn screen_cols(&self) -> usize { self.screen_cols }
}

fn window_size() -> (usize, usize) {
    let mut window = mem::MaybeUninit::<libc::winsize>::uninit();
    unsafe {
        if libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            window.as_mut_ptr()) >= 0 {
            let win = window.assume_init();
            if win.ws_row > 0 && win.ws_col > 0 {
                Logger::log(
                    format!("window size: ({}, {})", win.ws_row, win.ws_col).as_str());
                return (win.ws_row as usize, win.ws_col as usize)
            }
        }
    }
    cursor_position()
}

fn cursor_position() -> (usize, usize) {
    print!("\x1b[999C\x1b[999B"); //go 999 times forward and 999 times downward 
    print!("\x1b[6n"); //Device Status Report(DSR); get cursor position
    flush();

    let mut buff = [0u8; 32];
    let mut stdin = io::stdin();
    let mut i = 0;
    loop {
        match stdin.read(&mut buff[i..i + 1]) {
            Ok(1)    => { if buff[i] == b'R' { break } i += 1 },
            Ok(_)    => {},
            Err(err) => { die(DieReason::Panic(err.to_string())) },
        }
    }

    let response = str::from_utf8(&buff[2..i]).unwrap_or_else(|err| {
        die(DieReason::Panic(err.to_string()))
    });
    let (rows, cols) = response.split_once(';').unwrap_or_else(|| {
        die(DieReason::Panic(format!(
            "cant split the row and col from cursor position. response: {}", response)))
    });
    let rows: usize = rows.parse().unwrap_or_else(|_| {
        die(DieReason::Panic(format!(
            "cant parse row from cursor position. response: {}", response)))
    });
    let cols: usize = cols.parse().unwrap_or_else(|_| {
        die(DieReason::Panic(format!(
            "cant parse col from cursor position. response: {}", response)))
    });
    
    (rows, cols)
}
