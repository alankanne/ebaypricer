use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ConvertedAmount {
    pub value: Option<String>,
    pub currency: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Item {
    pub itemId: Option<String>,
    pub title: Option<String>,
    pub condition: Option<String>,
    pub price: Option<ConvertedAmount>,
    pub itemWebUrl: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ItemSummary {
    pub itemId: Option<String>,
    pub title: Option<String>,
    pub price: Option<ConvertedAmount>,
    pub condition: Option<String>,
    pub itemWebUrl: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    pub itemSummaries: Option<Vec<ItemSummary>>,
}

pub async fn get_item_by_legacy_id(
    client: &Client,
    token: &str,
    legacy_id: &str,
) -> Result<Item, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.ebay.com/buy/browse/v1/item/get_item_by_legacy_id?legacy_item_id={}",
        legacy_id
    );

    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .json::<Item>()
        .await?;

    Ok(resp)
}

pub async fn search_by_keywords(
    client: &Client,
    token: &str,
    query: &str,
) -> Result<Vec<ItemSummary>, Box<dyn std::error::Error>> {
    let encoded = urlencoding::encode(query);
    let url = format!(
        "https://api.ebay.com/buy/browse/v1/item_summary/search?q={}&limit=50",
        encoded
    );

    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .json::<SearchResponse>()
        .await?;

    Ok(resp.itemSummaries.unwrap_or_default())
}
