use std::{io::{self, Read}, fs, cmp, process};

mod cursor;
mod screen;

use cursor::Cursor;
use screen::Screen;
use super::common::*;

pub struct Editor {
    text  : Text,
    screen: Screen,
    cursor: Cursor,
}

struct Text {
    lines: Vec<String>,
}

impl Editor {
    pub fn init() -> Self {
        let screen = Screen::get();
        let editor = Self {
            text  : Text {
                lines: vec![],
            },
            cursor: Cursor::get(),
            screen: screen,
        };

        editor.screen.enable_raw_mode();
        editor
    }

    fn end(&self) {
        self.screen.disable_raw_mode();
    }

    pub fn read_file(&mut self, path: &str) {
        fs::read_to_string(path).unwrap_or_else(|err| {
            self.end();
            die(DieReason::Panic(err.to_string()))
        })
            .lines()
            .for_each(|line|  {
                self.text.lines.push(line.to_string());
            })
    }

    pub fn refresh_screen(&mut self) {
        self.scroll();
        print!("\x1b[?25l"); //hide the cursor
        print!("\x1b[H"); //reposition the cursor
        self.draw_rows();
        let cursor_position = format!("\x1b[{};{}H",
                                      self.cursor.y - self.cursor.y_offset + 1,
                                      self.cursor.x - self.cursor.x_offset + 1);
        print!("{}", cursor_position);
        print!("\x1b[?25h"); //show the cursor
        flush();
    }

    pub fn process_keypress(&mut self) {
        let input = self.read_key();
        match input {
            Key::ArrowUp   |
            Key::ArrowDown |
            Key::ArrowLeft |
            Key::ArrowRight => { self.move_cursor(input) },
            Key::Quit       => { clean_screen(); flush(); self.end(); process::exit(0) },
            Key::PageUp     => { for _ in 0..self.distance_from_top()    { self.move_cursor(Key::ArrowUp)   } },
            Key::PageDown   => { for _ in 0..self.distance_from_bottom() { self.move_cursor(Key::ArrowDown) } },
            Key::Home       => { self.cursor.x = self.screen.zero()      },
            Key::End        => { self.cursor.x = self.screen.cols() - 1  },
            _               => {}
        }
    }

    fn distance_from_top(&self)    -> usize {  self.cursor.y }
    fn distance_from_bottom(&self) -> usize {  self.screen.rows() - self.cursor.y }
    fn distance_from_left(&self)   -> usize {  self.cursor.x }
    fn distance_from_right(&self)  -> usize {  self.screen.cols() - self.cursor.x }

    fn read_key(&self) -> Key {
        let mut buff = [0u8; 1];
        let byte = self.read_byte(&mut buff);

        if byte == ctrl_key(b'q') { return Key::Quit }
        if byte == b'\x1b' {
            let mut seq = [0u8; 3];
            match self.read_byte(&mut seq[0..1]) {
                b'[' =>  {
                    match self.read_byte(&mut seq[1..2]) {
                        b'1' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Home     } return Key::ESC },
                        b'3' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Delete   } return Key::ESC },
                        b'4' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::End      } return Key::ESC },
                        b'5' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::PageUp   } return Key::ESC },
                        b'6' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::PageDown } return Key::ESC },
                        b'7' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Home     } return Key::ESC },
                        b'8' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::End      } return Key::ESC },
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
                    match self.read_byte(&mut seq[1..2]) {
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

    fn read_byte(&self, buff: &mut [u8]) -> u8 {
        let mut stdin = io::stdin();
        loop {
            match stdin.read(buff) {
                Ok(1) => return buff[0],
                Ok(_) => continue,
                Err(err) => {
                    self.end();
                    die(DieReason::Panic(err.to_string()))
                },
            }
        }
    }

    fn scroll(&mut self) {
        if self.cursor.y < self.cursor.y_offset {
            self.cursor.y_offset = self.cursor.y
        }

        if self.cursor.y >= self.cursor.y_offset + self.screen.rows() {
            self.cursor.y_offset = self.cursor.y - self.screen.rows() + 1;
        }

        if self.cursor.x < self.cursor.x_offset {
            self.cursor.x_offset = self.cursor.x
        }

        if self.cursor.x >= self.cursor.x_offset + self.screen.cols() {
            self.cursor.x_offset = self.cursor.x - self.screen.cols() + 1;
        }
    }

    fn draw_rows(&self) {
        let mut file_row;
        for screen_row in 0..self.screen.rows() {
            file_row = self.cursor.y_offset + screen_row;
            if file_row < self.text.lines.len() {
                let line = &self.text.lines[file_row];
                let start = self.cursor.x_offset;
                if start < line.len() {
                    let end = cmp::min(line.len(), start + self.screen.cols());
                    print!("{}", &line[start..end])
                }
            } else if self.text.lines.len() < self.screen.rows() {
                print!("~");

                //welcome message
                if self.text.lines.len() == 0 && screen_row == (self.screen.rows() / 3) {
                    let mut welcome = "kilo editor written in rust -- version 0.0.1";
                    let welcome_len = cmp::min(welcome.len(), self.screen.cols());
                    welcome = &welcome[..welcome_len];
                    let padding = (self.screen.cols() - 1 - welcome_len) / 2;
                    for _ in 0..padding { print!(" ") }
                    print!("{}", welcome);
                }
            }

            print!("\x1b[K"); //clear line
            if screen_row < self.screen.rows() - 1 {
                print!("\r\n");
            }
        }
    }

    fn max_x(&self) -> usize {
        if self.text.lines.len() > self.cursor.y {
            self.text.lines[self.cursor.y].len()
        } else {
            0
        }
    }

    fn max_y(&self) -> usize {
        cmp::max(self.text.lines.len(), self.screen.rows()) - 1
    }

    fn adjust_horizon(&mut self) {
        self.cursor.x = cmp::min(self.cursor.x, self.max_x()) //consider horizon
    }

    fn move_cursor(&mut self, key: Key) {
        match key {
            Key::ArrowUp => {
                if self.cursor.y > self.screen.zero() {
                    self.cursor.y -= 1;
                    self.adjust_horizon()
                }
            },
            Key::ArrowDown => {
                if self.cursor.y < self.max_y() {
                    self.cursor.y += 1;
                    self.adjust_horizon()
                }
            },
            Key::ArrowLeft => {
                if self.cursor.x > self.screen.zero() {
                    self.cursor.x -= 1;
                } else {
                    self.move_cursor(Key::ArrowUp)
                }
            },
            Key::ArrowRight => {
                if self.cursor.x < self.max_x() {
                    self.cursor.x += 1
                } else {
                    self.move_cursor(Key::ArrowDown)
                }
            },
            _ => {}
        }
    }
}
