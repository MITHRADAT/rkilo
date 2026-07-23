use std::env;

mod logger;
mod common;
mod editor;

use editor::Editor;

fn main() {
    let mut editor = Editor::init();
    
    if let Some(path) = env::args().nth(1) {
        editor.read_file(&path)
    }
    
    loop {
        editor.refresh_screen();
        editor.process_keypress();
    }
}
