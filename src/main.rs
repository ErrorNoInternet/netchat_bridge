mod commands;
mod configuration;
mod language;
mod logging;
mod secrets;
mod utilities;

use clap::Parser;
use logging::{log_message, LogMessageType::*};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::events::room::{
        member::StrippedRoomMemberEvent,
        message::{MessageType, OriginalSyncRoomMessageEvent},
    },
    Client,
};
use std::path::Path;
use tokio::time::{sleep, Duration};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// The JSON file where the bot's secrets
    /// (for example username & password) reside in.
    #[arg(short, long, default_value = "secrets.json")]
    secrets_file: String,

    /// The JSON file where the bot's settings
    /// are stored (for example command prefix).
    #[arg(short, long, default_value = "configuration.json")]
    configuration_file: String,

    /// Generate a new configuration file with defaults.
    #[arg(short, long)]
    generate_configuration_file: bool,
}

#[derive(Clone)]
pub struct MatrixState {
    configuration: configuration::Configuration,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log_message(
        Bot,
        &format!("Starting netchat_bridge v{}...", env!("CARGO_PKG_VERSION")),
    );

    let arguments = Arguments::parse();
    if arguments.generate_configuration_file {
        match configuration::Configuration::default()
            .to_json_file(Path::new(arguments.configuration_file.as_str()))
        {
            Ok(_) => log_message(
                Bot,
                &format!(
                    "Successfully saved new configuration to {}!",
                    arguments.configuration_file
                ),
            ),
            Err(error) => log_message(
                Error,
                &format!(
                    "Unable to save configuration to {}: {error}",
                    arguments.configuration_file
                ),
            ),
        };
        std::process::exit(0);
    }
    let bot_secrets =
        match secrets::Secrets::from_json_file(Path::new(arguments.secrets_file.as_str())) {
            Ok(bot_secrets) => bot_secrets,
            Err(error) => {
                log_message(
                    Error,
                    &format!(
                        "Unable to parse {} as JSON file: {error}",
                        arguments.secrets_file
                    ),
                );
                std::process::exit(1);
            }
        };
    let bot_configuration = match configuration::Configuration::from_json_file(Path::new(
        arguments.configuration_file.as_str(),
    )) {
        Ok(bot_configuration) => bot_configuration,
        Err(error) => {
            log_message(
                Warning,
                &format!(
                    "Unable to parse {} as JSON file ({error}), using default configuration!",
                    arguments.configuration_file
                ),
            );
            configuration::Configuration::default()
        }
    };

    login_and_sync(
        bot_secrets.homeserver_url,
        &bot_secrets.username,
        &bot_secrets.password,
        MatrixState {
            configuration: bot_configuration,
        },
    )
    .await?;
    Ok(())
}

async fn login_and_sync(
    homeserver_url: String,
    username: &str,
    password: &str,
    matrix_state: MatrixState,
) -> anyhow::Result<()> {
    #[allow(unused_mut)]
    let mut client_builder = Client::builder().homeserver_url(&homeserver_url);
    let client = match client_builder.build().await {
        Ok(client) => client,
        Err(error) => {
            log_message(MatrixError, &format!("Unable to build client: {error}"));
            std::process::exit(1);
        }
    };

    log_message(
        Matrix,
        &format!("Logging in as {} on {}...", &username, &homeserver_url),
    );
    match client
        .login_username(username, password)
        .device_id("NETCHATBRIDGE")
        .initial_device_display_name("NetChat Bridge")
        .send()
        .await
    {
        Ok(_) => (),
        Err(error) => {
            log_message(MatrixError, &format!("Unable to log in: {error}"));
            std::process::exit(1);
        }
    };
    log_message(
        Matrix,
        &format!(
            "Successfully logged in as {} on {}!",
            &username, &homeserver_url
        ),
    );

    client.add_event_handler_context(matrix_state);
    client.add_event_handler(on_stripped_state_member);
    client.sync_once(SyncSettings::default()).await.unwrap();
    client.add_event_handler(on_room_message);
    let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
    client.sync(settings).await?;

    Ok(())
}

async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    if room_member.state_key != client.user_id().unwrap() {
        return;
    }

    if let Room::Invited(room) = room {
        tokio::spawn(async move {
            let mut delay = 2;

            while let Err(error) = room.accept_invitation().await {
                log_message(
                    MatrixWarning,
                    &format!(
                        "Failed to join room {} ({error:?}), retrying in {delay}s...",
                        room.room_id()
                    ),
                );

                sleep(Duration::from_secs(delay)).await;
                delay *= 2;

                if delay > 3600 {
                    log_message(
                        MatrixError,
                        &format!("Unable to join room {}: {error}", room.room_id()),
                    );
                    break;
                }
            }
        });
    }
}

async fn on_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    matrix_state: Ctx<MatrixState>,
) {
    if let Room::Joined(room) = room {
        match event.content.msgtype {
            MessageType::Text(_) => {
                let body = event.content.body();

                if body.starts_with(&matrix_state.configuration.command_prefix) {
                    let mut characters = body.split(" ").nth(0).unwrap().chars();
                    characters.next();
                    let command = characters.as_str();
                    let arguments: Vec<String> = body
                        .split(" ")
                        .skip(1)
                        .map(|item| item.to_string())
                        .collect();

                    let command_input = commands::CommandInput {
                        event: event.clone(),
                        room,
                        matrix_state,
                        arguments,
                    };
                    match command {
                        "ping" => commands::basic::ping_command(&command_input).await,
                        _ => (),
                    };
                }
            }
            _ => (),
        }
    }
}
