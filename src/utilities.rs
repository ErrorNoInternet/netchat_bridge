use crate::logging::log_matrix_error;
use matrix_sdk::{room, ruma::events::room::message::RoomMessageEventContent};

pub async fn send_plain_message(room: &room::Joined, content: &str) {
    log_matrix_error(
        room.send(RoomMessageEventContent::text_plain(content), None)
            .await,
    );
}
