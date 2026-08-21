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
            self.request_name();
            self.write_to_disk()
        }
    }

    fn request_name(&mut self) {
        let file_name = String::new();
        self.name = Some(file_name)
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
                temp.write_all(&line.original)?
            }

            let mut buffer = [0; 4];
            for line in &self.lines[first_dirty..] {
                for c in &line.chars {
                    buffer = [0; 4];
                    temp.write_all(c.encode_utf8(&mut buffer).as_bytes())?
                }
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
}

pub struct Line {
    pub chars : Vec<char>,
    pub render: Vec<char>,
    pub original: Vec<u8>,
    pub dirty: bool
}

impl Line {
    pub fn insert(&mut self, c: char, chars_index: usize, render_index: usize) {
        self.chars.insert(chars_index, c);
        self.render.insert(render_index, c);
        self.dirty = true;
    }

    pub fn push(&mut self, c: char) {
        self.chars.push(c);
        self.render.push(c);
        self.dirty = true;
    }
}

pub enum SaveStatus {
    NoChanges,
    Successful,
    Fail(io::Error)
}
