use std::{cmp, time};
use super::Editor;
use super::super::common::*;

#[derive(Copy, Clone)]
pub enum InnerType {
    Save,
    Quit
}

pub struct Normal;
pub struct Prompt(pub InnerType);

pub trait InputMode {
    fn cursor_position(&self)   -> fn(&Editor) -> (usize, usize);
    fn scroll(&self)            -> fn(&mut Editor);
    fn write(&self)             -> Box<dyn Fn(&mut Editor, char)>;
    fn move_cursor(&self)       -> fn(&mut Editor, MoveKey);
    fn enter_pressed(&self)     -> fn(&mut Editor);
    fn delete_pressed(&self)    -> fn(&mut Editor);
    fn backspace_pressed(&self) -> fn(&mut Editor);
    fn set(self) -> Box<dyn FnOnce(&mut Editor)>;
}

impl InputMode for Normal {
    fn cursor_position(&self) -> fn(&Editor) -> (usize, usize) {
        |editor: &Editor| {
            let x =  editor.x_render() - editor.cursor.x_offset + 1;
            let y = editor.cursor.y - editor.cursor.y_offset + 1;
            (x, y)
        }
    }

    fn scroll(&self) -> fn(&mut Editor) {
        |editor: &mut Editor| {
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
    }

    fn write(&self) -> Box<dyn Fn(&mut Editor, char)> {
        Box::new(
            |editor: &mut Editor, c: char| {
                let cursor_y = editor.cursor.y;
                if editor.file.lines.len() > cursor_y {
                    let cursor_x = editor.cursor.x;
                    let line = &mut editor.file.lines[cursor_y];
                    if line.chars.len() > cursor_x {
                        line.insert(c, cursor_x);
                    } else {
                        line.push(c);
                    }
                } else {
                    let line: String = (c).into();
                    editor.file.add_new_line(&line, true);
                }
                editor.move_cursor(MoveKey::ArrowRight)
            })
    }

    fn move_cursor(&self) -> fn(&mut Editor, MoveKey) {
        |editor: &mut Editor, key: MoveKey| {
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
    }

    fn enter_pressed(&self) -> fn(&mut Editor) {
        |editor: &mut Editor| {
            editor.file.split_line(editor.cursor.y, editor.cursor.x);
            editor.move_cursor(MoveKey::ArrowRight);
        }
    }

    fn delete_pressed(&self) -> fn(&mut Editor) {
        |editor: &mut Editor| {
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
    }

    fn backspace_pressed(&self) -> fn(&mut Editor) {
        |editor: &mut Editor| {
            if editor.cursor.x == 0 && editor.cursor.y == 0 {
                return
            }

            if editor.cursor.x == 0 {
                let cursor_y = editor.cursor.y;
                editor.move_cursor(MoveKey::ArrowLeft);
                editor.file.merge_lines(cursor_y, editor.cursor.y);
                return
            }

            editor.move_cursor(MoveKey::ArrowLeft);
            editor.file.lines[editor.cursor.y].remove(editor.cursor.x);
            return
        }
    }

    fn set(self) -> Box<dyn FnOnce(&mut Editor)> {
        Box::new(
            |editor: &mut Editor| {
                editor.cursor.x = editor.cursor.x_normal;
                editor.status_bar.clear();
                editor.input_mode = Box::new(self)
            })
    }

}

impl InputMode for Prompt {
    fn cursor_position(&self) -> fn(&Editor) -> (usize, usize) {
        |editor: &Editor| {
            let x = editor.cursor.x + 1;
            let y = editor.screen.rows() + 2;
            (x, y)
        }
    }

    fn scroll(&self) -> fn(&mut Editor) {
        |_: &mut Editor| {}
    }

    fn write(&self) -> Box<dyn Fn(&mut Editor, char)> {
        let inner_type = self.0;
        Box::new(
            move |editor: &mut Editor, c: char| {
                if c == '\t' {
                    return
                }
                match inner_type {
                    InnerType::Save => {
                        editor.status_bar.prompt_insert(c, editor.cursor.x);
                        editor.move_cursor(MoveKey::ArrowRight)
                    },
                    InnerType::Quit => {
                        if c == 'y' || c == 'Y' {
                            editor.end(DieReason::Quit)
                        }
                        if c == 'n' || c == 'N' {
                            editor.change_input_mode(Normal)
                        }
                    }
                }
            })
    }

    fn move_cursor(&self) -> fn(&mut Editor, MoveKey) {
        |editor: &mut Editor, key: MoveKey| {
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
    }

    fn enter_pressed(&self) -> fn(&mut Editor) {
        |editor: &mut Editor| {
            let file_name = editor.status_bar.take_prompt();
            editor.file.set_name(file_name);
            editor.save();
            editor.change_input_mode(Normal);
        }
    }

    fn delete_pressed(&self) -> fn(&mut Editor) {
        |editor: &mut Editor| {
            editor.status_bar.prompt_delete(editor.cursor.x);
        }
    }

    fn backspace_pressed(&self) -> fn(&mut Editor) {
        |editor: &mut Editor| {
            if editor.status_bar.prompt_backspace(editor.cursor.x).is_some() {
                editor.move_cursor(MoveKey::ArrowLeft)
            }
        }
    }

    fn set(self) -> Box<dyn FnOnce(&mut Editor)> {
        Box::new(
            |editor: &mut Editor| {
                editor.cursor.x_normal = editor.cursor.x;
                editor.cursor.x = editor.message().len();
                editor.status_bar.set_timeout(time::Duration::from_mins(1));
                editor.input_mode = Box::new(self)
            })
    }
}
