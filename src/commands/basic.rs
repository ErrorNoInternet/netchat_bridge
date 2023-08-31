use super::CommandInput;
use crate::{language::get_text, utilities};

pub async fn ping_command(command_input: &CommandInput) {
    utilities::send_plain_message(&command_input.room, get_text("pong")).await
}
