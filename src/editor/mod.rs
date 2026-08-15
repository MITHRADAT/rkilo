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
    lines: Vec<Line>,
}

struct Line {
    chars : String,
    render: String
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
            .for_each(|line| {
                self.add_new_line(line);
            })
    }

    pub fn refresh_screen(&mut self) {
        self.scroll();
        print!("\x1b[?25l"); //hide the cursor
        print!("\x1b[H"); //reposition the cursor
        self.draw_rows();
        let cursor_position = format!("\x1b[{};{}H",
                                      self.cursor.y - self.cursor.y_offset + 1,
                                      self.cursor.x_render - self.cursor.x_offset + 1);
        print!("{}", cursor_position);
        print!("\x1b[?25h"); //show the cursor
        flush();
    }

    pub fn process_keypress(&mut self) {
        let input = self.read_key();
        match input {
            Key::ArrowUp    |
            Key::ArrowDown  |
            Key::ArrowLeft  |
            Key::ArrowRight |
            Key::PageUp     |
            Key::PageDown   => { self.move_cursor(input) },
            Key::Quit       => { clean_screen(); flush(); self.end(); process::exit(0) },
            Key::Home       => { self.cursor.x = 0; self.cursor.horizon = 0; },
            Key::End        => { self.cursor.x = self.max_x(); self.cursor.horizon = self.cursor.x;  },
            _               => {}
        }
    }

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
        self.cursor.x_render = self.x_render();

        if self.cursor.y < self.cursor.y_offset {
            self.cursor.y_offset = self.cursor.y
        }

        if self.cursor.y >= self.cursor.y_offset + self.screen.rows() {
            self.cursor.y_offset = self.cursor.y - self.screen.rows() + 1;
        }

        if self.cursor.x_render < self.cursor.x_offset {
            self.cursor.x_offset = self.cursor.x_render
        }

        if self.cursor.x_render >= self.cursor.x_offset + self.screen.cols() {
            self.cursor.x_offset = self.cursor.x_render - self.screen.cols() + 1;
        }
    }

    fn draw_rows(&self) {
        let mut file_row;
        for screen_row in 0..self.screen.rows() {
            file_row = self.cursor.y_offset + screen_row;
            if file_row < self.text.lines.len() {
                let line = &self.text.lines[file_row];
                let start = self.cursor.x_offset;
                if start < line.render.len() {
                    let end = cmp::min(line.render.len(), start + self.screen.cols());
                    print!("{}", &line.render[start..end])
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
            self.text.lines[self.cursor.y].chars.len()
        } else {
            0
        }
    }

    fn move_cursor(&mut self, key: Key) {
        match key {
            Key::ArrowUp => {
                if self.cursor.y > 0 {
                    self.cursor.y -= 1;
                }
                self.cursor.x = cmp::min(self.cursor.horizon, self.max_x())
            },
            Key::ArrowDown => {
                if self.cursor.y + 1 < self.text.lines.len() {
                    self.cursor.y += 1;
                }
                self.cursor.x = cmp::min(self.cursor.horizon, self.max_x())
            },
            Key::ArrowLeft => {
                if self.cursor.x > 0 {
                    self.cursor.x -= 1;
                } else if self.cursor.y > 0 {
                    self.cursor.y -= 1;
                    self.cursor.x = cmp::max(self.cursor.x, self.max_x());
                }
                self.cursor.horizon = self.cursor.x;
            },
            Key::ArrowRight => {
                if self.cursor.x < self.max_x() {
                    self.cursor.x += 1;
                } else if self.cursor.y + 1 < self.text.lines.len() {
                    self.cursor.y += 1;
                    self.cursor.x = 0;
                }
                self.cursor.horizon = self.cursor.x;
            },
            Key::PageUp => {
                if self.cursor.y >= self.screen.rows() {
                    self.cursor.y -= self.screen.rows()
                } else {
                    self.cursor.y = 0;
                }
                self.cursor.x = cmp::min(self.cursor.horizon, self.max_x())
            },
            Key::PageDown => {
                if self.cursor.y + self.screen.rows() < self.text.lines.len() {
                    self.cursor.y += self.screen.rows()
                } else {
                    self.cursor.y = self.text.lines.len().saturating_sub(1)
                }
                self.cursor.x = cmp::min(self.cursor.horizon, self.max_x())
            }
            _ => {}
        }
    }

    fn add_new_line(&mut self, line: &str) {
        let mut render = String::new();
        let mut index = 0 as usize;
        for c in line.chars() {
            if c == '\t' {
                let spaces = self.cursor.tab_stop() - (index % self.cursor.tab_stop());
                for _ in 0..spaces {
                    render.push(' ');
                }
                index += spaces;
            } else {
                render.push(c);
                index += 1;
            }
        }

        let new_line = Line {
            chars: String::from(line),
            render: render
        };

        self.text.lines.push(new_line)
    }

    fn x_render(&self) -> usize {
        let chars = self.text.lines[self.cursor.y].chars.as_bytes();
        let mut x_render = 0 as usize;
        for i in 0..self.cursor.x {
            if chars[i] == b'\t' {
                x_render += self.cursor.tab_stop() - (x_render % self.cursor.tab_stop())
            } else {
                x_render += 1
            }
        }

        x_render
    }

}
