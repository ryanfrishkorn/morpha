#[derive(Default)]
pub struct Status {
    pub silent: bool,
}

impl Status {
    /// Creates a new `Status` struct for isolating system message output on stderr
    pub fn new() -> Self {
        Self { silent: true }
    }

    /// Print text to standard error
    pub fn print(&self, text: &str) {
        if self.silent {
            return;
        }
        eprint!("{}", text);
    }
}
