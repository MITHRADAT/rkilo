use std::{fs, io::{self, Write}, time};
use super::super::common::*;

pub struct File {
    pub name : Option<String>,
    pub lines: Vec<Line>,
}

impl File {
    pub fn new() -> Self {
        Self {
            name   : None,
            lines  : vec![],
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string())
    }

    pub fn clear_name(&mut self) {
        self.name = None
    }

    pub fn persist(&mut self) -> SaveStatus {
        let Some(full_name) = self.name.as_ref()
        else { return SaveStatus::NameRequest };
        let full_name = full_name.to_string();
        let Some(_file_name) = full_name
            .rsplit('/')
            .next()
            .filter(|name| !name.is_empty())
        else {
            return SaveStatus::Fail(io::Error::new(
                io::ErrorKind::InvalidInput, "can not find a file name to save"))
        };

        if self.lines.len() == 0 {
            self.add_new_line("", true);
        }

        let Some(first_dirty) = self.lines.iter()
            .position(|line| line.dirty)
        else { return SaveStatus::NoChanges };

        let mut result = || -> io::Result<()> {
            let temp_name = format!("{}.rkilotemp{:?}", full_name, time::SystemTime::now());
            let mut temp = fs::File::create(&temp_name)?;

            for line in &self.lines[..first_dirty] {
                temp.write_all(&line.persist)?;
                temp.write_all(b"\n")?
            }

            let mut buffer = [0; 4];
            for line in &mut self.lines[first_dirty..] {
                line.persist.clear();
                for c in &line.chars {
                    buffer = [0; 4];
                    let bytes = c.encode_utf8(&mut buffer).as_bytes();
                    line.persist.extend_from_slice(bytes);
                    temp.write_all(bytes)?
                }
                temp.write_all(b"\n")?
            }

            temp.sync_all()?;
            drop(temp);

            fs::rename(temp_name, full_name)?;
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

    pub fn is_dirty(&self) -> bool {
        self.lines.iter().any(|line| line.dirty)
    }
}

pub struct Line {
    pub chars : Vec<char>,
    pub render: Vec<char>,
    pub persist: Vec<u8>,
    pub dirty: bool
}

impl Line {
    pub fn new(line: &str, dirty: bool) -> Self {
        let mut new_line = Self {
            chars: line.chars().collect(),
            render: vec![],
            persist: line.as_bytes().to_vec(),
            dirty: dirty
        };
        new_line.render();
        new_line
    }

    pub fn new_empty() -> Self {
        Self {
            chars: vec![],
            render: vec![],
            persist: vec![],
            dirty: true
        }
    }

    pub fn insert(&mut self, c: char, chars_index: usize) {
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
