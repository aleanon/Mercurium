use crate::message::Message;

use super::SetupMessage;




#[derive(Debug, Clone)]
pub enum RestoreWalletMessage {
    FromCloud,
    FromFile,
}


impl Into<Message> for RestoreWalletMessage {
    fn into(self) -> Message {
        Message::Setup(
            SetupMessage::RestoreWalletMessage(self)
        )
    }
}