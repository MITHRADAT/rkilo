use std::{mem, io::{self, Read, Write}, process};
use libc;
use term_size;


fn main() {
   let _raw_mode = RawMode::new();
    
    loop {
        refresh_screen();
        process_key_press();
    }
}


fn draw_rows() {
    let config = Config::new();
    for _ in 0..config.screen_rows {
        print!("~\r\n");
    }
}

fn refresh_screen() {
    clean_screen(false);
    draw_rows();
    print!("\x1b[H"); //reposition the cursor
    io::stdout().flush().unwrap()
}

fn clean_screen(do_flush: bool) {
    print!("\x1b[2J"); //clear the screen
    print!("\x1b[H"); //reposition the cursor
    if do_flush {
        io::stdout().flush().unwrap()
    }
}

fn process_key_press() {
    loop {
        let input = read_key();
        if input == ctrl_key(b'q') {
            clean_screen(true);
            process::exit(0)
        }
    }
}

fn read_key() -> u8 {
    let mut c = [0u8; 1];
    let mut stdin = io::stdin();
    while stdin.read(&mut c).unwrap() == 1 {
        
    }
    c[0]
}

struct Config {
    screen_rows: u16,
    screen_cols: u16
}

impl Config {
    fn new() -> Self {
        let (rows, cols) = get_window_size();
        Self {
            screen_rows: rows,
            screen_cols: cols
        }
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
                die("tcgetattr in RawMode::new()", true)
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
                == 1 {
                    die("tcsetattr in RawMode::enable()", true)
            }
        }
    }

    fn disable(&self) {
        unsafe {
            if libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSAFLUSH,
                &self.original)
                == 1 {
                    die("tcsetattr in RawMode::disable()", true)
                }
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        self.disable()
    }
}

fn get_window_size() -> (u16, u16) {
    let mut window = mem::MaybeUninit::<libc::winsize>::uninit();
    let mut result = (0u16, 0u16);
    unsafe {
        let mut success = false;
        if libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            &mut window) >= 0 {
            let win = window.assume_init();
            if win.ws_row > 0 && win.ws_col > 0 {
                success = true;
                result = (win.ws_row, win.ws_col);
            }
        }

        if !success {
            print!("\x1b[999C\x1b[999B");
            result = get_cursor_position();
        }
    }
    result
}

fn get_cursor_position() -> (u16, u16) {
    // print!("\x1b[6n");
    // io::stdout().flush().unwrap();
    // println!();
    die("cant get the size of window", true)
}

fn die(msg: &str, by_ffi: bool) -> ! {
    clean_screen(true);
    if by_ffi {
        panic!("by foreign function interface: {}", msg)
    } else {
        panic!("{}", msg)
    }
}

fn ctrl_key(c: u8) -> u8 {
    c & 0x1f
}
