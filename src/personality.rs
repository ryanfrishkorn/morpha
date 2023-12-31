const MAX_CHARS_DEFAULT: usize = 80;

/// A personality that we can customize
pub struct Personality {
    pub mode: Mode,
    pub name: String,
    pub instructions: String, // read from markdown
    pub max_chars: Option<usize>,
}

impl Personality {
    /// Create a new personality.
    pub fn new(name: &str, instructions: &str) -> Self {
        Self {
            name: name.to_string(),
            mode: Mode::NonInteractive,
            instructions: instructions.to_string(),
            max_chars: Some(MAX_CHARS_DEFAULT),
        }
    }

    /// Extract code blocks to avoid wrapping
    pub fn speak(&self, text: &str) {
        let lines: Vec<&str> = text.lines().collect();
        let mut output: Vec<String> = Vec::new();
        let mut code: Vec<String> = Vec::new();
        let mut speech: Vec<String> = Vec::new();

        let mut in_block = false;
        let mut block: Vec<&str> = Vec::new();
        for line in lines.iter() {
            // determine if this line is a code delimiter
            let delimiter_found = line.starts_with("```");

            match in_block {
                false => {
                    if delimiter_found {
                        // we are ENTERING a code block
                        in_block = true;
                        block.push(line); // delimiter ``` is always pushed to code
                        output.push(line.to_string());
                    } else {
                        speech.push(line.to_string());
                        output.push(
                            // wrap lines based on personality config
                            match self.max_chars {
                                Some(_) => self.split_at_word(line),
                                None => line.to_string(),
                            },
                        );
                    }
                }
                true => {
                    block.push(line);
                    output.push(line.to_string());
                    if delimiter_found {
                        // we are EXITING a code block
                        in_block = false;
                    }
                }
            }

            // push when delimiter is found or on last iteration for final speech
            if delimiter_found && !block.is_empty() {
                code.push(block.join("\n"));
                block.clear();
            }
        }

        /*
        println!(">>>SPEECH");
        println!("{:?}", speech);
        println!(">>>END SPEECH");

        println!(">>>CODE");
        println!("{:?}", code);
        println!(">>>END CODE");

        println!(">>>OUTPUT");
        println!("{}", output.join("\n"));
        println!(">>>END OUTPUT\n");
         */

        println!("{}", output.join("\n"));
    }

    /// Short message without wrapping
    pub fn speak_raw(&self, message: &str) {
        println!("{}: {}", self.name, message);
    }

    /// Split `message` into lines at whitespace honoring `self.max_chars`
    fn split_at_word(&self, message: &str) -> String {
        let max_chars: usize = match self.max_chars {
            Some(v) => v,
            None => return message.to_string(),
        };

        // split the message at whitespace and fold into readable lines
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in message.split_whitespace() {
            if line.chars().count() + word.chars().count() + 1 > max_chars {
                lines.push(line);
                line = String::new();
            }
            line.push_str(word);
            line.push(' ');
        }
        if line.chars().count() > 0 {
            lines.push(line);
        }
        lines.join("\n")
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
        let msg = r#"This is a response that is longer than the expected wrap character limit which is approximately 80.

It also has a code block here:

```
fn main() {
    println!("Hello World!");
}
```

Now, let me explain each line of code:
1. `#include <stdio.h>`: This line includes the standard input/output library, allowing the program to perform input and output operations.

2. `int main(void) {`: This line declares the main function, which is the entry point of the program. It returns an integer, and takes no arguments, as indicated by `void`.
"#;

        let p = Personality::new("name", "You are an assistant");
        p.speak(msg);
    }

    #[test]
    fn test_personality_split_at_word() {
        let personality = Personality::new("Morpha", "");
        let output = personality.split_at_word(include_str!("../data/lorem_ipsum.txt"));
        assert_eq!(6, output.matches('\n').count()); // count newlines
    }
}
