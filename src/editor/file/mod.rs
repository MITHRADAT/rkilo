use std::{fs, io::{self, Write}};

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

    pub fn add_new_line(&mut self, line: &str, tab_stop: usize) {
        let new_line = Line::new(line, tab_stop);
        self.lines.push(new_line)
    }
}

pub struct Line {
    pub chars : Vec<char>,
    pub render: Vec<char>,
    pub original: Vec<u8>,
    pub dirty: bool
}

impl Line {
    pub fn new(line: &str, tab_stop: usize) -> Self {
        let mut new_line = Self {
            chars: line.chars().collect(),
            render: vec![],
            original: line.as_bytes().to_vec(),
            dirty: false
        };
        new_line.render(tab_stop);
        new_line
    }
    pub fn insert(&mut self, c: char, chars_index: usize, render_index: usize, tab_stop: usize) {
        self.chars.insert(chars_index, c);
        if c == '\t' {
            let spaces = tab_stop - (render_index % tab_stop);
            for _ in 0..spaces {
                self.render.insert(render_index, ' ');
            }
        } else {
            self.render.insert(render_index, c);
        }
        self.dirty = true;
    }

    pub fn push(&mut self, c: char) {
        self.chars.push(c);
        self.render.push(c);
        self.dirty = true;
    }
    fn render(&mut self, tab_stop: usize) {
        self.render.clear();
        let mut index = 0 as usize;
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
