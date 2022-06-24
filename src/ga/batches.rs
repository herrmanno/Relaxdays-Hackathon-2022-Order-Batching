//! Genetic search for partitioning batches into waives
use std::collections::BTreeSet;
use std::ops::{Div};

use crate::model::*;
use crate::ga::orders::{Batch,BatchedArticles};

use genevo::{operator::{prelude::*}, prelude::*, population::ValueEncodedGenomeBuilder};

type Fitness = usize;

/// A mapping from batches to waives
/// 
/// Acts as genotype / individual
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct WaivedBatches {
    waive_mapping: WaiveMapping
}

impl WaivedBatches {
    fn from_waive_mapping(waive_mapping: WaiveMapping) -> WaivedBatches {
        WaivedBatches { waive_mapping: waive_mapping }
    }

    pub fn to_waives<'a>(&self, model: &'a Model, batched_articles: &'a BatchedArticles) -> Vec<Waive<'a>> {
        let mut waives: Vec<Waive<'a>> = (0..batched_articles.len())
            .into_iter()
            .map(|_| Waive::new())
            .collect();

        let batches: Vec<Batch<'a>> = batched_articles.to_batches(model);

        batches
            .iter()
            .enumerate()
            .for_each(|(idx, batch)| {
                let waive_id = self.waive_mapping[idx];
                waives[waive_id as usize].push(batch.to_owned())
            });

        waives
            .into_iter()
            .filter(|waive| waive.num_batches() > 0)
            .collect()
    }

    pub fn has_split_orders(&self, model: &Model, batched_articles: &BatchedArticles) -> bool {
        self.get_split_orders(model, batched_articles).len() > 0
    }

    pub fn get_split_orders(&self, model: &Model, batched_articles: &BatchedArticles) -> BTreeSet<ID> {
        let order_ids_per_batch = self.to_waives(model, batched_articles)
            .iter()
            .map(Waive::order_ids_in_waive)
            .collect::<Vec<_>>();

        let mut split_order_ids = BTreeSet::new();

        order_ids_per_batch
            .iter()
            .enumerate()
            .for_each(|(idx, order_ids)| {
                order_ids_per_batch
                    .iter()
                    .skip(idx + 1)
                    .for_each(|other_order_ids| {
                        let mut common_order_ids = order_ids & other_order_ids;
                        split_order_ids.append(&mut common_order_ids);
                    })
            });

        split_order_ids
    }

    pub fn rest_cost(&self) -> usize {
        let num_waives = self.waive_mapping
            .iter()
            .collect::<BTreeSet<_>>()
            .len();

        num_waives * COST_PER_WAIVE
    }
}

impl Genotype for WaivedBatches {
    type Dna = WaiveId;
}

/// A singe batch, containing (ordered) articles
#[derive(Debug)]
pub struct Waive<'a> {
    batches: Vec<Batch<'a>>
}

impl<'a> Waive<'a> {
    fn new() -> Waive<'a> {
        Waive { batches: Vec::new() }
    }

    fn push(&mut self, batch: Batch<'a>) {
        self.batches.push(batch);
    }

    fn num_batches(&self) -> usize {
        self.batches.len()
    }

    pub fn batches(&self) -> &Vec<Batch<'a>> {
        &self.batches
    }

    pub fn num_articles(&self) -> usize {
        self.batches
            .iter()
            .map(Batch::num_articles)
            .sum::<usize>()
    }

    pub fn order_ids_in_waive(&self) -> BTreeSet<ID> {
        self.batches
            .iter()
            .flat_map(|batch| batch.order_ids_in_batch().into_iter())
            .collect()
    }

}

/// id (index) of a single, specific waive
type WaiveId = usize;

/// A 'mapping' from batches (by index) to waives (by id / index)
/// 
/// Acts as DNA for the genotype `WaivedBatches`
type WaiveMapping = Vec<WaiveId>;

#[derive(Debug,Clone, Copy)]
struct FitnessCalc<'a> {
    model: &'a Model,
    batched_articles: &'a BatchedArticles
}

