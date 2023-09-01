pub mod basic;
pub mod bridge;
pub mod username;

use crate::MatrixContext;
use matrix_sdk::room::Joined;
use matrix_sdk::{event_handler::Ctx, ruma::events::room::message::OriginalSyncRoomMessageEvent};

pub struct CommandInput {
    pub event: OriginalSyncRoomMessageEvent,
    pub room: Joined,
    pub matrix_context: Ctx<MatrixContext>,
    pub arguments: Vec<String>,
}
