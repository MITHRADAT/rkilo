use std::time;

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

    pub fn set(&mut self, msg: &str, timeout: time::Duration) {
        self.set_message(msg);
        self.timeout = timeout;
        self.init = time::SystemTime::now();
    }

    pub fn set_message(&mut self, msg: &str) {
        self.msg = String::from(msg);
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

    pub fn prompt_insert(&mut self, c: char, index: usize) {
        let prompt_index = index - self.msg.len();
        self.prompt.insert(prompt_index, c);
        self.timeout = time::Duration::from_mins(1)
    }

    pub fn take_prompt(&mut self) -> String {
        let prompt: String = self.prompt.iter().collect();
        self.prompt.clear();
        self.timeout = StatusBar::default_timeout();
        prompt
    }

    fn default_timeout() -> time::Duration {
        time::Duration::from_secs(5)
    }

}
