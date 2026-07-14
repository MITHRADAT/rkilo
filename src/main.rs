use std::{mem, io::{self, Read}};
use libc;

fn main() {
   let _raw_mode = RawMode::new();
    
    let mut c = [0u8; 1];
    let mut stdin = io::stdin();
    while stdin.read(&mut c).unwrap() == 1 && c[0] != b'q' {
        if c[0].is_ascii_control() {
            println!("{}\r\n", c[0])
        } else {
            println!("{} -> {}\r\n", c[0], c[0] as char)
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
                die_by_ffi("tcgetattr in RawMode::new()")
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
                    die_by_ffi("tcsetattr in RawMode::enable()")
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
                    die_by_ffi("tcsetattr in RawMode::disable()")
                }
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        self.disable()
    }
}


fn die_by_ffi(msg: &str) -> ! {
    panic!("{}", msg)
}
