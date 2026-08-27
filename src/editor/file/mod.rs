use std::{fs, io::{self, Write}};
use super::super::common::*;

pub struct File {
    pub name: Option<String>,
    pub lines: Vec<Line>,
}

impl File {
    pub fn save(&mut self) -> SaveStatus {
        if self.name.is_some() {
            self.write_to_disk()
        } else {
            SaveStatus::NameRequest
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name)
    }

    fn write_to_disk(&mut self) -> SaveStatus {
        let Some(first_dirty) = self.lines.iter()
            .position(|line| line.dirty)
        else { return SaveStatus::NoChanges };

        let result = || -> io::Result<()> {
            let file_name = self.name.as_ref().unwrap();
            let temp_name = format!("{}.temp", file_name);
            let mut temp = fs::File::create(&temp_name)?;

            for line in &self.lines[..first_dirty] {
                temp.write_all(&line.original)?;
                temp.write_all(b"\n")?
            }

            let mut buffer = [0; 4];
            for line in &self.lines[first_dirty..] {
                for c in &line.chars {
                    buffer = [0; 4];
                    temp.write_all(c.encode_utf8(&mut buffer).as_bytes())?
                }
                temp.write_all(b"\n")?
            }

            temp.sync_all()?;
            drop(temp);

            fs::rename(temp_name, file_name)?;
            Ok(())
        };

        match result() {
            Ok(()) => {
                for line in &mut self.lines[first_dirty..] {
                    line.dirty = false;
                }
                SaveStatus::Successful
            },
            Err(error) => {
                SaveStatus::Fail(error)
            }
        }
    }

    pub fn add_new_line(&mut self, line: &str, dirty: bool) {
        let new_line = Line::new(line, dirty);
        self.lines.push(new_line)
    }

    pub fn split_line(&mut self, line_index: usize, chars_index: usize) {
        let mut new_line = Line::new_empty();
        if self.lines.len() == 0 {
            self.lines.push(new_line);
            new_line = Line::new_empty();
            self.lines.push(new_line);
            return
        }
        let line = &mut self.lines[line_index];
        line.dirty = true;
        new_line.chars = line.chars.split_off(chars_index);
        line.render();
        new_line.render();
        self.lines.insert(line_index + 1, new_line)
    }

    pub fn merge_lines(&mut self, from_index: usize, to_index: usize) {
        if from_index == to_index || self.lines.len() == 0 {
            return
        }
        let from = self.lines.remove(from_index);
        let to = &mut self.lines[to_index];
        for c in from.chars {
            to.push(c)
        }
        to.dirty = true;
        to.render()
    }
}

pub struct Line {
    pub chars : Vec<char>,
    pub render: Vec<char>,
    pub original: Vec<u8>,
    pub dirty: bool
}

impl Line {
    pub fn new(line: &str, dirty: bool) -> Self {
        let mut new_line = Self {
            chars: line.chars().collect(),
            render: vec![],
            original: line.as_bytes().to_vec(),
            dirty: dirty
        };
        new_line.render();
        new_line
    }

    pub fn new_empty() -> Self {
        Self {
            chars: vec![],
            render: vec![],
            original: vec![],
            dirty: true
        }
    }

    pub fn insert(&mut self, c: char, chars_index: usize, render_index: usize) {
        self.chars.insert(chars_index, c);
        self.render();
        self.dirty = true;
    }

    pub fn push(&mut self, c: char) {
        self.chars.push(c);
        self.render.push(c);
        self.dirty = true;
    }

    pub fn remove(&mut self, chars_index: usize) {
        self.chars.remove(chars_index);
        self.render();
        self.dirty = true;
    }

    fn render(&mut self) {
        self.render.clear();
        let mut index = 0 as usize;
        let tab_stop = tab_stop();
        for c in &self.chars {
            if *c == '\t' {
                let spaces = tab_stop - (index % tab_stop);
                for _ in 0..spaces {
                    self.render.push(' ');
                }
                index += spaces;
            } else {
                self.render.push(*c);
                index += 1;
            }
        }
    }
}

pub enum SaveStatus {
    NoChanges,
    Successful,
    NameRequest,
    Fail(io::Error)
}
