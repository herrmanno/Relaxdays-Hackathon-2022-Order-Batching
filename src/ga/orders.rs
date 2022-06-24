//! Genetic search for partitioning order into batches
use std::collections::BTreeSet;

use crate::model::*;

use genevo::{operator::{prelude::*}, prelude::*, population::ValueEncodedGenomeBuilder};

type Fitness = usize;

/// A set of batches
/// 
/// Acts as genotype / individual
#[derive(Clone, Debug)]
pub struct BatchedArticles<'a> {
    batch_mapping: BatchMapping,
    batches: Vec<Batch<'a>>,
}

impl<'a> PartialEq for BatchedArticles<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.batch_mapping == other.batch_mapping
    }
}

impl<'a> PartialOrd for BatchedArticles<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.batch_mapping.partial_cmp(&other.batch_mapping)
    }
}

impl<'a> BatchedArticles<'a> {
    fn from_batch_mapping(batch_mapping: BatchMapping, model: &'a Model) -> BatchedArticles<'a> {
        let mut batches: Vec<Batch> = (0..model.max_batches_num())
                .into_iter()
                .enumerate()
                .map(|(idx,_)| Batch::new(idx))
                .collect();

            model.get_ordered_articles()
                .into_iter()
                .enumerate()
                .for_each(|(idx, article)| {
                    let batch_id = batch_mapping[idx];
                    batches[batch_id as usize].push(article)
                });

        batches = batches
                .into_iter()
                .filter(|batch| batch.num_articles() > 0)
                .collect();

        BatchedArticles { batch_mapping, batches }
    }

    pub fn len(&self) -> usize {
        self.batch_mapping.len()
    }

    pub fn to_batches(&self) -> &Vec<Batch<'a>> {
        &self.batches
    }

    pub fn rest_cost(&self) -> usize {
        let num_batches = self.batch_mapping
            .iter()
            .collect::<BTreeSet<_>>()
            .len();

        num_batches * COST_PER_BATCH
    }

    pub fn tour_cost(&self) -> Option<usize> {
        self
            .to_batches()
            .iter()
            .map(Batch::fitness)
            .sum::<Option<usize>>()
    }
}

impl<'a> Genotype for BatchedArticles<'a> {
    type Dna = BatchId;
}

/// A singe batch, containing (ordered) articles
#[derive(Debug, Clone)]
pub struct Batch<'a> {
    pub id: BatchId,
    ordered_articles: Vec<&'a OrderedArticle>
}

impl<'a> Batch<'a> {
    fn new(id: BatchId) -> Batch<'a> {
        Batch { id, ordered_articles: Vec::new() }
    }

    fn push(&mut self, article: &'a OrderedArticle) {
        self.ordered_articles.push(article);
    }

    pub fn fitness(&self) -> Option<usize> {
        if self.volume() > MAX_WEIGHT_PER_BATCH {
            None
        } else {
            Some(
                self.num_warehouses() * COST_PER_WAREHOUSE
              + self.num_aisles() * COST_PER_AISLE
              + COST_PER_BATCH)
        }
    }

    pub fn ordered_articles(&self) -> &Vec<&OrderedArticle> {
        &self.ordered_articles
    }

    pub fn num_articles(&self) -> usize {
        self.ordered_articles.len()
    }

    pub fn volume(&self) -> u16 {
        self.ordered_articles
            .iter()
            .map(|article| article.volume as u16)
            .sum::<u16>()
    }

    fn num_warehouses(&self) -> usize {
        self.ordered_articles
            .iter()
            .map(|article| article.location.warehouse)
            .collect::<BTreeSet<_>>()
            .len()
    }

    fn num_aisles(&self) -> usize {
        self.ordered_articles
            .iter()
            //TODO: check if two aisles w/ same ID in different warehouses count as one or two
            .map(|article| (article.location.warehouse, article.location.aisle))
            .collect::<BTreeSet<_>>()
            .len()
    }

    pub fn order_ids_in_batch(&self) -> BTreeSet<ID> {
        self.ordered_articles
            .iter()
            .map(|article| article.order_id)
            .collect()
    }
}

/// id (index) of a single, specific batch
type BatchId = usize;

/// A 'mapping' from articles (by index) to batches (by id / index)
/// 
/// Acts as DNA for the genotype `BatchedArticles`
type BatchMapping = Vec<BatchId>;

#[derive(Debug,Clone, Copy)]
struct FitnessCalc<'a> {
    model: &'a Model
}

impl<'a> FitnessCalc<'a> {
    fn best_batch_fitness_approx(&self) -> usize {
          self.model.num_warehouses_of_orders() * 10
        + self.model.num_aisles_of_orders() * 5
    }
}

impl<'a> FitnessFunction<BatchMapping, Fitness> for FitnessCalc<'a> {
    // TODO: add penalty for used number of batches
    // TODO: add penalty if articles of one order are in many batches
    fn fitness_of(&self, batch_mapping: &BatchMapping) -> Fitness {
        let batch_mapping = batch_mapping.clone();
        let fitness = BatchedArticles::from_batch_mapping(batch_mapping, self.model).to_batches()
            .iter()
            .map(Batch::fitness)
            .sum::<Option<usize>>();
        
        if let Some(fitness) = fitness {
            let f = (self.best_batch_fitness_approx() as f32) * 100f32 / (fitness as f32);
            f as Fitness
        } else {
            0
        }
    }

    fn average(&self, a: &[Fitness]) -> Fitness {
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

pub fn find_best_batches(model: &Model) -> BatchedArticles {
    
    let fitness_calc = FitnessCalc { model };

    let genome_config = GenomeConfig {
        length: model.get_ordered_articles().len(),
        min_value: 0,
        max_value: model.max_batches_num() - 1
    };

    println!(
        "Best possible tour cost: {}",
        fitness_calc.best_batch_fitness_approx());

    let initial_population: Population<_> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(
            genome_config.length,
            genome_config.min_value,
            genome_config.max_value,
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
                return BatchedArticles::from_batch_mapping(batch_mapping, model);

            }
            Err(err) => {
                panic!("{}",err)
            }
        }
    }
}