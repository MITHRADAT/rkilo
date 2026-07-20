use std::{mem, io::{self, Read, Write}, sync::OnceLock, str};
use libc;

mod logger;

use logger::Logger;

fn main() {
   let _raw_mode = RawMode::new();
    
    loop {
        refresh_screen();
        match process_keypress() {
            ProcessKeyResult::Continue => (),
            ProcessKeyResult::Quit => break
        }
    }
}


fn draw_rows() {
    let config = Config::get();
    for _ in 0..config.screen_rows {
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
            Err(err) => die(DieReason::Panic(err.to_string()))
        }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

struct Config {
    screen_rows: u16,
    screen_cols: u16
}

impl Config {
    pub fn get() -> &'static Config {
        let config = CONFIG.get_or_init(|| {
            let (rows, cols) = window_size();
            Config {
                screen_rows: rows,
                screen_cols: cols,
            }
        });
        config
    }
}

pub struct RawMode {
    original: libc::termios,
}

impl RawMode {
    pub fn new() -> Self {
        let mut termios = mem::MaybeUninit::<libc::termios>::uninit();
        unsafe {
            if libc::tcgetattr(libc::STDIN_FILENO, termios.as_mut_ptr()) == -1 {
                die(DieReason::FFI("tcgetattr in RawMode::new()".to_string()))
            }
        }
        let raw = Self {
            original: unsafe {
                termios.assume_init()
            }
        };
        raw.enable();
        raw
    }
    
    fn enable(&self) {
        let mut raw = self.original;
        raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
        raw.c_oflag &= !libc::OPOST;
        raw.c_cflag |= libc::CS8;
        raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
        raw.c_cc[libc::VMIN] = 0;
        raw.c_cc[libc::VTIME] = 1;
        unsafe {
            if libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSAFLUSH,
                &raw)
                == -1 {
                    die(DieReason::FFI("tcsetattr in RawMode::enable()".to_string()))
            }
        }
    }

    fn disable(&self) {
        unsafe {
            if libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSAFLUSH,
                &self.original)
                == -1 {
                    die(DieReason::FFI("tcsetattr in RawMode::disable()".to_string()))
                }
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        self.disable()
    }
}

fn window_size() -> (u16, u16) {
    // let mut window = mem::MaybeUninit::<libc::winsize>::uninit();
    // unsafe {
    //     if libc::ioctl(
    //         libc::STDOUT_FILENO,
    //         libc::TIOCGWINSZ,
    //         window.as_mut_ptr()) >= 0 {
    //         let win = window.assume_init();
    //         if win.ws_row > 0 && win.ws_col > 0 {
    //             return (win.ws_row, win.ws_col)
    //         }
    //     }
    // }
    cursor_position()
}

fn cursor_position() -> (u16, u16) {
    print!("\x1b[999C\x1b[999B");
    print!("\x1b[6n");
    flush();

    let mut buff = [0u8; 32];
    let mut stdin = io::stdin();
    let mut i = 0;
    loop {
        match stdin.read(&mut buff[i..i + 1]) {
            Ok(1) => {
                if buff[i] == b'R' {
                    break;
                }
                if buff[i].is_ascii_control() {
                    Logger::log(format!("control({}): {} -> {:?}", i, buff[i], buff[i] as char).as_str());
                } else {
                    Logger::log(format!("usual({}): {}", i, buff[i] as char).as_str())
                }
                i += 1;
            },
            Ok(_) => {},
            Err(err) => die(DieReason::Panic(err.to_string())) 
        }
    }

    let response = str::from_utf8(&buff[2..i]).unwrap_or_else(|err| {
        die(DieReason::Panic(err.to_string()))
    });
    Logger::log(format!("raw form of cursor position: {}", response).as_str());
    let (rows, cols) = response.split_once(';').unwrap_or_else(|| {
        die(DieReason::Panic("cant split the row and col from cursor position".to_string()))
    });
    let rows: u16 = rows.parse().unwrap_or_else(|_| {
        die(DieReason::Panic("cant parse row from cursor position".to_string()))
    });
    let cols: u16 = cols.parse().unwrap_or_else(|_| {
        die(DieReason::Panic("cant parse col from cursor position".to_string()))
    });
    Logger::log(format!("positionn from cusrsor: ({}, {})", rows, cols).as_str());
    (rows, cols)
}

fn die(reason: DieReason) -> ! {
    Logger::log("die started");
    clean_screen();
    flush();
    match reason {
        DieReason::Panic(msg) => panic!("{}", msg),
        DieReason::FFI(msg) => panic!("by foreign function interface: {}", msg)
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
