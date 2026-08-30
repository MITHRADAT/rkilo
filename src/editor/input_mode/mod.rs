use std::{cmp, path::Path};
use super::Editor;
use super::super::common::*;

#[derive(Copy, Clone)]
pub enum InnerType {
    Save,
    Quit,
    Overwrite,
}

pub struct Normal;
pub struct Prompt(pub InnerType);

pub trait InputMode {
    fn cursor_position(&self)   -> fn(&Editor) -> (usize, usize);
    fn process_keypress(&self)  -> Box<dyn Fn(&mut Editor, Key)>;
    fn scroll(&self)            -> fn(&mut Editor);
    fn set(self)                -> Box<dyn FnOnce(&mut Editor)>;
}

impl InputMode for Normal {
    fn cursor_position(&self) -> fn(&Editor) -> (usize, usize) {
        |editor: &Editor| {
            let x =  editor.x_render() - editor.cursor.x_offset + 1;
            let y = editor.cursor.y - editor.cursor.y_offset + 1;
            (x, y)
        }
    }

    fn process_keypress(&self) -> Box<dyn Fn(&mut Editor, Key)> {
        Box::new(
            |editor: &mut Editor, input: Key| {
                match input {
                    Key::Move(key)   => { Normal::move_cursor(editor, key) },
                    Key::Quit        => { editor.quit() },
                    Key::Save        => { editor.save() },
                    Key::Enter       => { Normal::enter_pressed(editor) },
                    Key::Delete      => { Normal::delete_pressed(editor) },
                    Key::BackSpace   => { Normal::backspace_pressed(editor) }
                    Key::Char(c)     => { Normal::write(editor, c as char)}
                    _                => {}
                }
            })
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

    fn set(self) -> Box<dyn FnOnce(&mut Editor)> {
        Box::new(
            |editor: &mut Editor| {
                editor.cursor.restore_x();
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

    fn process_keypress(&self) -> Box<dyn Fn(&mut Editor, Key)> {
        let inner_type = self.0;
        Box::new(
            move |editor: &mut Editor, input: Key| {
                match input {
                    Key::Move(key)   => { Prompt::move_cursor(inner_type, editor, key) },
                    Key::Quit        => { Prompt::quit(editor) },
                    Key::Save        => {  },
                    Key::Enter       => { Prompt::enter_pressed(inner_type, editor) },
                    Key::Delete      => { Prompt::delete_pressed(editor) },
                    Key::BackSpace   => { Prompt::backspace_pressed(inner_type, editor) }
                    Key::Char(c)     => { Prompt::write(inner_type, editor, c as char)}
                    _                => {}
                }
                editor.status_bar.set_init();
            })
    }

    fn scroll(&self) -> fn(&mut Editor) {
        |_: &mut Editor| {}
    }

    fn set(self) -> Box<dyn FnOnce(&mut Editor)> {
        Box::new(
            |editor: &mut Editor| {
                editor.cursor.store_x();
                editor.cursor.x = editor.message().len();
                editor.input_mode = Box::new(self)
            })
    }
}

impl Normal {
    fn enter_pressed(editor: &mut Editor) {
        editor.file.split_line(editor.cursor.y, editor.cursor.x);
        Normal::move_cursor(editor, MoveKey::ArrowRight);
    }

    fn delete_pressed(editor: &mut Editor) {
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

    fn backspace_pressed(editor: &mut Editor) {
        if editor.cursor.x == 0 && editor.cursor.y == 0 {
            return
        }

        if editor.cursor.x == 0 {
            let cursor_y = editor.cursor.y;
            Normal::move_cursor(editor, MoveKey::ArrowLeft);
            editor.file.merge_lines(cursor_y, editor.cursor.y);
            return
        }

        Normal::move_cursor(editor, MoveKey::ArrowLeft);
        editor.file.lines[editor.cursor.y].remove(editor.cursor.x);
        return
    }

    fn write(editor: &mut Editor, c: char) {
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
        Normal::move_cursor(editor, MoveKey::ArrowRight)
    }

    fn move_cursor(editor: &mut Editor, key: MoveKey) {
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

impl Prompt {
    fn quit(editor: &mut Editor) {
        editor.change_input_mode(Normal);
        editor.status_bar.clear()
    }

    fn enter_pressed(inner_type: InnerType, editor: &mut Editor) {
        match inner_type {
            InnerType::Save => {
                let file_name = editor.status_bar.take_prompt();
                editor.file.set_name(&file_name);
                if Path::new(&file_name).is_file() {
                    editor.status_bar.set_message(
                        format!("{} already exists. overwrite? (y/n)", file_name));
                    return editor.change_input_mode(Prompt(InnerType::Overwrite))
                }
                editor.save();
            },
            InnerType::Quit      => {},
            InnerType::Overwrite => {},
        }
    }

    fn delete_pressed(editor: &mut Editor) {
        editor.status_bar.prompt_delete(editor.cursor.x);
    }

    fn backspace_pressed(inner_type: InnerType, editor: &mut Editor) {
        if editor.status_bar.prompt_backspace(editor.cursor.x).is_some() {
            Prompt::move_cursor(inner_type, editor, MoveKey::ArrowLeft)
        }
    }

    fn write(inner_type: InnerType, editor: &mut Editor, c: char) {
        if c == '\t' {
            return
        }
        match inner_type {
            InnerType::Save => {
                editor.status_bar.prompt_insert(c, editor.cursor.x);
                Prompt::move_cursor(inner_type, editor, MoveKey::ArrowRight)
            },
            InnerType::Quit => {
                if c == 'y' || c == 'Y' {
                    editor.end(DieReason::Quit)
                }
                if c == 'n' || c == 'N' {
                    Prompt::quit(editor)
                }
            },
            InnerType::Overwrite => {
                if c == 'y' || c == 'Y' {
                    editor.save();
                }
                if c == 'n' || c == 'N' {
                    editor.file.clear_name();
                    Prompt::quit(editor)
                }
            }
        }
    }

    fn move_cursor(inner_type: InnerType, editor: &mut Editor, key: MoveKey) {
        match inner_type {
            InnerType::Save => {
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
                    MoveKey::ArrowUp   => {},
                    MoveKey::ArrowDown => {},
                    MoveKey::PageUp    => {},
                    MoveKey::PageDown  => {},
                }
            },
            InnerType::Quit      => {},
            InnerType::Overwrite => {},
        }
    }
}
