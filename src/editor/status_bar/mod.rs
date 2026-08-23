use std::time;
use super::super::common::*;

pub struct StatusBar {
    msg: String,
    prompt: Vec<char>,
    timeout: time::Duration,
    init: time::SystemTime
}

impl StatusBar {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: String::from(msg),
            prompt: vec![],
            timeout: StatusBar::default_timeout(),
            init: time::SystemTime::now()
        }
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt.chars().collect();
        self.init = time::SystemTime::now();
    }

    pub fn set_message(&mut self, msg: String) {
        self.msg = msg;
        self.init = time::SystemTime::now()
    }

    pub fn message(&self) -> Result<String, time::SystemTimeError> {
        if time::SystemTime::now()
            .duration_since(self.init)? < self.timeout {
                let prompt: String = self.prompt.iter().collect();
                return Ok(format!("{}{}", self.msg, prompt))
            }
        Ok(String::new())
    }

    pub fn prompt_insert(&mut self, c: char, cursor_x: usize) {
        let prompt_index = self.prompt_index(cursor_x);
        self.prompt.insert(prompt_index, c);
        self.init = time::SystemTime::now()
    }

    pub fn prompt_remove(&mut self, index: usize) -> Option<char> {
        self.init = time::SystemTime::now();

        if self.prompt.len() == 0 {
            return None
        }

        return Some(self.prompt.remove(index));
    }

    pub fn take_prompt(&mut self) -> String {
        let prompt: String = self.prompt.iter().collect();
        self.prompt.clear();
        self.timeout = StatusBar::default_timeout();
        prompt
    }

    pub fn prompt_index(&self, cursor_x: usize) -> usize {
        if cursor_x > self.msg.len() {
            return cursor_x - self.msg.len()
        }
        return 0
    }

    pub fn prompt_len(&self) -> usize {
        self.prompt.len()
    }

    pub fn set_timeout(&mut self, timeout: time::Duration) {
        self.timeout = timeout
    }

    fn default_timeout() -> time::Duration {
        time::Duration::from_secs(5)
    }

}
