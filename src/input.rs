use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json as json;

#[derive(Deserialize)]
pub(crate) struct Input {
    #[serde(rename = "ArticleLocations")]
    pub(crate) article_locations: Vec<ArticleLocation>,

    #[serde(rename = "Orders")]
    pub(crate) orders: Vec<Order>,

    #[serde(rename = "Articles")]
    pub(crate) articles: Vec<Article>,
}

#[derive(Deserialize)]
pub(crate) struct ArticleLocation {
    #[serde(rename = "Warehouse")]
    pub(crate) warehouse: u16,

    #[serde(rename = "Aisle")]
    pub(crate) aisle: u16,

    #[allow(dead_code)]
    #[serde(rename = "Position")]
    pub(crate) position: u16,

    #[serde(rename = "ArticleId")]
    pub(crate) article_id: u16,
}

#[derive(Deserialize)]
pub(crate) struct Order {
    #[serde(rename = "OrderId")]
    pub(crate) order_id: u16,
    #[serde(rename = "ArticleIds")]
    pub(crate) article_ids: Vec<u16>,
}

#[derive(Deserialize)]
pub(crate) struct Article {
    #[serde(rename = "ArticleId")]
    pub(crate) article_id: u16,

    #[serde(rename = "Volume")]
    pub(crate) volume: u16,
}

pub(crate) fn load_input(file_path: &str) -> Result<Input> {
    let input_file = std::fs::File::open(file_path)?;
    json::from_reader(input_file).context("cannot deserialize input")
}
