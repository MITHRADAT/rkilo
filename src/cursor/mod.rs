use super::config::Config;

pub static mut CURSOR: Cursor = Cursor {
    x: 0,
    y: 0
};

pub struct Cursor {
   pub x: u16,
   pub y: u16
}

pub fn distance_from_top() -> u16 {
    unsafe { CURSOR.y }
}

pub fn distance_from_bottom() -> u16 {
    unsafe { Config::get().screen_rows() - CURSOR.y }
}

pub fn distance_from_left() -> u16 {
    unsafe { CURSOR.x }
}

pub fn distance_from_right() -> u16 {
    unsafe { Config::get().screen_cols() - CURSOR.x }
}
