use async_openai::{
    types::{
        CreateAssistantRequestArgs, CreateMessageRequestArgs, CreateRunRequestArgs,
        CreateThreadRequestArgs, MessageContent, RunStatus,
    },
    Client,
};
use std::error::Error;
use std::io::stdin;

/// A personality that we can customize
struct Personality {
    name: String,
    instructions: String, // read from markdown
}

impl Personality {
    /// Create a new personality.
    fn new(name: &str, instructions: &str) -> Self {
        Self {
            name: name.to_string(),
            instructions: instructions.to_string(),
        }
    }

    /// Short message without wrapping
    fn speak(&self, message: &str) {
        println!("{}: {}", self.name, message);
    }

    /// Speak with the appropriate name prefix.
    fn respond(&self, message: &str) {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect::<Vec<_>>();

    let home = std::env::var("HOME")?;
    let mut personality_profile_path = format!("{}/.morpha_profile", home);
    // read personality data from file
    if let Some(path) = args.get(1) {
        personality_profile_path = path.clone();
    }
    let personality_profile_path = personality_profile_path; // freezing is good

    let personality_profile = std::fs::read_to_string(personality_profile_path)?;
    let personality = Personality::new("Morpha", &personality_profile);

    let client = Client::new();
    let query = [("limit", "1")]; //limit the list responses to 1 message
    let thread_request = CreateThreadRequestArgs::default().build()?;
    let thread = client.threads().create(thread_request).await?;

    let assistant_request = CreateAssistantRequestArgs::default()
        .name(&personality.name)
        .instructions(&personality.instructions)
        .model("gpt-3.5-turbo-1106")
        .build()?;

    let assistant = client.assistants().create(assistant_request).await?;
    let assistant_id = assistant.id;

    // Initial greeting
    personality.speak("How may I assist you?");
    println!(); // I like readability

    // MAIN LOOP
    loop {
        // show data prompt read user input
        eprint!("> ");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        if input.is_empty() {
            continue;
        }

        input = input.trim().to_string();
        match input.as_str() {
            "" => continue, // ignore empty line
            "q" => break,
            "quit" => break,
            "exit" => break,
            _ => (),
        }
        println!(); // I like readability

        //create a message for the thread
        let message = CreateMessageRequestArgs::default()
            .role("user")
            .content(input.clone())
            .build()?;

        //attach message to the thread
        let _message_obj = client
            .threads()
            .messages(&thread.id)
            .create(message)
            .await?;

        //create a run for the thread
        let run_request = CreateRunRequestArgs::default()
            .assistant_id(&assistant_id)
            .build()?;
        let run = client
            .threads()
            .runs(&thread.id)
            .create(run_request)
            .await?;

        //wait for the run to complete
        let mut awaiting_response = true;
        let mut status_printed = false;
        while awaiting_response {
            //retrieve the run
            let run = client.threads().runs(&thread.id).retrieve(&run.id).await?;
            //check the status of the run
            match run.status {
                RunStatus::Completed => {
                    awaiting_response = false;
                    // once the run is completed we
                    // get the response from the run
                    // which will be the first message
                    // in the thread

                    //retrieve the response from the run
                    let response = client.threads().messages(&thread.id).list(&query).await?;
                    //get the message id from the response
                    let message_id = response.data.get(0).unwrap().id.clone();
                    //get the message from the response
                    let message = client
                        .threads()
                        .messages(&thread.id)
                        .retrieve(&message_id)
                        .await?;
                    //get the content from the message
                    let content = message.content.get(0).unwrap();
                    //get the text from the content
                    let text = match content {
                        MessageContent::Text(text) => text.text.value.clone(),
                        MessageContent::ImageFile(_) => {
                            panic!("imaged are not supported in the terminal")
                        }
                    };
                    // print the response
                    println!(); // reset cursor after progress output
                    println!(); // start at first column
                    personality.respond(&text);
                    println!(); // I like readability
                }
                RunStatus::Failed => {
                    awaiting_response = false;
                    eprintln!("--- Run Failed: {:#?}", run);
                }
                RunStatus::Queued => {
                    eprintln!("--- Run Queued");
                }
                RunStatus::Cancelling => {
                    eprintln!("--- Run Cancelling");
                }
                RunStatus::Cancelled => {
                    eprintln!("--- Run Cancelled");
                }
                RunStatus::Expired => {
                    eprintln!("--- Run Expired");
                }
                RunStatus::RequiresAction => {
                    eprintln!("--- Run Requires Action");
                }
                RunStatus::InProgress => {
                    if !status_printed {
                        eprint!("--- Waiting for response...");
                        status_printed = true;
                    } else {
                        eprint!(".");
                    }
                }
            }
            //wait for 1 second before checking the status again
            // std::thread::sleep(std::time::Duration::from_secs(1));
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    // remove assistant and threads
    client.assistants().delete(&assistant_id).await?;
    client.threads().delete(&thread.id).await?;

    Ok(())
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
