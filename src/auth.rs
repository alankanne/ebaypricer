use reqwest::Client;
use serde::Deserialize;
use std::env;
use base64::Engine;

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

pub async fn get_token() -> Result<String, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let client_id = env::var("EBAY_CLIENT_ID")?;
    let client_secret = env::var("EBAY_CLIENT_SECRET")?;

    let creds = format!("{}:{}", client_id, client_secret);
    let creds_b64 = base64::engine::general_purpose::STANDARD.encode(creds);

    let client = Client::new();
    let resp = client
        .post("https://api.ebay.com/identity/v1/oauth2/token")
        .header("Authorization", format!("Basic {}", creds_b64))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("grant_type=client_credentials&scope=https://api.ebay.com/oauth/api_scope")
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;

    Ok(resp.access_token)
}
