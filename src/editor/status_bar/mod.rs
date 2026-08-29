use std::time;

pub struct StatusBar {
    msg: String,
    prompt: Vec<char>,
    init: time::SystemTime
}

impl StatusBar {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: String::from(msg),
            prompt: vec![],
            init: time::SystemTime::now()
        }
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt.chars().collect();
    }

    pub fn set_message(&mut self, msg: String) {
        self.prompt.clear();
        self.msg = msg;
        self.init = time::SystemTime::now()
    }

    pub fn message(&self) -> Result<String, time::SystemTimeError> {
        if time::SystemTime::now()
            .duration_since(self.init)? < StatusBar::default_timeout() {
                let prompt: String = self.prompt.iter().collect();
                return Ok(format!("{}{}", self.msg, prompt))
            }
        Ok(String::new())
    }

    pub fn prompt_insert(&mut self, c: char, cursor_x: usize) {
        let prompt_index = self.prompt_index(cursor_x);
        self.prompt.insert(prompt_index, c);
    }

    pub fn prompt_delete(&mut self, cursor_x: usize) -> Option<char> {
        let prompt_len = self.prompt.len();
        if prompt_len == 0 {
            return None
        }
        let prompt_index = self.prompt_index(cursor_x);
        if prompt_index < prompt_len {
            return Some(self.prompt.remove(prompt_index))
        }

        return None
    }

    pub fn prompt_backspace(&mut self, cursor_x: usize) -> Option<char> {
        if self.prompt.len() == 0 {
            return None
        }
        let prompt_index = self.prompt_index(cursor_x);
        if prompt_index > 0 {
            return Some(self.prompt.remove(prompt_index - 1))
        }

        return None
    }

    pub fn take_prompt(&mut self) -> String {
        let prompt: String = self.prompt.iter().collect();
        self.prompt.clear();
        prompt
    }

    pub fn prompt_index(&self, cursor_x: usize) -> usize {
        if cursor_x > self.msg.len() {
            return cursor_x - self.msg.len()
        }
        return 0
    }

    pub fn clear(&mut self) {
        self.prompt.clear();
        self.msg.clear();
    }

    pub fn set_init(&mut self) {
        self.init = time::SystemTime::now()
    }

    fn default_timeout() -> time::Duration {
        time::Duration::from_secs(5)
    }

}
