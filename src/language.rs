use crate::logging::{log_message, LogMessageType};
use phf::phf_map;

static TEXTS: phf::Map<&'static str, &'static str> = phf_map! {
    "pong" => "🏓 Pong!",
    "command_no_permissions" => "You do not have the permissions to use this command! This command requires power level <code>{minimum_power_level}</code>.",
    "fetch_permissions_failed" => "Uh oh! An error occurred while fetching your room permissions (<code>{error}</code>). For safety reasons, you have been denied access to use this command.",
    "missing_arguments" => "You did not supply enough arguments! The command requires at least {count} arguments ({arguments}).",
    "database_error" => "Uh oh! Something went wrong while querying the database (<code>{error}</code>). Please try again later.",
    "fetch_room_failed" => "Uh oh! An error occurred while fetching that NetChat room (<code>{error}</code>). Please try again later.",
    "room_currently_initializing" => "Seems like that NetChat room is currently being initialized. Please try again in a few minutes.",
    "room_already_bridged" => "Hmm, seems like this room has already been bridged. You can use the unbridge command to unbridge this room and try again.",
    "room_wrong_password" => "Hmm, I can't seem to access that room, maybe the password you supplied is wrong?"
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
