pub struct File {
    pub name: Option<String>,
    pub lines: Vec<Line>,
}

pub struct Line {
    pub chars : Vec<char>,
    pub render: Vec<char>,
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
