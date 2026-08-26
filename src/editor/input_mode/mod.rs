use super::Editor;

enum PromptType {
    Save,
    Quit
}

struct Normal;
struct Prompt {
    prompt_type: PromptType
}

pub trait InputMode {
    fn cursor_position(&self, editor: &Editor) -> (usize, usize);
    fn scroll(&self, editor: &mut Editor);
    fn write(&self);
    fn move_cursor(&self);
    fn enter_pressed(&self);
    fn delete_pressed(&self);
    fn backspace_pressed(&self);
    fn commit(&self);
}

impl InputMode for Normal {
    fn cursor_position(&self, editor: &Editor) -> (usize, usize) {
        let x =  editor.x_render() - editor.cursor.x_offset + 1;
        let y = editor.cursor.y - editor.cursor.y_offset + 1;
        (x, y)
    }

    fn scroll(&self, editor: &mut Editor) {
        let x_render = editor.x_render();

        if editor.cursor.y < editor.cursor.y_offset {
            editor.cursor.y_offset = editor.cursor.y
        }

        if editor.cursor.y >= editor.cursor.y_offset + editor.screen.rows() {
            editor.cursor.y_offset = editor.cursor.y - editor.screen.rows() + 1;
        }

        if x_render < editor.cursor.x_offset {
            editor.cursor.x_offset = x_render
        }

        if x_render >= editor.cursor.x_offset + editor.screen.cols() {
            editor.cursor.x_offset = x_render - editor.screen.cols() + 1;
        }
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
    fn cursor_position(&self, editor: &Editor) -> (usize, usize) {
        let x = editor.cursor.x + 1;
        let y = editor.screen.rows() + 2;
        (x, y)
    }

    fn scroll(&self, _: &mut Editor) {
        return
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
