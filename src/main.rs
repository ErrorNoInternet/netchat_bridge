mod commands;
mod configuration;
mod database;
mod language;
mod logging;
mod netchat;
mod permissions;
mod secrets;
mod utilities;

use clap::Parser;
use configuration::Configuration;
use database::Database;
use logging::{log_error, log_matrix_error, log_message, LogMessageType::*};
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
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
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

    /// The path to the database (automatically
    /// created if it doesn't exist)
    #[arg(short, long, default_value = "netchat_bridge.db")]
    database_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BridgedRoomData {
    room_name: String,
    room_password: String,
    message_count: usize,
}

pub struct NetChatBridgeMessage {
    content: String,
    matrix_room_id: String,
}

pub struct MatrixBridgeMessage {
    netchat_room_name: String,
    netchat_room_password: String,
    netchat_username: String,
    netchat_message: String,
}

#[derive(Clone)]
pub struct MatrixContext {
    bot_configuration: configuration::Configuration,
    database: Database,
    matrix_queue_sender: Arc<Mutex<mpsc::Sender<MatrixBridgeMessage>>>,
}

async fn receive_netchat_messages(
    netchat_queue_sender: mpsc::Sender<NetChatBridgeMessage>,
    bot_configuration: &Configuration,
    database: Database,
) {
    log_message(
        Bridge,
        &format!("Running NetChat receiver thread! Waiting for messages from NetChat..."),
    );

    loop {
        for (key, value) in database.iter() {
            if key.starts_with("bridge.") {
                let mut bridged_room_data =
                    match serde_json::from_str::<BridgedRoomData>(value.as_str()) {
                        Ok(bridged_room_data) => bridged_room_data,
                        Err(error) => {
                            log_error(error);
                            continue;
                        }
                    };
                let message_count = match netchat::message_count(
                    &bot_configuration,
                    &bridged_room_data.room_name,
                    &bridged_room_data.room_password,
                )
                .await
                {
                    Ok(message_count) => message_count,
                    Err(error) => {
                        log_error(error);
                        continue;
                    }
                };
                if bridged_room_data.message_count > message_count {
                    bridged_room_data.message_count = message_count;
                    continue;
                }
                if message_count > bridged_room_data.message_count {
                    let room_messages = match netchat::get_room_messages(
                        &bot_configuration,
                        &bridged_room_data.room_name,
                        &bridged_room_data.room_password,
                    )
                    .await
                    {
                        Ok(room_messages) => room_messages,
                        Err(error) => {
                            log_error(error);
                            continue;
                        }
                    };
                    for message in &room_messages[bridged_room_data.message_count..] {
                        let mut processed_message = message.to_string();
                        processed_message.insert_str("[1970-01-01 00:00:00]".len(), "</b>");
                        processed_message.insert_str(0, "<b>");
                        netchat_queue_sender
                            .send(NetChatBridgeMessage {
                                content: processed_message,
                                matrix_room_id: key["bridge.".len()..].to_string(),
                            })
                            .unwrap();
                    }

                    bridged_room_data.message_count = message_count;
                    match database.set(
                        &key,
                        serde_json::to_string(&bridged_room_data).unwrap().as_str(),
                    ) {
                        Ok(_) => (),
                        Err(error) => {
                            log_error(error);
                            continue;
                        }
                    };
                }
            };
        }
        thread::sleep(std::time::Duration::from_secs(
            bot_configuration.refresh_interval,
        ));
    }
}

async fn bridge_netchat_messages(
    netchat_queue_receiver: mpsc::Receiver<NetChatBridgeMessage>,
    client: Client,
) {
    log_message(
        Bridge,
        &format!(
            "Running NetChat -> Matrix thread! Waiting for messages from the NetChat receiver..."
        ),
    );

    loop {
        let bridge_message = netchat_queue_receiver.recv().unwrap();
        match client
            .joined_rooms()
            .iter()
            .find(|item| item.room_id().as_str() == bridge_message.matrix_room_id)
        {
            Some(joined_room) => {
                utilities::send_html_message(&joined_room, &bridge_message.content).await
            }
            None => (),
        }
    }
}

async fn bridge_matrix_messages(
    matrix_queue_receiver: mpsc::Receiver<MatrixBridgeMessage>,
    bot_configuration: &Configuration,
) {
    log_message(
        Bridge,
        &format!(
            "Running Matrix -> NetChat thread! Waiting for messages from the on_room_message event..."
        ),
    );

    loop {
        let bridge_message = matrix_queue_receiver.recv().unwrap();
        match netchat::send_message(
            &bot_configuration,
            &bridge_message.netchat_room_name,
            &bridge_message.netchat_room_password,
            &bridge_message.netchat_username,
            &bridge_message.netchat_message,
        )
        .await
        {
            Ok(_) => (),
            Err(error) => {
                log_error(error);
            }
        }
    }
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

    let database = match Database::new(&arguments.database_path) {
        Ok(database) => database,
        Err(error) => {
            log_message(Error, &format!("Unable to open database: {error}"));
            std::process::exit(1);
        }
    };
    let (netchat_tx, netchat_rx): (Sender<NetChatBridgeMessage>, Receiver<NetChatBridgeMessage>) =
        mpsc::channel();
    let (matrix_tx, matrix_rx): (Sender<MatrixBridgeMessage>, Receiver<MatrixBridgeMessage>) =
        mpsc::channel();
    login_and_sync(
        bot_secrets.homeserver_url,
        &bot_secrets.username,
        &bot_secrets.password,
        MatrixContext {
            bot_configuration,
            database,
            matrix_queue_sender: Arc::new(Mutex::new(matrix_tx)),
        },
        netchat_tx,
        netchat_rx,
        matrix_rx,
    )
    .await?;
    Ok(())
}

