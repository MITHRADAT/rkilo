use std::cmp;
use super::Editor;
use super::super::common::*;

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
    fn move_cursor(&self, editor: &mut Editor, key: MoveKey);
    fn enter_pressed(&self, editor: &mut Editor);
    fn delete_pressed(&self, editor: &mut Editor);
    fn backspace_pressed(&self, editor: &mut Editor);
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

    fn move_cursor(&self, editor: &mut Editor, key: MoveKey) {
        match key {
            MoveKey::ArrowUp => {
                if editor.cursor.y > 0 {
                    editor.cursor.y -= 1;
                }
                editor.cursor.x = cmp::min(editor.cursor.horizon, editor.max_x())
            },
            MoveKey::ArrowDown => {
                if editor.cursor.y + 1 < editor.file.lines.len() {
                    editor.cursor.y += 1;
                }
                editor.cursor.x = cmp::min(editor.cursor.horizon, editor.max_x())
            },
            MoveKey::ArrowLeft => {
                if editor.cursor.x > 0 {
                    editor.cursor.x -= 1;
                } else if editor.cursor.y > 0 {
                    editor.cursor.y -= 1;
                    editor.cursor.x = editor.max_x();
                }
                editor.cursor.horizon = editor.cursor.x;
            },
            MoveKey::ArrowRight => {
                if editor.cursor.x < editor.max_x() {
                    editor.cursor.x += 1;
                } else if editor.cursor.y + 1 < editor.file.lines.len() {
                    editor.cursor.y += 1;
                    editor.cursor.x = 0;
                }
                editor.cursor.horizon = editor.cursor.x;
            },
            MoveKey::PageUp => {
                if editor.cursor.y >= editor.screen.rows() {
                    editor.cursor.y -= editor.screen.rows()
                } else {
                    editor.cursor.y = 0;
                }
                editor.cursor.x = cmp::min(editor.cursor.horizon, editor.max_x())
            },
            MoveKey::PageDown => {
                if editor.cursor.y + editor.screen.rows() < editor.file.lines.len() {
                    editor.cursor.y += editor.screen.rows()
                } else {
                    editor.cursor.y = editor.file.lines.len().saturating_sub(1)
                }
                editor.cursor.x = cmp::min(editor.cursor.horizon, editor.max_x())
            },
            MoveKey::Home => {
                editor.cursor.x = 0;
                editor.cursor.horizon = 0
            },
            MoveKey::End => {
                let max_x = editor.max_x();
                editor.cursor.x = max_x;
                editor.cursor.horizon = max_x
            }
        }
    }

    fn enter_pressed(&self, editor: &mut Editor) {
        editor.file.split_line(editor.cursor.y, editor.cursor.x);
        self.move_cursor(editor, MoveKey::ArrowRight);
    }

    fn delete_pressed(&self, editor: &mut Editor) {
        let max_x = editor.max_x();
        if editor.cursor.x == max_x && editor.cursor.y + 1 == editor.file.lines.len() {
            return
        }

        if editor.cursor.x == max_x {
            editor.file.merge_lines(editor.cursor.y + 1, editor.cursor.y);
            return
        }

        editor.file.lines[editor.cursor.y].remove(editor.cursor.x)
    }

    fn backspace_pressed(&self, editor: &mut Editor) {
        if editor.cursor.x == 0 && editor.cursor.y == 0 {
            return
        }

        if editor.cursor.x == 0 {
            let cursor_y = editor.cursor.y;
            self.move_cursor(editor, MoveKey::ArrowLeft);
            editor.file.merge_lines(cursor_y, editor.cursor.y);
            return
        }

        self.move_cursor(editor, MoveKey::ArrowLeft);
        editor.file.lines[editor.cursor.y].remove(editor.cursor.x);
        return
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

    fn move_cursor(&self, editor: &mut Editor, key: MoveKey) {
        match key {
            MoveKey::ArrowLeft => {
                if editor.status_bar.prompt_index(editor.cursor.x) > 0 {
                    editor.cursor.x -= 1;
                }
            },
            MoveKey::ArrowRight => {
                if editor.cursor.x < editor.message().len() {
                    editor.cursor.x += 1;
                }
            },
            MoveKey::Home => {
                editor.cursor.x = 0
            },
            MoveKey::End => {
                editor.cursor.x = editor.max_x()
            },
            MoveKey::ArrowUp   => { return },
            MoveKey::ArrowDown => { return },
            MoveKey::PageUp    => { return },
            MoveKey::PageDown  => { return },
        }
    }

    fn enter_pressed(&self, editor: &mut Editor) {
        self.commit(editor);
    }

    fn delete_pressed(&self, editor: &mut Editor) {
        editor.status_bar.prompt_delete(editor.cursor.x);
    }

    fn backspace_pressed(&self, editor: &mut Editor) {
        if editor.status_bar.prompt_backspace(editor.cursor.x).is_some() {
            self.move_cursor(editor, MoveKey::ArrowLeft)
        }
    }

}

impl Prompt {
    fn commit(&self, editor: &mut Editor) {
        let file_name = editor.status_bar.take_prompt();
        editor.file.set_name(file_name);
        editor.save()
    }
}
