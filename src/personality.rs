/// A personality that we can customize
pub struct Personality {
    pub mode: Mode,
    pub name: String,
    pub instructions: String, // read from markdown
}

impl Personality {
    /// Create a new personality.
    pub fn new(name: &str, instructions: &str) -> Self {
        Self {
            name: name.to_string(),
            mode: Mode::NonInteractive,
            instructions: instructions.to_string(),
        }
    }

    /// Short message without wrapping
    pub fn speak(&self, message: &str) {
        println!("{}: {}", self.name, message);
    }

    /// Speak with the appropriate name prefix.
    pub fn respond(&self, message: &str) {
        const MAX_CHARS: usize = 80;
        // split the message at whitespace and fold into readable lines
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in message.split_whitespace() {
            if line.chars().count() + word.chars().count() + 1 > MAX_CHARS {
                lines.push(line);
                line = String::new();
            }
            line.push_str(word);
            line.push(' ');
        }
        if line.chars().count() > 0 {
            lines.push(line);
        }
        println!("{}", lines.join("\n"));
    }
}

/// Mode of interaction for the assistant
#[derive(Debug)]
pub enum Mode {
    Interactive,
    NonInteractive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_speak() {
        let personality = Personality::new("Morpha", "");
        personality.respond(include_str!("../data/lorem_ipsum.txt"));
    }
}
