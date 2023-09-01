use super::CommandInput;
use crate::{language::get_text, logging::log_error, utilities};

pub async fn username_command(command_input: &CommandInput) {
    if command_input.arguments.len() < 1 {
        utilities::send_html_message(
            &command_input.room,
            get_text("missing_subcommand")
                .replace("{subcommands}", "set/get/clear")
                .as_str(),
        )
        .await;
        return;
    }
    if command_input.arguments[0] == "set" {
        if command_input.arguments.len() < 2 {
            utilities::send_html_message(
                &command_input.room,
                get_text("missing_arguments")
                    .replace("{count}", "1")
                    .replace("{arguments}", "set <name>")
                    .as_str(),
            )
            .await;
            return;
        }
    }

    match command_input.arguments[0].as_str() {
        "set" => {
            match command_input.matrix_context.database.set(
                &format!(
                    "username.{}.{}",
                    command_input.room.room_id().as_str(),
                    command_input.event.sender.as_str()
                ),
                command_input.arguments[1].as_str(),
            ) {
                Ok(_) => (),
                Err(error) => {
                    log_error(&error);
                    utilities::send_html_message(
                        &command_input.room,
                        get_text("database_error")
                            .replace("{error}", &error)
                            .as_str(),
                    )
                    .await;
                    return;
                }
            }
            utilities::send_html_message(
                &command_input.room,
                get_text("username_set_successfully")
                    .replace("{username}", command_input.arguments[1].as_str())
                    .as_str(),
            )
            .await;
        }
        "get" => {
            let username = match command_input.matrix_context.database.get(&format!(
                "username.{}.{}",
                command_input.room.room_id().as_str(),
                command_input.event.sender.as_str()
            )) {
                Ok(username) => match username {
                    Some(username) => username,
                    None => {
                        utilities::send_plain_message(
                            &command_input.room,
                            get_text("username_not_set"),
                        )
                        .await;
                        return;
                    }
                },
                Err(error) => {
                    log_error(&error);
                    utilities::send_html_message(
                        &command_input.room,
                        get_text("database_error")
                            .replace("{error}", &error)
                            .as_str(),
                    )
                    .await;
                    return;
                }
            };
            utilities::send_html_message(
                &command_input.room,
                get_text("current_username")
                    .replace("{username}", &username)
                    .as_str(),
            )
            .await;
        }
        "clear" => {
            match command_input.matrix_context.database.remove(&format!(
                "username.{}.{}",
                command_input.room.room_id().as_str(),
                command_input.event.sender.as_str()
            )) {
                Ok(_) => (),
                Err(error) => {
                    log_error(&error);
                    utilities::send_html_message(
                        &command_input.room,
                        get_text("database_error")
                            .replace("{error}", &error)
                            .as_str(),
                    )
                    .await;
                    return;
                }
            }
            utilities::send_plain_message(
                &command_input.room,
                get_text("username_cleared_successfully"),
            )
            .await;
        }
        _ => (),
    }
}
