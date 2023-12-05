/// A personality that we can customize
pub struct Personality {
    pub name: String,
    pub instructions: String, // read from markdown
}

impl Personality {
    /// Create a new personality.
    pub fn new(name: &str, instructions: &str) -> Self {
        Self {
            name: name.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_speak() {
        let personality = Personality::new("Morpha", "");
        personality.respond(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum sodales risus ipsum, ac sagittis sem dignissim nec. Etiam non euismod orci. Integer in metus a lacus malesuada placerat eu id nisi. Donec ut volutpat justo. Aenean vehicula imperdiet eros, ac aliquet urna placerat at. Nullam nec mattis nulla. Integer mattis nec nulla nec efficitur. Cras lacinia, ligula ac ullamcorper maximus, nunc mi ultrices nisl, eu dapibus libero nisl sed felis. In risus magna, lobortis in nisl in, vulputate porta nunc. Sed vel dapibus est."
        );
    }
}
