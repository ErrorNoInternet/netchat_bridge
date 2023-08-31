pub mod basic;

use crate::MatrixState;
use matrix_sdk::room::Joined;
use matrix_sdk::{event_handler::Ctx, ruma::events::room::message::OriginalSyncRoomMessageEvent};

pub struct CommandInput {
    pub event: OriginalSyncRoomMessageEvent,
    pub room: Joined,
    pub matrix_state: Ctx<MatrixState>,
    pub arguments: Vec<String>,
}
