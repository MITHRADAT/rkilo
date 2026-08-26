enum PromptType {
    Save,
    Quit
}

struct Normal;
struct Prompt {
    prompt_type: PromptType
}

pub trait InputMode {
    fn cursor_position(&self) -> (usize, usize);
    fn scroll(&self);
    fn write(&self);
    fn move_cursor(&self);
    fn enter_pressed(&self);
    fn delete_pressed(&self);
    fn backspace_pressed(&self);
    fn commit(&self);
}

impl InputMode for Normal {
    fn cursor_position(&self) -> (usize, usize) {
        (0, 0)
    }

    fn scroll(&self) {

    }
    
    fn write(&self) {

    }

    fn move_cursor(&self) {

    }

    fn enter_pressed(&self) {

    }

    fn delete_pressed(&self) {

    }

    fn backspace_pressed(&self) {

    }

    fn commit(&self) {

    }
}

impl InputMode for Prompt {
    fn cursor_position(&self) -> (usize, usize) {
        (0, 0)
    }

    fn scroll(&self) {

    }

    fn write(&self) {

    }

    fn move_cursor(&self) {
        
    }

    fn enter_pressed(&self) {

    }

    fn delete_pressed(&self) {

    }

    fn backspace_pressed(&self) {

    }

    fn commit(&self) {

    }
}
