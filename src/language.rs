use crate::logging::{log_message, LogMessageType};
use phf::phf_map;

static TEXTS: phf::Map<&'static str, &'static str> = phf_map! {
    "pong" => "🏓 Pong!",
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