impl<'a> FitnessCalc<'a> {
    #[allow(dead_code)] // TODO: remove if not used anymore
    fn average_num_of_waives(&self, model: &Model) -> f32 {
        let batches = self.batched_articles.to_batches(model);
        let average_articles_per_batch = batches
            .iter()
            .map(|batch| batch.num_articles())
            .sum::<usize>()
            .div(batches.len());

        let average_batches_per_waive = MAX_ARTICLES_PER_WAIVE / average_articles_per_batch;

        (batches.len() as f32) / (average_batches_per_waive as f32).min(1f32)
    }
}

impl<'a> FitnessFunction<WaiveMapping, Fitness> for FitnessCalc<'a> {
    fn fitness_of(&self, waive_mapping: &WaiveMapping) -> Fitness {
        let waived_batches = WaivedBatches::from_waive_mapping(waive_mapping.to_owned());
        let waives = waived_batches.to_waives(self.model, self.batched_articles);
        
        let has_invalid_waive = waives
            .iter()
            .any(|wave| wave.num_articles() > MAX_ARTICLES_PER_WAIVE);

        if has_invalid_waive {
            return 0
        }

        let base_fitness = {
            // let average_num_of_waves = self.average_num_of_waives(self.model);
            // let relative_fitness = average_num_of_waves / (waives.len().max(1) as f32);
            let relative_fitness = 1f32 / (waives.len().max(1) as f32);
            (relative_fitness * 100f32) as Fitness
        };

        let num_split_orders = waived_batches.get_split_orders(self.model, self.batched_articles).len();

        let max_num_split_orders = self.model.num_orders();

        let split_bonus = max_num_split_orders - num_split_orders;

        // add split bonus if base fitness is still low to favour solution that dont have any
        // penalty for splitting order
        if base_fitness < split_bonus {
            (base_fitness + split_bonus).min(100)
        } else {
            base_fitness.min(100)
        }
    }

    fn average(&self, a: &[Fitness]) -> Fitness {
        // todo add penalty for used number of waives
        a.iter().sum::<Fitness>() / a.len()
    }

    fn highest_possible_fitness(&self) -> Fitness {
        100
    }

    fn lowest_possible_fitness(&self) -> Fitness {
        0
    }
}

struct GenomeConfig {
    length: usize,
    min_value: usize,
    max_value: usize,
}

pub fn find_best_waives(model: &Model, batched_articles: &BatchedArticles) -> WaivedBatches {
    
    let fitness_calc = FitnessCalc { model, batched_articles };

    let genome_config = GenomeConfig {
        length: batched_articles.to_batches(model).len(),
        min_value: 0,
        max_value: batched_articles.to_batches(model).len() - 1
    };

    let initial_population: Population<_> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(
            genome_config.length,
            genome_config.min_value,
            genome_config.max_value
        ))
        .of_size(50)
        .uniform_at_random();

    let mut batch_sim = simulate(
        genetic_algorithm()
            .with_evaluation(fitness_calc)
            .with_selection(RouletteWheelSelector::new(
                0.7,
                2
            ))
            .with_crossover(UniformCrossBreeder::new())
            .with_mutation(RandomValueMutator::new(
                0.05,
                genome_config.min_value,
                genome_config.max_value))
            .with_reinsertion(ElitistReinserter::new(
                fitness_calc,
            true,
        0.7))
            .with_initial_population(initial_population)
            .build()
    )
        .until(
            Or::new(
                GenerationLimit::new(200),
                FitnessLimit::new(100)
            ))
        .build();

    loop {
        match batch_sim.step() {
            Ok(SimResult::Intermediate(step)) => {
                if cfg!(feature = "verbose") {
                    println!(
                        "Generation {} fitness {}",
                        step.result.best_solution.generation,
                        step.result.best_solution.solution.fitness
                    );
                }
            }
            Ok(SimResult::Final(step, time, duration, stop_reason)) => {
                println!(
                    "Generation {} fitness {}",
                    step.result.best_solution.generation,
                    step.result.best_solution.solution.fitness
                );
                if cfg!(feature = "verbose") {
                    println!("Time: {} Duration {} Stop reason {}", time, duration, stop_reason);
                }
                let batch_mapping = step.result.best_solution.solution.genome;
                return WaivedBatches::from_waive_mapping(batch_mapping);

            }
            Err(err) => {
                panic!("{}",err)
            }
        }
    }
}