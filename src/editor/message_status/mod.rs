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
            timeout: time::Duration::from_secs(5),
            init: time::SystemTime::now()
        }
    }

    pub fn set(&mut self, msg: &str, timeout: time::Duration) {
        self.set_message(msg);
        self.set_timeout(timeout)
    }

    pub fn set_message(&mut self, msg: &str) {
        self.msg = String::from(msg);
        self.init = time::SystemTime::now()
    }

    pub fn set_timeout(&mut self, timeout: time::Duration) {
        self.timeout = timeout
    }

    pub fn timeout(&self) -> time::Duration {
        self.timeout
    }

    pub fn message(&self) -> Result<String, time::SystemTimeError> {
        if time::SystemTime::now()
            .duration_since(self.init)?
            < self.timeout() {
                let prompt: String = self.prompt.iter().collect();
                return Ok(format!("{}{}", self.msg, prompt))
            }
        Ok(String::new())
    }

    pub fn set_prompt(&mut self, c: char) {
        self.prompt.push(c);
    }

    pub fn empty_prompt(&mut self) {
        self.prompt.clear()
    }

}
