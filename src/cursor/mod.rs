use super::config::Config;

pub static mut CURSOR: Cursor = Cursor {
    x        : 0,
    y        : 0,
    x_offset : 0,
    y_offset : 0,
};

pub struct Cursor {
    pub x        : usize,
    pub y        : usize,
    pub x_offset : usize,
    pub y_offset : usize,
}

pub fn distance_from_top() -> usize {
    unsafe { CURSOR.y }
}

pub fn distance_from_bottom() -> usize {
    unsafe { Config::get().screen_rows() - CURSOR.y }
}

pub fn distance_from_left() -> usize {
    unsafe { CURSOR.x }
}

pub fn distance_from_right() -> usize {
    unsafe { Config::get().screen_cols() - CURSOR.x }
}
