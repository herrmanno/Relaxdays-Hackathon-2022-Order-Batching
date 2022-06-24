use std::{collections::{BTreeMap, BTreeSet}, iter::repeat};

use crate::input::Input;

pub const MAX_WEIGHT_PER_BATCH: u16 = 1000;

pub type ID = u16;

#[derive(Debug)]
pub struct Model {
    // articles: Articles,
    orders: Orders
}

impl Model {
    pub fn from_input(input: &Input) -> Model {
        let articles = Articles::from_input(input);
        let orders = Orders::from_input(input, &articles);
        Model { orders }
    }

    pub fn get_ordered_articles(&self) -> Vec<&OrderedArticle> {
        self.orders.ordered_articles()
    }

    pub fn max_batches_num(&self) -> usize {
        //TODO: cache result
        self.orders.ordered_articles().len()
    }

    pub fn max_items_per_batch(&self) -> usize {
        let mut volumes = self.get_ordered_articles()
            .iter()
            .map(|article| article.volume as u16)
            .collect::<Vec<u16>>();
        
        volumes.sort();

        let mut sum = 0;
        let mut n = 0;
        for v in volumes {
            if sum + v > MAX_WEIGHT_PER_BATCH {
                break
            } else {
                sum += v;
                n += 1;
            }
        }

        n
    }
}

#[derive(Debug)]
pub struct Articles {
    article_map: BTreeMap<ID, Article>
}

impl Articles {
    fn from_input(input: &Input) -> Articles {
        let mut article_map = BTreeMap::new();

        let ordered_article_ids = input.orders
            .iter()
            .flat_map(|order| order.article_ids.iter())
            .map(|id| *id)
            .collect::<BTreeSet<_>>();
        
        let ordered_articles = ordered_article_ids
            .iter()
            .map(|id| {
                let volume = input.articles
                    .iter()
                    .find(|article| article.article_id == *id)
                    .map(|article| article.volume as u8)
                    .expect(format!("Article {} ordered but not listed as article", id).as_str());
                
                let location = input.article_locations
                    .iter()
                    .find(|article_location| article_location.article_id == *id)
                    .map(|article_location| {
                        ArticleLocation {
                            warehouse: article_location.warehouse,
                            aisle: article_location.aisle
                        }
                    })
                    .expect(format!("Article {} ordered but has no location", id).as_str());

                Article { id: *id, volume, location }
            });

        ordered_articles.for_each(|article| {
            article_map.insert(article.id, article);
        });

        Articles { article_map }
    }

    pub fn get_article(&self, id: ID) -> &Article {
        self.article_map.get(&id).unwrap()
    }

    // pub fn size(&self) -> usize {
    //     self.article_map.len()
    // }
}


#[derive(Debug, Clone, Copy)]
pub struct Article {
    id: u16,
    volume: u8,
    location: ArticleLocation
}

#[derive(Debug, Clone, Copy)]
pub struct ArticleLocation {
    pub warehouse: ID,
    pub aisle: ID,
    // position: u16,
}

#[derive(Debug)]
pub struct OrderedArticle {
    pub order_id: ID,
    pub id: ID,
    pub volume: u8,
    pub location: ArticleLocation
}

impl OrderedArticle {
    pub fn new(order_id: ID, article: Article) -> OrderedArticle {
        OrderedArticle {
            order_id,
            id: article.id,
            volume: article.volume,
            location: article.location
        }
    }
}

#[derive(Debug)]
pub struct Orders {
    orders: Vec<Order>
}

impl Orders {
    fn from_input(input: &Input, articles: &Articles) -> Orders {
        let ordered_articles = input.orders
            .iter()
            .map(|order| {
                repeat(order.order_id).zip(order.article_ids.iter())
                    .map(|(order_id,article_id)| {
                        let article = articles.get_article(*article_id);
                        OrderedArticle::new(order_id, *article)
                    })
                    .collect::<Vec<OrderedArticle>>()
            })
            .map(Order::new)
            .collect::<Vec<_>>();

        Orders { orders: ordered_articles }
    }

    fn ordered_articles(&self) -> Vec<&OrderedArticle> {
        self.orders
            .iter()
            .flat_map(|o| o.articles.iter())
            .collect()
    }

}

#[derive(Debug)]
pub struct Order {
    articles: Vec<OrderedArticle>
}

impl Order {
    fn new(articles: Vec<OrderedArticle>) -> Order {
        Order { articles }
    }

    // fn articles(&self) -> &[OrderedArticle] {
    //     self.articles.as_slice()
    // }
}