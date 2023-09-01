use super::CommandInput;
use crate::{
    language::get_text, logging::log_error, netchat, permissions::Action, utilities,
    BridgedRoomData,
};

pub async fn bridge_command(command_input: &CommandInput) {
    if command_input.arguments.len() < 1 {
        utilities::send_html_message(
            &command_input.room,
            get_text("missing_subcommand")
                .replace("{subcommands}", "create/destroy/status")
                .as_str(),
        )
        .await;
        return;
    }
    if command_input.arguments[0] == "create" {
        if command_input.arguments.len() < 3 {
            utilities::send_html_message(
                &command_input.room,
                get_text("missing_arguments")
                    .replace("{count}", "2")
                    .replace("{arguments}", "create <room_name> <room_password>")
                    .as_str(),
            )
            .await;
            return;
        }
    }

    match command_input.arguments[0].as_str() {
        "create" => {
            if utilities::handle_permissions(command_input, Action::BridgeCreate).await {
                return;
            };

            let room_name = &command_input.arguments[1];
            let room_password = &command_input.arguments[2];
            match command_input
                .matrix_context
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

            utilities::set_typing(&command_input.room, true).await;
            match netchat::is_initializing(
                &command_input.matrix_context.bot_configuration,
                room_name,
                room_password,
            )
            .await
            {
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
            match netchat::is_correct_password(
                &command_input.matrix_context.bot_configuration,
                room_name,
                room_password,
            )
            .await
            {
                Ok(is_correct_password) => {
                    if !is_correct_password {
                        utilities::send_plain_message(
                            &command_input.room,
                            get_text("room_wrong_password"),
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
            let message_count = match netchat::message_count(
                &command_input.matrix_context.bot_configuration,
                room_name,
                room_password,
            )
            .await
            {
                Ok(message_count) => message_count,
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

            match command_input.matrix_context.database.set(
                &format!("bridge.{}", command_input.room.room_id().as_str()),
                serde_json::to_string(&BridgedRoomData {
                    room_name: room_name.to_string(),
                    room_password: room_password.to_string(),
                    message_count,
                })
                .unwrap()
                .as_str(),
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
            };
            utilities::send_html_message(
                &command_input.room,
                get_text("room_successfully_bridged")
                    .replace("{room_name}", &room_name)
                    .as_str(),
            )
            .await;
        }
        "destroy" => {
            if utilities::handle_permissions(command_input, Action::BridgeDestroy).await {
                return;
            };

            let room_name = match command_input
                .matrix_context
                .database
                .get(&format!("bridge.{}", command_input.room.room_id().as_str()))
            {
                Ok(value) => match value {
                    Some(value) => match serde_json::from_str::<BridgedRoomData>(value.as_str()) {
                        Ok(bridged_room_data) => bridged_room_data.room_name,
                        Err(error) => {
                            log_error(&error);
                            utilities::send_html_message(
                                &command_input.room,
                                get_text("database_possibly_corrupted")
                                    .replace("{error}", &error.to_string())
                                    .as_str(),
                            )
                            .await;
                            return;
                        }
                    },
                    None => {
                        utilities::send_plain_message(
                            &command_input.room,
                            get_text("room_not_bridged"),
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

            match command_input
                .matrix_context
                .database
                .remove(&format!("bridge.{}", command_input.room.room_id().as_str()))
            {
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
            };
            utilities::send_html_message(
                &command_input.room,
                get_text("room_successfully_unbridged")
                    .replace("{room_name}", &room_name)
                    .as_str(),
            )
            .await;
        }
        "status" | "info" | "information" => {
            let bridged_room_data = match command_input
                .matrix_context
                .database
                .get(&format!("bridge.{}", command_input.room.room_id().as_str()))
            {
                Ok(value) => match value {
                    Some(value) => match serde_json::from_str::<BridgedRoomData>(value.as_str()) {
                        Ok(bridged_room_data) => bridged_room_data,
                        Err(error) => {
                            log_error(&error);
                            utilities::send_html_message(
                                &command_input.room,
                                get_text("database_possibly_corrupted")
                                    .replace("{error}", &error.to_string())
                                    .as_str(),
                            )
                            .await;
                            return;
                        }
                    },
                    None => {
                        utilities::send_plain_message(
                            &command_input.room,
                            get_text("room_not_bridged"),
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
                get_text("room_status")
                    .replace("{room_name}", &bridged_room_data.room_name)
                    .replace(
                        "{room_message_count}",
                        &bridged_room_data.message_count.to_string(),
                    )
                    .as_str(),
            )
            .await;
        }
        _ => (),
    }
}
