#[derive(Default)]
pub struct Status {
    pub silent: bool,
}

impl Status {
    /// Creates a new `Status` struct for isolating system message output on stderr
    pub fn new() -> Self {
        Self { silent: true }
    }

    /// Clear a partially printed line that has not printed a final newline character
    pub fn clear_line(&self) {
        for _ in 0..100 {
            self.print("\u{8}");
            self.print("\r"); // just to be sure
        }
    }

    /// Print text to standard error
    pub fn print(&self, text: &str) {
        if self.silent {
            return;
        }
        eprint!("{}", text);
    }
}
