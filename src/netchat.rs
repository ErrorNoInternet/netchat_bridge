use crate::configuration::Configuration;

const NETCHAT_INSTANCE: &str = "https://netchat.repl.co";

async fn make_request(
    bot_configuration: &Configuration,
    url: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(
            bot_configuration.request_timeout,
        ))
        .user_agent(format!(
            "netchat_bridge/{} (url:{url}) reqwest",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .unwrap();
    client.get(url).send().await
}

pub async fn get_room(
    bot_configuration: &Configuration,
    name: &str,
    password: &str,
) -> Result<String, String> {
    match make_request(
        &bot_configuration,
        &format!("{NETCHAT_INSTANCE}/{password}/{name}/allMessages"),
    )
    .await
    {
        Ok(response) => {
            if response.status().is_server_error() {
                Err(format!("encountered server error"))
            } else if response.status() == 429 {
                Err(format!("encountered ratelimit"))
            } else if response.status() == 401 {
                Err(format!("unauthorized"))
            } else {
                match response.text().await {
                    Ok(text) => Ok(text),
                    Err(error) => Err(format!("failed to get response text: {error}")),
                }
            }
        }
        Err(error) => Err(format!("failed to send GET request: {error}")),
    }
}

pub async fn get_room_message_count(
    bot_configuration: &Configuration,
    name: &str,
    password: &str,
) -> Result<usize, String> {
    match make_request(
        &bot_configuration,
        &format!("{NETCHAT_INSTANCE}/{password}/{name}/messageCount"),
    )
    .await
    {
        Ok(response) => {
            if response.status().is_server_error() {
                Err(format!("encountered server error"))
            } else if response.status() == 429 {
                Err(format!("encountered ratelimit"))
            } else if response.status() == 401 {
                Err(format!("unauthorized"))
            } else {
                match response.text().await {
                    Ok(text) => match text.parse() {
                        Ok(message_count) => Ok(message_count),
                        Err(error) => Err(format!("failed to deserialize response: {error}")),
                    },
                    Err(error) => Err(format!("failed to get response text: {error}")),
                }
            }
        }
        Err(error) => Err(format!("failed to send GET request: {error}")),
    }
}

pub async fn get_room_messages(
    bot_configuration: &Configuration,
    name: &str,
    password: &str,
) -> Result<Vec<String>, String> {
    match make_request(
        &bot_configuration,
        &format!("{NETCHAT_INSTANCE}/{password}/{name}/rawMessages"),
    )
    .await
    {
        Ok(response) => {
            if response.status().is_server_error() {
                Err(format!("encountered server error"))
            } else if response.status() == 429 {
                Err(format!("encountered ratelimit"))
            } else if response.status() == 401 {
                Err(format!("unauthorized"))
            } else {
                match response.text().await {
                    Ok(text) => match serde_json::from_str(&text) {
                        Ok(raw_messages) => Ok(raw_messages),
                        Err(error) => Err(format!("failed to deserialize response: {error}")),
                    },
                    Err(error) => Err(format!("failed to get response text: {error}")),
                }
            }
        }
        Err(error) => Err(format!("failed to send GET request: {error}")),
    }
}

pub async fn send_message(
    bot_configuration: &Configuration,
    name: &str,
    password: &str,
    username: &str,
    message: &str,
) -> Result<(), String> {
    let substitutions = vec![
        ("#", "||HAS||"),
        ("%", "||PER||"),
        ("&", "||AMP||"),
        ("/", "||SLA||"),
        ("?", "||QUE||"),
        ("\\", "||RSLA||"),
        ("\n", "||NEWL||"),
    ];
    let mut formatted_username = username.to_string();
    let mut formatted_message = message.to_string();
    for substitution in substitutions {
        formatted_username = formatted_username.replace(substitution.0, substitution.1);
        formatted_message = formatted_message.replace(substitution.0, substitution.1);
    }
    match make_request(&bot_configuration, &format!(
        "{NETCHAT_INSTANCE}/{password}/{name}/:FFFFFF/:000000/send/{formatted_username}/{formatted_message}"
    ))
    .await
    {
        Ok(response) => {
            if response.status().is_server_error() {
                Err(format!("encountered server error"))
            } else if response.status() == 429 {
                Err(format!("encountered ratelimit"))
            } else if response.status() == 401 {
                Err(format!("unauthorized"))
            } else {
                Ok(())
            }
        }
        Err(error) => Err(format!("failed to send GET request: {error}")),
    }
}
