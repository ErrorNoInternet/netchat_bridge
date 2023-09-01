const NETCHAT_INSTANCE: &str = "https://netchat.repl.co/";

pub async fn get_room(name: String, password: String) -> Result<String, String> {
    match reqwest::get(format!("{NETCHAT_INSTANCE}/{password}/{name}/allMessages")).await {
        Ok(response) => match response.text().await {
            Ok(text) => Ok(text),
            Err(error) => Err(format!("failed to get response text: {error}")),
        },
        Err(error) => Err(format!("failed to send GET request: {error}")),
    }
}

pub async fn is_initializing(name: String, password: String) -> Result<bool, String> {
    Ok(get_room(name, password)
        .await?
        .contains("<title>Initializing...</title><style>body"))
}
