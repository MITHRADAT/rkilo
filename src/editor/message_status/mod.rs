use std::time;

pub struct MessageStatus {
    msg: String,
    prompt: Vec<char>,
    timeout: time::Duration,
    init: time::SystemTime
}

impl MessageStatus {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: String::from(msg),
            prompt: vec![],
            timeout: MessageStatus::default_timeout(),
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

    pub fn push_prompt(&mut self, c: char) {
        self.prompt.push(c);
        self.timeout = time::Duration::from_mins(1)
    }

    pub fn take_prompt(&mut self) -> String {
        let prompt: String = self.prompt.iter().collect();
        self.prompt.clear();
        self.timeout = MessageStatus::default_timeout();
        prompt
    }

    fn default_timeout() -> time::Duration {
        time::Duration::from_secs(5)
    }

}
