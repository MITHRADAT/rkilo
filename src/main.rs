use std::{mem, io::{self, Read}};
use libc;

fn main() {
   let _raw_mode = RawMode::new();
    
    let mut c = [0u8; 1];
    let mut stdin = io::stdin();
    while stdin.read(&mut c).unwrap() == 1 && c[0] != b'q' {
    }
}



pub struct RawMode {
    original: libc::termios,
}

impl RawMode {
    pub fn new() -> Self {
        let mut termios = mem::MaybeUninit::<libc::termios>::uninit();
        unsafe {
            libc::tcgetattr(libc::STDIN_FILENO, termios.as_mut_ptr());
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
        raw.c_lflag &= !libc::ECHO;
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw);
        }
    }

    fn disable(&self) {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &self.original);
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        self.disable()
    }
}
