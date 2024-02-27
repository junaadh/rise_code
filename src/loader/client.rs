use std::env;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient};

pub async fn get_client() -> Result<DiscordIpcClient, String> {
    let client_id = env::var("clientid")
        .map_err(|err| log::error!("Unable to fetch client_id: {err}"))
        .unwrap_or_default();
    let mut client = DiscordIpcClient::new(&client_id).expect("invalid client id");
    client
        .connect()
        .map_err(|_| "Failed at connecting to discord client".to_string())?;
    Ok(client)
}
