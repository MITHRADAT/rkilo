pub struct Cursor {
    pub x        : usize,
    pub y        : usize,
    pub x_offset : usize,
    pub y_offset : usize,
    pub horizon  : usize,
        tab_stop : usize,
    pub x_render : usize,
    pub x_normal : usize,
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
            x_render: 0,
            x_normal: 0,
        }
    }

    pub fn tab_stop(&self) -> usize {
        self.tab_stop
    }
}
