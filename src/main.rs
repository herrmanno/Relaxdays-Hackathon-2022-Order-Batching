use std::collections::BTreeSet;

use anyhow::Result;

mod input;
mod output;
mod model;
mod ga;

use input::*;
use model::*;
use ga::orders::*;

use crate::{ga::batches::find_best_waives, output::Output};

fn main() -> Result<()>  {
    let args = std::env::args().collect::<Vec<String>>();
    let input_path = args.get(1).expect("no input file defined");
    let input = load_input(input_path)?;
    let output_path = args.get(2);

    let model = Model::from_input(&input);

    println!("Got {} different articles ordered", model.get_ordered_articles().len());
    println!("Max number of batches {}", model.max_batches_num());
    println!("Max number of articles in  batch {}", model.max_items_per_batch());
    // println!("{:?}", model);

    let batched_articles = find_best_batches(&model);
    let tour_cost_batches = batched_articles.tour_cost(&model)
        .expect(format!("Calculated invalid batches {:?}", batched_articles).as_str());
    let rest_cost_batches = batched_articles.rest_cost();
    println!("");
    println!("Tour cost {:?}", tour_cost_batches);
    println!("Rest cost (batches) {:?}", rest_cost_batches);
    println!("");

    let batches = batched_articles.to_batches(&model);
    for (idx, batch) in batches.iter().enumerate() {
        println!("Batch {}", idx);
        println!("\t- #articles: {}", batch.num_articles());
        println!("\t- orders: {:?}", batch.order_ids_in_batch());
    }

    let waived_batches = find_best_waives(&model, &batched_articles);
    let waives = waived_batches.to_waives(&model, &batched_articles);

    if waived_batches.has_split_orders(&model, &batched_articles) {
        println!("Waived batches are invalid because orders are split between waives!")
    } else {
        for (idx, waive) in waives.iter().enumerate() {
            let baches = waive.batches();
            println!("Waive {}", idx);
            println!("\t- #articles: {}", baches.iter().map(|b| b.num_articles()).sum::<usize>());
            println!("\t- batches: {:?}", batches.iter().map(|b| b.id).collect::<BTreeSet<_>>());
        }
    }

    let rest_cost_waives = waived_batches.rest_cost();
    println!("Rest cost (waives) {:?}", rest_cost_waives);

    let overall_cost =
        tour_cost_batches + rest_cost_batches + rest_cost_waives;
    println!("");
    println!("Overall cost {}", overall_cost);

    let output = Output::new(&model, &batched_articles, &waived_batches);
    if let Some(output_path) = output_path {
        let out_file = std::fs::File::create(output_path)
            .expect(format!("Cannot open out file at {}", output_path).as_str());
        serde_json::to_writer_pretty(out_file, &output)?;
    } else {
        serde_json::to_writer_pretty(std::io::stdout(), &output)?;
    }

    Ok(())
}
