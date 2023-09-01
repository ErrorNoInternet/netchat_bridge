use super::CommandInput;
use crate::{language::get_text, netchat, permissions::Action, utilities};

pub async fn bridge_command(command_input: &CommandInput) {
    if utilities::handle_permissions(command_input, Action::BridgeCreate).await {
        return;
    };

    if command_input.arguments.len() < 2 {
        utilities::send_plain_message(
            &command_input.room,
            get_text("missing_arguments")
                .replace("{count}", "2")
                .replace("{arguments}", "room_name, room_password")
                .as_str(),
        )
        .await
    }
    let room_name = &command_input.arguments[0];
    let room_password = &command_input.arguments[1];
    utilities::set_typing(&command_input.room, true).await;

    let is_initializing =
        match netchat::is_initializing(room_name.to_string(), room_password.to_string()).await {
            Ok(is_initializing) => is_initializing,
            Err(error) => {
                utilities::send_html_message(
                    &command_input.room,
                    &get_text("fetch_room_failed").replace("{error}", &error.to_string()),
                )
                .await;
                return;
            }
        };
    if is_initializing {
        utilities::send_plain_message(&command_input.room, get_text("room_currently_initializing"))
            .await;
        return;
    }

    //    utilities::send_plain_message(&command_input.room, "Success!").await;
}
