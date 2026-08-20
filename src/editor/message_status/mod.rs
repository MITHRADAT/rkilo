use std::time;

pub struct MessageStatus {
    msg: String,
    timeout: time::Duration,
    init: time::SystemTime
}

impl MessageStatus {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: String::from(msg),
            timeout: time::Duration::from_secs(5),
            init: time::SystemTime::now()
        }
    }

    pub fn set(&mut self, msg: &str) {
        self.msg = String::from(msg);
        self.init = time::SystemTime::now()
    }

    pub fn set_timeout(&mut self, timeout: time::Duration) {
        self.timeout = timeout
    }

    pub fn timeout(&self) -> time::Duration {
        self.timeout
    }

    pub fn message(&self) -> Result<&str, time::SystemTimeError> {
        if time::SystemTime::now()
            .duration_since(self.init)?
            < self.timeout() {
                return Ok(&self.msg)
            }
        Ok("")
    }

}
