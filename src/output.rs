use serde::{Serialize};

use crate::model::*;
use crate::ga::orders::*;
use crate::ga::batches::*;

#[derive(Serialize,Debug)]
pub struct Output {
    #[serde(rename = "Waves")]
    waves: Vec<Wave>,
    #[serde(rename = "Batches")]
    batches: Vec<Batch>,
}

impl Output {
    pub fn new(
        model: &Model,
        batched_articles: &BatchedArticles,
        waived_batches: &WaivedBatches
    ) -> Output {
        let waves = waived_batches
            .to_waives(model, batched_articles)
            .iter()
            .enumerate()
            .map(|(idx,waive)| {
                let wave_id = idx as ID;
                let batch_ids = waive.batches()
                    .iter()
                    .map(|b| b.id as ID)
                    .collect();
                let order_ids = waive.order_ids_in_waive().into_iter().collect();
                let wave_size = waive.num_articles();

                Wave { wave_id, batch_ids, order_ids, wave_size }
            })
            .collect();

        let batches = batched_articles
            .to_batches(model)
            .iter()
            .map(|batch| {
                let batch_id = batch.id as ID;
                let items = batch.ordered_articles()
                    .into_iter()
                    .map(|article| {
                        let order_id = article.order_id;
                        let article_id = article.id;
                        Item { order_id, article_id }
                    })
                    .collect();
                let batch_volume = batch.volume() as usize;

                Batch { batch_id, items, batch_volume }
            })
            .collect();

        Output { waves, batches }
    }
}

#[derive(Serialize,Debug)]
pub struct Wave {
    #[serde(rename = "WaveId")]
    wave_id: ID,
    #[serde(rename = "BatchIds")]
    batch_ids: Vec<ID>,
    #[serde(rename = "OrderIds")]
    order_ids: Vec<ID>,
    #[serde(rename = "WaveSize")]
    wave_size: usize,
}

#[derive(Serialize,Debug)]
pub struct Batch {
    #[serde(rename = "BatchId")]
    batch_id: ID,
    #[serde(rename = "Items")]
    items: Vec<Item>,
    #[serde(rename = "BatchVolume")]
    batch_volume: usize,
}

#[derive(Serialize,Debug)]
pub struct Item {
    #[serde(rename = "OrderId")]
    order_id: ID,
    #[serde(rename = "ArticleId")]
    article_id: ID,
}
