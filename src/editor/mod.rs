use std::{io::{self, Read}, fs, cmp, process, time, env};

mod cursor;
mod screen;
mod file;
mod message_status;

use cursor::Cursor;
use screen::Screen;
use message_status::MessageStatus;
use file::*;
use super::common::*;

#[derive(PartialEq)]
enum PromptType {
    Save
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Prompt(PromptType),
}

pub struct Editor {
    file  : File,
    screen: Screen,
    cursor: Cursor,
    message_status: MessageStatus,
    input_mode: InputMode
}

impl Editor {
    pub fn init() -> Self {
        let screen = Screen::get();
        let editor = Self {
            file  : File {
                name: None,
                lines: vec![],
            },
            message_status: MessageStatus::new("Help: Ctrl-Q: quit, Ctrl-S: save"),
            cursor: Cursor::get(),
            screen: screen,
            input_mode: InputMode::Normal
        };

        editor.screen.enable_raw_mode();
        editor
    }

    fn end(&self) {
        self.screen.disable_raw_mode();
    }

    pub fn read_file(&mut self, path: &str) {
        self.file.name = Some(path.to_string());
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
        self.draw_status_bar();
        self.draw_message_bar();
        self.show_cursor();
        flush();
    }

    fn show_cursor(&self) {
        let mut cursor_position = String::new();
        match self.input_mode {
            InputMode::Normal => {
                cursor_position = format!("\x1b[{};{}H",
                                          self.cursor.y - self.cursor.y_offset + 1,
                                          self.cursor.x_render - self.cursor.x_offset + 1);
            },
            InputMode::Prompt(_) => {
                cursor_position = format!("\x1b[{};{}H",
                                          self.screen.rows() + 2,
                                          self.cursor.x + 1
                )
            }
        }
        print!("{}", cursor_position);
        print!("\x1b[?25h"); //show the cursor
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
            Key::End        => { self.cursor.x = self.max_x(); self.cursor.horizon = self.cursor.x; },
            Key::Save       => { self.save() }
            Key::Enter      => { self.enter_pressed() }
            Key::Char(c)    => { self.write(c)}
            _               => {}
        }
    }

    fn read_key(&self) -> Key {
        let mut buff = [0u8; 1];
        let byte = self.read_byte(&mut buff);

        if byte == ctrl_key(b'q') { return Key::Quit }
        if byte == ctrl_key(b's') { return Key::Save }
        if byte == b'\r'          { return Key::Enter }
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
        if self.input_mode != InputMode::Normal {
            return
        }

        if self.file.lines.len() > self.cursor.y {
            self.cursor.x_render = self.x_render();
        }

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
            if file_row < self.file.lines.len() {
                let line = &self.file.lines[file_row];
                let start = self.cursor.x_offset;
                if start < line.render.len() {
                    let end = cmp::min(line.render.len(), start + self.screen.cols());
                    let display_line: String = line.render[start..end].iter().collect();
                    print!("{}", display_line)
                }
            } else if self.file.lines.len() < self.screen.rows() {
                print!("~");

                //welcome message
                if self.file.lines.len() == 0 && screen_row == (self.screen.rows() / 3) {
                    let mut welcome = "kilo editor written in rust -- version 0.0.1";
                    let welcome_len = cmp::min(welcome.len(), self.screen.cols());
                    welcome = &welcome[..welcome_len];
                    let padding = (self.screen.cols() - 1 - welcome_len) / 2;
                    for _ in 0..padding { print!(" ") }
                    print!("{}", welcome);
                }
            }

            print!("\x1b[K"); //clear line
            print!("\r\n");
        }
    }

    fn draw_status_bar(&self) {
        print!("\x1b[7m"); //0: clear all attribute, 1: bold, 4: underscore, 5: blink, 7: inverted color

        let mut display_name = String::from("scratch");
        let mut current_line_number = String::from("");
        if let Some(file_name) = &self.file.name {
            if file_name.len() > 19 {
                display_name = format!("{} - {} lines", &file_name[..19], self.file.lines.len());
            } else {
                display_name = format!("{} - {} lines", &file_name, self.file.lines.len());
            }
            current_line_number = format!("{}/{}", self.cursor.y + 1, self.file.lines.len());
        }
        if display_name.len() < self.screen.cols() {
            print!("{}", display_name);
        } else {
            print!("{}", &display_name[..self.screen.cols()]);
        }

        for i in display_name.len()..self.screen.cols() {
            if self.screen.cols() - i == current_line_number.len() {
                print!("{}", current_line_number);
                break;
            } else {
                print!(" ");
            }
        }

        print!("\x1b[m"); //switch back to normal formatting, equal to x1b[0m
        print!("\r\n");
    }

    fn draw_message_bar(&self) {
        print!("\x1b[K"); //clear the line

        let message = self.message_status.message()
            .unwrap_or_else(|error| {
                self.end();
                die(DieReason::Panic(error.to_string()))
            });

        if message.len() > self.screen.cols() {
            print!("{}", &message[..self.screen.cols()]);
        } else {
            print!("{}", message);
        }
    }

    fn max_x(&self) -> usize {
        if self.file.lines.len() > self.cursor.y {
            self.file.lines[self.cursor.y].chars.len()
        } else {
            0
        }
    }

    fn move_cursor(&mut self, key: Key) {
        match self.input_mode {
            InputMode::Normal => { self.move_cursor_normal(key) },
            InputMode::Prompt(_) => { self.move_cursor_prompt(key) }
        }
    }

    fn move_cursor_normal(&mut self, key: Key) {
        match key {
            Key::ArrowUp => {
                if self.cursor.y > 0 {
                    self.cursor.y -= 1;
                }
                self.cursor.x = cmp::min(self.cursor.horizon, self.max_x())
            },
            Key::ArrowDown => {
                if self.cursor.y + 1 < self.file.lines.len() {
                    self.cursor.y += 1;
                }
                self.cursor.x = cmp::min(self.cursor.horizon, self.max_x())
            },
            Key::ArrowLeft => {
                if self.cursor.x > 0 {
                    self.cursor.x -= 1;
                } else if self.cursor.y > 0 {
                    self.cursor.y -= 1;
                    self.cursor.x = self.max_x();
                }
                self.cursor.horizon = self.cursor.x;
            },
            Key::ArrowRight => {
                if self.cursor.x < self.max_x() {
                    self.cursor.x += 1;
                } else if self.cursor.y + 1 < self.file.lines.len() {
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
                if self.cursor.y + self.screen.rows() < self.file.lines.len() {
                    self.cursor.y += self.screen.rows()
                } else {
                    self.cursor.y = self.file.lines.len().saturating_sub(1)
                }
                self.cursor.x = cmp::min(self.cursor.horizon, self.max_x())
            }
            _ => {}
        }
    }

    fn move_cursor_prompt(&mut self, key: Key) {
        match key {
            Key::ArrowLeft => {
                if self.cursor.x > 0 {
                    self.cursor.x -= 1;
                }
            },
            Key::ArrowRight => {
                if self.cursor.x < self.message_status.message().unwrap().len() {
                    self.cursor.x += 1;
                }
            },
            _ => {}
        }
    }

    fn add_new_line(&mut self, line: &str) {
        let mut render = vec![];
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
            chars: line.chars().collect(),
            render: render,
            original: line.as_bytes().to_vec(),
            dirty: false
        };

        self.file.lines.push(new_line)
    }

    fn x_render(&self) -> usize {
        let chars = &self.file.lines[self.cursor.y].chars;
        let mut x_render = 0 as usize;
        for i in 0..self.cursor.x {
            if chars[i] == '\t' {
                x_render += self.cursor.tab_stop() - (x_render % self.cursor.tab_stop())
            } else {
                x_render += 1
            }
        }

        x_render
    }

    fn write(&mut self, key: u8) {
        match self.input_mode {
            InputMode::Normal => self.write_normal(key),
            InputMode::Prompt(_) => self.write_prompt(key)
        }

        self.move_cursor(Key::ArrowRight);
    }

    fn write_normal(&mut self, key: u8) {
        if self.file.lines.len() > self.cursor.y {
            let render_index = self.x_render();
            let chars_index = self.cursor.x;
            let line = &mut self.file.lines[self.cursor.y];
            if line.chars.len() > chars_index {
                line.insert(key as char, chars_index, render_index, self.cursor.tab_stop());
            } else {
                line.push(key as char);
            }
        } else {
            let line: String = (key as char).into();
            self.add_new_line(&line);
        }

    }

    fn write_prompt(&mut self, key: u8) {
        self.message_status.push_prompt(key as char);
    }

    fn save(&mut self) {
        match self.file.save() {
            SaveStatus::Successful => {
                self.message_status.set_message(
                    &format!("{} saved sucessfully!", &self.file.name.as_ref().unwrap()))
            },
            SaveStatus::NoChanges => {
                self.message_status.set_message(
                    &format!("no changes to save!"))
            },
            SaveStatus::NameRequest => {
                self.request_file_name()
            }
            SaveStatus::Fail(error) => {
                self.message_status.set_message(
                    &format!("error occured while saving: {}", error))
            }
        }
    }

    fn request_file_name(&mut self) {
        let dir = env::current_dir()
            .unwrap_or_else(|err| {
                self.end();
                die(DieReason::Panic(err.to_string()))
            });
        self.message_status.set(&format!("file name to save: {}/", dir.display()), time::Duration::from_mins(1));
        self.change_input_mode(InputMode::Prompt(PromptType::Save));
    }

    fn enter_pressed(&mut self) {
        match self.input_mode {
            InputMode::Normal => { },
            InputMode::Prompt(_) => { self.commit() }
        }
    }

    fn commit(&mut self) {
        match self.input_mode {
            InputMode::Prompt(PromptType::Save) => {
                let file_name = self.message_status.take_prompt();
                self.file.set_name(file_name);
                self.save()
            },
            InputMode::Normal => {}
        }

        self.change_input_mode(InputMode::Normal);
    }

    fn change_input_mode(&mut self, target: InputMode) {
        match target {
            InputMode::Normal => {
                self.cursor.x = self.cursor.x_normal;
            },
            InputMode::Prompt(_) => {
                self.cursor.x_normal = self.cursor.x;
                self.cursor.x = self.message_status.message().unwrap().len()
            },
        };

        self.input_mode = target;
    }
}
