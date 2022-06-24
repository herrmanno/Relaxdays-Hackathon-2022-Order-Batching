use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(value_parser)]
    pub input_file: String,

    #[clap(value_parser)]
    pub output_file: Option<String>,

    #[clap(long = "bi", default_value_t = 100)]
    pub num_batch_individuals: usize,

    #[clap(long = "bg", default_value_t = 100)]
    pub num_batch_generations: usize,

    #[clap(long = "wi", default_value_t = 100)]
    pub num_waive_individuals: usize,

    #[clap(long = "wg", default_value_t = 100)]
    pub num_waive_generations: usize,

    #[clap(long, short, action)]
    pub no_output: bool,
}
