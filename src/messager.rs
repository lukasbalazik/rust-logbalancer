pub struct Messager {
    pub penultimate_last_line: String,
    pub complete: bool,
}
impl Messager {
    pub fn corrector(&mut self, received: &str) -> String {
        let mut _message: String = "".to_owned();
        if self.complete != true { 
            _message.push_str(&self.penultimate_last_line);
        }
        _message.push_str(received);

        if _message.chars().last().unwrap() != '\n' {
            self.complete = false;
        } else {
            self.complete = true;
        }

        _message
    }

    pub fn set_penultimate_last_line(&mut self, line: String) {
        self.penultimate_last_line = line;
    }
}
