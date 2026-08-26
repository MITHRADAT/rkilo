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
    fn write(&self, editor: &mut Editor, c: char);
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

    fn write(&self, editor: &mut Editor, c: char) {
        let cursor_y = editor.cursor.y;
        if editor.file.lines.len() > cursor_y {
            let x_render = editor.x_render();
            let cursor_x = editor.cursor.x;
            let line = &mut editor.file.lines[cursor_y];
            if line.chars.len() > cursor_x {
                line.insert(c, cursor_x, x_render);
            } else {
                line.push(c);
            }
        } else {
            let line: String = (c).into();
            editor.file.add_new_line(&line, true);
        }
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

    fn write(&self, editor: &mut Editor, c: char) {
        if c == '\t' {
            return
        }
        editor.status_bar.prompt_insert(c, editor.cursor.x);
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
