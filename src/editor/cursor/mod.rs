pub struct Cursor {
    pub x        : usize,
    pub y        : usize,
    pub x_offset : usize,
    pub y_offset : usize,
    pub horizon  : usize,
        stored_x : Option<usize>,
}

impl Cursor {
    pub fn get() -> Self {
        Self {
            x: 0,
            y: 0,
            x_offset: 0,
            y_offset: 0,
            horizon : 0,
            stored_x: None,
        }
    }

    pub fn refresh(&mut self) {
        self.x = 0;
        self.y = 0;
        self.x_offset = 0;
        self.y_offset = 0;
        self.horizon = 0;
        self.stored_x = None;
    }

    pub fn store_x(&mut self) {
        if self.stored_x.is_some() {
            return
        }
        self.stored_x = Some(self.x)
    }

    pub fn restore_x(&mut self) {
        let stored_x = self.stored_x.take();
        if stored_x.is_none() {
            return
        }
        self.x = stored_x.unwrap()
    }
}
