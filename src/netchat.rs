const NETCHAT_INSTANCE: &str = "https://netchat.repl.co";

async fn make_request(url: &str) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::ClientBuilder::new()
        .user_agent(format!(
            "netchat_bridge/{} (url:{url}) reqwest",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .unwrap();
    client.get(url).send().await
}

pub async fn get_room(name: &str, password: &str) -> Result<String, String> {
    match make_request(&format!("{NETCHAT_INSTANCE}/{password}/{name}/allMessages")).await {
        Ok(response) => {
            if response.status().is_server_error() {
                Err(format!("encountered server error"))
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

pub async fn is_initializing(name: &str, password: &str) -> Result<bool, String> {
    Ok(get_room(name, password)
        .await?
        .contains("<title>Initializing...</title><style>body"))
}

pub async fn is_correct_password(name: &str, password: &str) -> Result<bool, String> {
    Ok(get_room(name, password).await? != "Wrong Password!")
}
