use morpha::conversation::{Conversation, Message};
use morpha::database;
use morpha::personality::Personality;

use async_openai::{
    types::{
        CreateAssistantRequestArgs, CreateMessageRequestArgs, CreateRunRequestArgs,
        CreateThreadRequestArgs, MessageContent, RunStatus,
    },
    Client,
};
use clap::Parser;
use std::error::Error;
use std::io::stdin;

const CLAP_HELP: &str = r#"{name} version: {version}
{author}
{about-section}
build: {before-help}{usage-heading} {usage}
{all-args} {tab}"#;

#[derive(Parser)]
#[command(about = "A ChatGPT assistant with SQLite archiving of conversations")]
#[command(author, version = env!("CARGO_PKG_VERSION"), before_help = env!("GIT_HASH"))]
#[command(help_template = CLAP_HELP)]
struct Config {
    /// OpenAI model name
    #[arg(long, default_value = "gpt-3.5-turbo-1106")]
    model: String,
    /// SQLite database path
    #[arg(long, default_value = "")]
    db_path: String,
    /// One response only, exits after first response.
    #[arg(long, default_value_t = false)]
    one: bool,
    #[arg(long, default_value = "")]
    profile: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut config = Config::parse();

    let home = std::env::var("HOME")?;
    if config.db_path.is_empty() {
        config.db_path = format!("{}/.morpha.sqlite3", home);
    }
    if config.profile.is_empty() {
        config.profile = format!("{}/.morpha_profile", home);
    }
    let personality_profile = std::fs::read_to_string(config.profile)?;
    let personality = Personality::new("Morpha", &personality_profile);
    let mut status = Status::new();
    // suppress status messages for a singular answer
    if config.one {
        status.silent = true;
    }

    let client = Client::new();
    let query = [("limit", "1")]; //limit the list responses to 1 message
    let thread_request = CreateThreadRequestArgs::default().build()?;
    let thread = client.threads().create(thread_request).await?;

    let assistant_request = CreateAssistantRequestArgs::default()
        .name(&personality.name)
        .instructions(&personality.instructions)
        .model(&config.model)
        .build()?;

    let assistant = client.assistants().create(assistant_request).await?;
    let assistant_id = assistant.id;

    // Open database
    let db = database::open_database(&config.db_path)?;

    // Create conversation and write to database
    let conversation = Conversation {
        id: assistant_id.clone(),
        messages: Vec::new(),
        msec: database::current_msec(),
    };

    // Initial greeting
    if !config.one {
        personality.speak("How may I assist you?");
        status.print("\n");
    }

    // MAIN LOOP
    let mut first_run = true;
    'main: loop {
        // show data prompt read user input
        status.print("> ");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        if input.is_empty() {
            continue;
        }

        input = input.trim().to_string();
        match input.as_str() {
            "" => {
                // ignore empty line and print exit instructions
                status.print("q/quit/exit to leave application\n");
                continue;
            }
            "q" => break,
            "quit" => break,
            "exit" => break,
            _ => (),
        }
        status.print("\n"); // I like readability

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

                    let response = client.threads().messages(&thread.id).list(&query).await?;
                    let message_id = response.data.get(0).unwrap().id.clone();
                    // get the message, content from the response
                    let message = client
                        .threads()
                        .messages(&thread.id)
                        .retrieve(&message_id)
                        .await?;
                    let content = message.content.get(0).unwrap();
                    let text = match content {
                        MessageContent::Text(text) => text.text.value.clone(),
                        MessageContent::ImageFile(_) => {
                            panic!("images are not supported in the terminal")
                        }
                    };

                    // print the response
                    status.print("\n"); // reset cursor after progress output
                    status.print("\n"); // newline for readability
                    personality.respond(&text);
                    status.print("\n"); // I really like readability

                    // Write the conversation only after valid input and response has been obtained.
                    // Otherwise, we will have empty conversations when user input is cancelled.
                    if first_run {
                        conversation.write_to_database(&db)?;
                    }
                    // Write the prompt and response to database
                    let msg = Message {
                        conversation_id: conversation.id.clone(),
                        msec: database::current_msec(),
                        prompt: input.clone(),
                        response: text.clone(),
                    };
                    msg.write_to_database(&db)?;

                    // Exit if one response is requested
                    if config.one {
                        break 'main;
                    }
                }
                RunStatus::Failed => {
                    awaiting_response = false;
                    status.print(&format!("--- Run Failed: {:#?}", run));
                }
                RunStatus::Queued => {
                    status.print("--- Run Queued");
                }
                RunStatus::Cancelling => {
                    status.print("--- Run Cancelling");
                }
                RunStatus::Cancelled => {
                    status.print("--- Run Cancelled");
                }
                RunStatus::Expired => {
                    status.print("--- Run Expired");
                }
                RunStatus::RequiresAction => {
                    status.print("--- Run Requires Action");
                }
                RunStatus::InProgress => {
                    if !status_printed {
                        status.print("--- Waiting for response...");
                        status_printed = true;
                    } else {
                        status.print(".");
                    }
                }
            }
            //wait for 1 second before checking the status again
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        first_run = false;
    }

    // remove assistant and threads
    client.assistants().delete(&assistant_id).await?;
    client.threads().delete(&thread.id).await?;

    Ok(())
}

struct Status {
    pub silent: bool,
}

impl Status {
    /// Creates a new `Status` struct for isolating system message output on stderr
    pub fn new() -> Self {
        Self { silent: false }
    }

    /// Print text to standard error
    pub fn print(&self, text: &str) {
        if self.silent {
            return;
        }
        eprint!("{}", text);
    }
}
