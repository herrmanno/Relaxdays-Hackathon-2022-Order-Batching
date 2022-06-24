use anyhow::Result;

mod input;
mod model;
mod ga;

use input::*;
use model::*;
use ga::orders::*;

fn main() -> Result<()>  {
    let args = std::env::args().collect::<Vec<String>>();
    let input_path = args.get(1).expect("no input file defined");
    let input = load_input(input_path)?;

    let model = Model::from_input(&input);

    println!("Got {} different articles ordered", model.get_ordered_articles().len());
    println!("Max number of batches {}", model.max_batches_num());
    println!("Max number of articles in  batch {}", model.max_items_per_batch());
    // println!("{:?}", model);

    // find best arrangement of articles in orders by using ga where
    // a solution consists of `max_num_batches` batches and a batch consists of
    // `max_num_articles_per_batch` articles (article ids)

    let batched_articles = find_best_batches(&model);
    println!("{:?}", batched_articles);
    let tour_cost = batched_articles
        .to_batches(&model)
        .iter()
        .map(Batch::fitness)
        .sum::<Option<usize>>();
    println!("Tour cost {:?}", tour_cost);
    // todo!("Group articles in batches");
    // todo!("Group batches in waves");

    Ok(())
}
