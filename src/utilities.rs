use crate::{
    commands::CommandInput,
    language::get_text,
    logging::{log_error, log_matrix_error, log_message},
    permissions::{self, Action},
};
use matrix_sdk::{room, ruma::events::room::message::RoomMessageEventContent};

fn has_html(string: &str) -> bool {
    let html_tags = ["b", "code"];
    for tag in html_tags {
        if string.contains(&format!("<{tag}>")) && string.contains(&format!("</{tag}>")) {
            return true;
        }
    }
    false
}

pub async fn handle_permissions(command_input: &CommandInput, action: Action) -> bool {
    if !match permissions::is_allowed(&command_input, Action::BridgeCreate).await {
        Ok(is_allowed) => is_allowed,
        Err(error) => {
            log_error(&error);
            send_plain_message(
                &command_input.room,
                &get_text("fetch_permissions_failed").replace("{error}", &error.to_string()),
            )
            .await;
            true
        }
    } {
        send_plain_message(
            &command_input.room,
            &get_text("command_no_permissions").replace(
                "{minimum_level}",
                permissions::get_power_level_constraint(action)
                    .minimum
                    .to_string()
                    .as_str(),
            ),
        )
        .await;
        return true;
    }
    false
}

pub async fn set_typing(room: &room::Joined, typing: bool) {
    log_matrix_error(room.typing_notice(typing).await);
}

pub async fn send_plain_message(room: &room::Joined, content: &str) {
    if has_html(&content) {
        log_message(
            crate::logging::LogMessageType::Warning,
            &format!("HTML not used but sent formatted message ({content})",),
        );
    }

    log_matrix_error(
        room.send(RoomMessageEventContent::text_plain(content), None)
            .await,
    );
    set_typing(room, false).await;
}

pub async fn send_html_message(room: &room::Joined, content: &str) {
    if !has_html(&content) {
        log_message(
            crate::logging::LogMessageType::Warning,
            &format!("HTML not used but sent formatted message ({content})",),
        );
    }

    log_matrix_error(
        room.send(RoomMessageEventContent::text_html(content, content), None)
            .await,
    );
    set_typing(room, false).await;
}
