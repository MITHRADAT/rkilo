use std::{io::{self, Read}, cmp, env};

mod cursor; mod screen; mod file; mod status_bar; mod input_mode;
use cursor::Cursor; use screen::Screen; use status_bar::StatusBar;
use input_mode::*; use file::*; use super::common::*;


pub struct Editor {
    file  : File,
    screen: Screen,
    cursor: Cursor,
    status_bar: StatusBar,
    input_mode: Box<dyn InputMode>
}

impl Editor {
    pub fn init() -> Self {
        let screen = Screen::get();
        let mut editor = Self {
            file  : File::new(),
            status_bar: StatusBar::new("Help: Ctrl-Q: quit, Ctrl-S: save, Ctrl-W: save as, Ctrl-O: open"),
            cursor: Cursor::get(),
            screen: screen,
            input_mode: Box::new(Normal)
        };

        editor.screen.enable_raw_mode();
        editor.process_args();
        editor
    }

    fn process_args(&mut self) {
        if let Some(path) = env::args().nth(1) {
            match self.file.set_path(&path) {
                Ok(_) => { self.read_file()  },
                Err(reason) => { self.end(reason) }
            }
        }
    }

    fn end(&self, reason: DieReason) -> ! {
        self.screen.disable_raw_mode();
        die(reason)
    }

    pub fn read_file(&mut self) {
        match self.file.read() {
            Ok(_) => {},
            Err(reason) => self.end(reason)
        }
    }

    pub fn refresh_screen(&mut self) {
        self.scroll();
        print!("\x1b[?25l"); //hide the cursor
        print!("\x1b[H"); //reposition the cursor
        self.draw_rows();
        self.draw_info_bar();
        self.draw_status_bar();
        self.show_cursor();
        flush();
    }

    fn show_cursor(&self) {
        let f = self.input_mode.cursor_position();
        let (x, y) = f(self);
        let cursor_position = format!("\x1b[{};{}H", y, x);
        print!("{}", cursor_position);
        print!("\x1b[?25h"); //show the cursor
    }

    pub fn process_keypress(&mut self) {
        let input = self.read_key();
        let f = self.input_mode.process_keypress();
        f(self, input);
    }

    fn read_key(&self) -> Key {
        let mut buff = [0u8; 1];
        let byte = self.read_byte(&mut buff);

        if byte == ctrl_key(b'h') { return Key::BackSpace }
        if byte == ctrl_key(b'l') { return Key::ESC }
        if byte == ctrl_key(b'q') { return Key::Quit }
        if byte == ctrl_key(b's') { return Key::Save }
        if byte == ctrl_key(b'o') { return Key::Open }
        if byte == ctrl_key(b'w') { return Key::SaveAs }
        if byte == 8              { return Key::BackSpace }
        if byte == 127            { return Key::BackSpace }
        if byte == b'\r'          { return Key::Enter }
        if byte == b'\x1b' {
            let mut seq = [0u8; 3];
            match self.read_byte(&mut seq[0..1]) {
                b'[' =>  {
                    match self.read_byte(&mut seq[1..2]) {
                        b'1' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Move(MoveKey::Home)     } return Key::ESC },
                        b'3' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Delete                  } return Key::ESC },
                        b'4' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Move(MoveKey::End)      } return Key::ESC },
                        b'5' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Move(MoveKey::PageUp)   } return Key::ESC },
                        b'6' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Move(MoveKey::PageDown) } return Key::ESC },
                        b'7' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Move(MoveKey::Home)     } return Key::ESC },
                        b'8' => { if self.read_byte(&mut seq[2..3]) == b'~' { return Key::Move(MoveKey::End)      } return Key::ESC },
                        b'A' => return Key::Move(MoveKey::ArrowUp),
                        b'B' => return Key::Move(MoveKey::ArrowDown),
                        b'C' => return Key::Move(MoveKey::ArrowRight),
                        b'D' => return Key::Move(MoveKey::ArrowLeft),
                        b'F' => return Key::Move(MoveKey::End),
                        b'H' => return Key::Move(MoveKey::Home),
                        _    => return Key::ESC,
                    }
                },
                b'O' => {
                    match self.read_byte(&mut seq[1..2]) {
                        b'H' => return Key::Move(MoveKey::Home),
                        b'F' => return Key::Move(MoveKey::End),
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
                    self.end(DieReason::Panic(err.to_string()))
                },
            }
        }
    }

    fn scroll(&mut self) {
        let f = self.input_mode.scroll();
        f(self)
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

    fn draw_info_bar(&self) {
        print!("\x1b[7m"); //0: clear all attribute, 1: bold, 4: underscore, 5: blink, 7: inverted color

        let mut display_name = String::from("scratch");
        let mut current_line_number = String::from("");
        if let Some(file_name) = self.file.name() {
            let file_name = file_name.to_string_lossy();
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

    fn draw_status_bar(&self) {
        print!("\x1b[K"); //clear the line

        let message = self.status_bar.message()
            .unwrap_or_else(|error| {
                self.end(DieReason::Panic(error.to_string()))
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

    fn x_render(&self) -> usize {
        if self.file.lines.len() == 0 {
            return 0
        }

        let chars = &self.file.lines[self.cursor.y].chars;
        let mut x_render = 0 as usize;
        let tab_stop = tab_stop();
        for i in 0..self.cursor.x {
            if chars[i] == '\t' {
                x_render += tab_stop - (x_render % tab_stop)
            } else {
                x_render += 1
            }
        }

        x_render
    }

    fn quit(&mut self) {
        if self.file.is_dirty() {
            self.status_bar.set_message(format!("there are some unsaved changes. are you sure? (y/n)"));
            self.change_input_mode(Prompt(InnerType::Quit));
        } else {
            self.end(DieReason::Quit)
        }
    }

    fn flush(&mut self) {
        self.cursor.refresh();
        self.file.lines.clear();
        self.status_bar.clear();
    }

    fn save_as(&mut self) {
        self.file.make_dirty();
        self.request("file name to save: ");
        self.change_input_mode(Prompt(InnerType::Save))
    }

    fn save(&mut self) {
        match self.file.persist() {
            SaveStatus::Successful => {
                self.status_bar.set_message(
                    format!("{} saved successfully!", self.file.path().unwrap().display()));
                self.change_input_mode(Normal)
            },
            SaveStatus::NoChanges => {
                self.status_bar.set_message(
                    format!("no changes to save!"));
                self.change_input_mode(Normal)
            },
            SaveStatus::NameRequest => {
                self.request("file name to save: ");
                self.change_input_mode(Prompt(InnerType::Save))
            },
            SaveStatus::Fail(error) => {
                self.file.clear_path();
                self.status_bar.set_message(
                    format!("error occurred while saving: {}", error));
                self.change_input_mode(Normal)
            }
        }
    }

    fn request(&mut self, msg: &str) {
        let dir = env::current_dir()
            .unwrap_or_else(|err| {
                self.end(DieReason::Panic(err.to_string()))
            });
        self.status_bar.set_message(msg.to_string());
        self.status_bar.set_prompt(format!("{}/", dir.display()));
    }

    fn change_input_mode<T>(&mut self, input_mode: T)
    where
        T: InputMode
    {
        let f = input_mode.set();
        f(self);
    }

    fn message(&self) -> String {
        self.status_bar.message()
            .unwrap_or_else(|err| {
                self.end(DieReason::Panic(err.to_string()))
            })
    }
}
