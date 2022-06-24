use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(value_parser)]
    pub(crate) input_file: String,

    #[clap(value_parser)]
    pub(crate) output_file: Option<String>,

    #[clap(
        long = "batch-population",
        alias = "bp",
        default_value_t = 100,
        help = "Initial size of orders<->batches population"
    )]
    pub(crate) num_batch_individuals: usize,

    #[clap(
        long = "batch-generations",
        alias = "bg",
        default_value_t = 100,
        help = "Max number of generations for orders<->batches"
    )]
    pub(crate) num_batch_generations: usize,

    #[clap(
        long = "waive-population",
        alias = "wp",
        default_value_t = 100,
        help = "Initial size of batches<->waives population"
    )]
    pub(crate) num_waive_individuals: usize,

    #[clap(
        long = "waive-generations",
        alias = "wg",
        default_value_t = 100,
        help = "Max number of generations for batches<->waives"
    )]
    pub(crate) num_waive_generations: usize,

    #[clap(long, action)]
    pub(crate) no_output: bool,
}