async fn login_and_sync(
    homeserver_url: String,
    username: &str,
    password: &str,
    matrix_context: MatrixContext,
    netchat_queue_sender: mpsc::Sender<NetChatBridgeMessage>,
    netchat_queue_receiver: mpsc::Receiver<NetChatBridgeMessage>,
    matrix_queue_receiver: mpsc::Receiver<MatrixBridgeMessage>,
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

    let thread_database = matrix_context.database.clone();
    let thread_bot_configuration = matrix_context.bot_configuration.clone();
    tokio::spawn(async move {
        receive_netchat_messages(
            netchat_queue_sender,
            &thread_bot_configuration,
            thread_database,
        )
        .await
    });
    let thread_client = client.clone();
    tokio::spawn(async { bridge_netchat_messages(netchat_queue_receiver, thread_client).await });
    let thread_bot_configuration = matrix_context.bot_configuration.clone();
    tokio::spawn(async move {
        bridge_matrix_messages(matrix_queue_receiver, &thread_bot_configuration).await
    });
    log_message(Bridge, "All threads have been spawned!");

    client.add_event_handler_context(matrix_context);
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
    matrix_context: Ctx<MatrixContext>,
) {
    if event.sender == room.client().user_id().unwrap() {
        return;
    }

    if let Room::Joined(room) = room {
        match event.content.msgtype {
            MessageType::Text(_) => {
                let body = event.content.body();
                if body.starts_with(&matrix_context.bot_configuration.command_prefix) {
                    let mut characters = body.chars();
                    characters.next();
                    let command = characters.as_str().split(" ").nth(0).unwrap();
                    let mut arguments = Vec::new();
                    let mut current_argument = String::new();
                    let mut in_string = (false, "");
                    for letter in characters {
                        if letter == '\\' {
                            in_string = (true, "\\");
                            continue;
                        }
                        if letter == ' ' && !in_string.0 {
                            if current_argument.len() > 0 {
                                arguments.push(current_argument);
                                current_argument = String::new();
                            }
                            continue;
                        }
                        if letter == '"' && !in_string.0 {
                            in_string = (true, "\"");
                            if current_argument.len() > 0 {
                                arguments.push(current_argument);
                                current_argument = String::new();
                            }
                            continue;
                        } else if letter == '"' && in_string.0 {
                            in_string = (false, "");
                            if current_argument.len() > 0 {
                                arguments.push(current_argument);
                                current_argument = String::new();
                            }
                            continue;
                        }
                        current_argument.push(letter);
                        if in_string == (true, "\\") {
                            in_string = (false, "");
                        }
                    }
                    if current_argument.len() > 0 {
                        arguments.push(current_argument);
                    }
                    arguments.remove(0);

                    let command_input = commands::CommandInput {
                        event: event.clone(),
                        room,
                        matrix_context,
                        arguments,
                    };
                    match command {
                        "ping" => commands::basic::ping_command(&command_input).await,
                        "bridge" => commands::bridge::bridge_command(&command_input).await,
                        "username" => commands::username::username_command(&command_input).await,
                        _ => (),
                    };
                } else {
                    match matrix_context
                        .database
                        .get(&format!("bridge.{}", room.room_id().as_str()))
                    {
                        Ok(value) => match value {
                            Some(value) => {
                                match serde_json::from_str::<BridgedRoomData>(value.as_str()) {
                                    Ok(bridged_room_data) => {
                                        let mut bridged_room_data = bridged_room_data;
                                        let mut custom_username = false;
                                        let mut netchat_username =
                                            match matrix_context.database.get(&format!(
                                                "username.{}.{}",
                                                room.room_id().as_str(),
                                                event.sender.as_str()
                                            )) {
                                                Ok(netchat_username) => match netchat_username {
                                                    Some(netchat_username) => {
                                                        custom_username = true;
                                                        netchat_username
                                                    }
                                                    None => "".to_string(),
                                                },
                                                Err(error) => {
                                                    log_error(error);
                                                    "".to_string()
                                                }
                                            };
                                        if !custom_username {
                                            netchat_username = match room
                                                .get_member(&event.sender)
                                                .await
                                            {
                                                Ok(member) => match member {
                                                    Some(member) => match member.display_name() {
                                                        Some(display_name) => {
                                                            display_name.to_string()
                                                        }
                                                        None => event.sender.as_str().to_string(),
                                                    },
                                                    None => event.sender.as_str().to_string(),
                                                },
                                                Err(error) => {
                                                    log_error(error);
                                                    event.sender.as_str().to_string()
                                                }
                                            }
                                        };
                                        matrix_context
                                            .matrix_queue_sender
                                            .lock()
                                            .unwrap()
                                            .send(MatrixBridgeMessage {
                                                netchat_room_name: bridged_room_data
                                                    .clone()
                                                    .room_name,
                                                netchat_room_password: bridged_room_data
                                                    .clone()
                                                    .room_password,
                                                netchat_username: netchat_username.to_string(),
                                                netchat_message: body.to_string(),
                                            })
                                            .unwrap();
                                        bridged_room_data.message_count += 1;
                                        match matrix_context.database.set(
                                            &format!("bridge.{}", room.room_id().as_str()),
                                            serde_json::to_string(&bridged_room_data)
                                                .unwrap()
                                                .as_str(),
                                        ) {
                                            Ok(_) => (),
                                            Err(error) => {
                                                log_error(error);
                                            }
                                        };
                                        log_matrix_error(room.read_receipt(&event.event_id).await);
                                    }
                                    Err(error) => {
                                        log_error(&error);
                                    }
                                }
                            }
                            None => (),
                        },
                        Err(error) => {
                            log_error(&error);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
