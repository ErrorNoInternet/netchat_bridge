use crate::logging::{log_message, LogMessageType};
use phf::phf_map;

static TEXTS: phf::Map<&'static str, &'static str> = phf_map! {
    "pong" => "🏓 Pong!",
    "command_no_permissions" => "You do not have the permissions to use this command! This command requires power level <code>{minimum_power_level}</code>.",
    "fetch_permissions_failed" => "Uh oh! An error occurred while fetching your room permissions (<code>{error}</code>). For safety reasons, you have been denied access to use this command.",
    "missing_subcommand" => "This command requires a subcommand! Valid choices are <b>{subcommands}</b>.",
    "missing_arguments" => "You did not supply enough arguments! This command requires at least <b>{count} argument(s)</b> ({arguments}).",
    "database_error" => "Uh oh! Something went wrong while interacting with the database (<code>{error}</code>). Please try again later.",
    "database_possibly_corrupted" => "Uh oh! Something went wrong while processing data from the database (<code>{error}</code>). This issue might be resolved later.",
    "fetch_room_failed" => "Uh oh! An error occurred while fetching that NetChat room (<code>{error}</code>). Please try again later.",
    "room_currently_initializing" => "Seems like that NetChat room is currently being initialized. Please try again in a few minutes.",
    "room_already_bridged" => "Hmm, seems like this room has already been bridged. You can use the \"unbridge\" command to unbridge this room and try again.",
    "room_not_bridged" => "This Matrix room is currently not bridged to any NetChat room.",
    "room_wrong_password" => "Hmm, I can't seem to access that room, maybe the password you supplied is wrong?",
    "room_successfully_bridged" => "This Matrix room has been successfully bridged to <b>{room_name}</b>.",
    "room_successfully_unbridged" => "This Matrix room has been successfully unbridged from <b>{room_name}</b>.",
    "room_status" => "This Matrix room is currently bridged to <b>{room_name}</b> (<b>{room_message_count}</b> messages).",
    "message_bridge_failed" => "Uh oh! Something went wrong while bridging that message (<code>{error}</code>). Please try again later.",
    "username_set_successfully" => "Your NetChat username for this room has been successfully set to <b>{username}</b>.",
    "username_cleared_successfully" => "Your NetChat username for this room has been successfully cleared. Your NetChat messages will now send as your Matrix display name.",
    "username_not_set" => "You do not have a NetChat username for this room.",
    "current_username" => "Your NetChat username for this room is <b>{username}</b>."
};

pub fn get_text(key: &str) -> &str {
    match TEXTS.get(key) {
        Some(value) => value,
        None => {
            log_message(
                LogMessageType::Error,
                &format!("Missing text for \"{key}\"!"),
            );
            key
        }
    }
}
