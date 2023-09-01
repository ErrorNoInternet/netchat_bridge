use super::CommandInput;
use crate::{language::get_text, logging::log_error, netchat, permissions::Action, utilities};

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
        .await;
        return;
    }
    let room_name = &command_input.arguments[0];
    let room_password = &command_input.arguments[1];

    match command_input
        .matrix_state
        .database
        .get(&format!("bridge.{}", command_input.room.room_id().as_str()))
    {
        Ok(value) => match value {
            Some(_value) => {
                utilities::send_plain_message(
                    &command_input.room,
                    get_text("room_already_bridged"),
                )
                .await;
                return;
            }
            None => (),
        },
        Err(error) => {
            log_error(&error);
            utilities::send_plain_message(
                &command_input.room,
                get_text("database_error")
                    .replace("{error}", &error)
                    .as_str(),
            )
            .await;
            return;
        }
    }

    utilities::set_typing(&command_input.room, true).await;
    match netchat::is_initializing(room_name, room_password).await {
        Ok(is_initializing) => {
            if is_initializing {
                utilities::send_plain_message(
                    &command_input.room,
                    get_text("room_currently_initializing"),
                )
                .await;
                return;
            }
        }
        Err(error) => {
            log_error(&error);
            utilities::send_html_message(
                &command_input.room,
                &get_text("fetch_room_failed").replace("{error}", &error.to_string()),
            )
            .await;
            return;
        }
    };

    match netchat::is_correct_password(room_name, room_password).await {
        Ok(is_correct_password) => {
            if !is_correct_password {
                utilities::send_plain_message(&command_input.room, get_text("room_wrong_password"))
                    .await;
                return;
            }
        }
        Err(error) => {
            log_error(&error);
            utilities::send_html_message(
                &command_input.room,
                &get_text("fetch_room_failed").replace("{error}", &error.to_string()),
            )
            .await;
            return;
        }
    };

    //    utilities::send_plain_message(&command_input.room, "all checks passed!").await;
}
