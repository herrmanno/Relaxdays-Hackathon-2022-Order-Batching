use anyhow::Result;

mod cli;
mod ga;
mod input;
mod model;
mod output;

use clap::Parser;
use ga::orders::*;
use input::*;
use model::*;

use crate::{cli::Cli, ga::batches::find_best_waives, output::Output};

fn main() -> Result<()> {
    let args = Cli::parse();
    let input = load_input(args.input_file.as_str())?;

    let model = Model::from_input(&input);

    if cfg!(feature = "info") {
        println!(
            "Got {} different articles ordered",
            model.get_ordered_articles().len()
        );
        println!("Max number of batches {}", model.max_batches_num());
        println!(
            "Max number of articles in  batch {}",
            model.max_items_per_batch()
        );
    }

    let batched_articles = find_best_batches(
        &model,
        args.num_batch_individuals,
        args.num_batch_generations,
    );

    let waived_batches = find_best_waives(
        &model,
        &batched_articles,
        args.num_waive_individuals,
        args.num_waive_generations,
    );

    let tour_cost_batches = batched_articles
        .tour_cost()
        .expect(format!("Calculated invalid batches {:?}", batched_articles).as_str());
    let rest_cost_batches = batched_articles.rest_cost();
    let rest_cost_waives = waived_batches.rest_cost();
    let overall_cost = tour_cost_batches + rest_cost_batches + rest_cost_waives;

    println!("");
    println!("[RESULTS]");
    println!("#Waives {}", waived_batches.to_waives().len());
    println!("#WBatches {}", batched_articles.to_batches().len());
    println!("Tour cost {:?}", tour_cost_batches);
    println!("Rest cost (batches) {:?}", rest_cost_batches);
    println!("Rest cost (waives) {:?}", rest_cost_waives);
    println!("");
    println!("Overall cost {}", overall_cost);

    let output = Output::new(&batched_articles, &waived_batches);
    if let Some(output_path) = args.output_file {
        let out_file = std::fs::File::create(&output_path)
            .expect(format!("Cannot open out file at {}", output_path).as_str());
        serde_json::to_writer_pretty(out_file, &output)?;
    } else if !args.no_output {
        serde_json::to_writer_pretty(std::io::stdout(), &output)?;
    }

    Ok(())
}
