use serde::{Deserialize};
use serde_json as json;
use anyhow::{Result, Context};

#[derive(Deserialize,Debug)]
pub struct Input {
    #[serde(rename = "ArticleLocations")]
    pub article_locations: Vec<ArticleLocation>,
    #[serde(rename = "Orders")]
    pub orders: Vec<Order>,
    #[serde(rename = "Articles")]
    pub articles: Vec<Article>
}

#[derive(Deserialize,Debug)]
pub struct ArticleLocation {
    #[serde(rename = "Warehouse")]
    pub warehouse: u16,
    #[serde(rename = "Aisle")]
    pub aisle: u16,
    #[serde(rename = "Position")]
    pub position: u16,
    #[serde(rename = "ArticleId")]
    pub article_id: u16,
}

#[derive(Deserialize,Debug)]
pub struct Order {
    #[serde(rename = "OrderId")]
    pub order_id: u16,
    #[serde(rename = "ArticleIds")]
    pub article_ids: Vec<u16>,
}

#[derive(Deserialize,Debug)]
pub struct Article {
    #[serde(rename = "ArticleId")]
    pub article_id: u16,
    #[serde(rename = "Volume")]
    pub volume: u16,
}


pub fn load_input(file_path: &str) -> Result<Input> {
    let input_file = std::fs::File::open(file_path)?;
    json::from_reader(input_file)
        .context("cannot deserialize input")
}