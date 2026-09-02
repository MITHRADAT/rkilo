mod logger; mod common; mod editor;
use editor::Editor;

fn main() {
    let mut editor = Editor::init();
    
    loop {
        editor.refresh_screen();
        editor.process_keypress();
    }
}
