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

#[derive(Parser)]
#[command(about = "A ChatGPT assistant with SQLite archiving of conversations")]
#[command(author, version = env!("CARGO_PKG_VERSION"), before_help = env!("GIT_HASH"))]
#[command(
    help_template = "{name} version: {version} \n{author} \n{about-section}\nbuild: {before-help}{usage-heading} {usage} \n{all-args} {tab}"
)]
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
    personality.speak("How may I assist you?");
    println!(); // I like readability

    // MAIN LOOP
    let mut first_run = true;
    'main: loop {
        // show data prompt read user input
        eprint!("> ");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        if input.is_empty() {
            continue;
        }

        input = input.trim().to_string();
        match input.as_str() {
            "" => {
                // ignore empty line and print exit instructions
                println!("q/quit/exit to leave application\n");
                continue;
            }
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
                    println!(); // reset cursor after progress output
                    println!(); // newline for readability
                    personality.respond(&text);
                    println!(); // I really like readability

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
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        first_run = false;
    }

    // remove assistant and threads
    client.assistants().delete(&assistant_id).await?;
    client.threads().delete(&thread.id).await?;

    Ok(())
}
