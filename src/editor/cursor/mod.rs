pub struct Cursor {
    pub x        : usize,
    pub y        : usize,
    pub x_offset : usize,
    pub y_offset : usize,
    pub horizon  : usize,
        tab_stop : usize,
}

impl Cursor {
    pub fn get() -> Self {
        Self {
            x: 0,
            y: 0,
            x_offset: 0,
            y_offset: 0,
            horizon : 0,
            tab_stop: 8,
        }
    }

    pub fn tab_stop(&self) -> usize {
        self.tab_stop
    }
}
