use std::collections::BTreeSet;

use crate::{Model, model::{OrderedArticle, MAX_WEIGHT_PER_BATCH}};

use genevo::{operator::{prelude::*}, prelude::*, population::ValueEncodedGenomeBuilder};

type Fitness = usize;

/// The genotype / individual
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct BatchedArticles {
    batch_mapping: BatchMapping
}

impl BatchedArticles {
    fn from_batch_mapping(batch_mapping: BatchMapping) -> BatchedArticles {
        BatchedArticles { batch_mapping }
    }

    pub fn to_batches<'a>(&self, model: &'a Model) -> Vec<Batch<'a>> {
        let mut batches: Vec<Batch> = (0..model.max_batches_num())
            .into_iter()
            .map(|_| Batch::new())
            .collect();

        model.get_ordered_articles()
            .iter()
            .enumerate()
            .for_each(|(idx, article)| {
                let batch_id = self.batch_mapping[idx];
                batches[batch_id as usize].push(article)
            });

        batches
    }
}

impl Genotype for BatchedArticles {
    type Dna = BatchId;
}

#[derive(Debug)]
pub struct Batch<'a> {
    ordered_articles: Vec<&'a OrderedArticle>
}

impl<'a> Batch<'a> {
    fn new() -> Batch<'a> {
        Batch { ordered_articles: Vec::new() }
    }

    fn push(&mut self, article: &'a OrderedArticle) {
        self.ordered_articles.push(article);
    }

    pub fn fitness(&self) -> Option<usize> {
        if self.volume() > MAX_WEIGHT_PER_BATCH {
            None
        } else {
            Some(self.num_warehouses() * 10 + self.num_aisles() * 5)
        }
    }

    fn volume(&self) -> u16 {
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
}

type BatchId = usize;

/// A 'mapping' from articles (by index) to batches (by id / index)
type BatchMapping = Vec<BatchId>;

#[derive(Debug,Clone, Copy)]
struct FitnessCalc<'a> {
    model: &'a Model
}

impl<'a> FitnessCalc<'a> {
    // fn worst_batch_fitness(&self) -> usize {
    //     self.model.max_batches_num() * 15
    // }

    fn best_batch_fitness_approx(&self) -> usize {
        //FIXME:
        // calculate by
        //      num_warehouses(ordered_articles) * 10
        //    + num_aisles(ordered_articles) * 5
        50
    }
}

impl<'a> FitnessFunction<BatchMapping, Fitness> for FitnessCalc<'a> {
    fn fitness_of(&self, batch_mapping: &BatchMapping) -> Fitness {
        let batch_mapping = batch_mapping.clone();
        let batches =
            BatchedArticles::from_batch_mapping(batch_mapping)
                .to_batches(self.model);
        let fitness = batches
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
        // todo add penalty for used number of batches
        a.iter().sum::<Fitness>() / a.len()
    }

    fn highest_possible_fitness(&self) -> Fitness {
        100
    }

    fn lowest_possible_fitness(&self) -> Fitness {
        0
    }
}

pub fn find_best_batches(model: &Model) -> BatchedArticles {
    
    let fitness_calc = FitnessCalc { model };

    let initial_population: Population<_> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(
            model.get_ordered_articles().len().into(),
            0,
            (model.max_batches_num() - 1).into()
        ))
        .of_size(100)
        .uniform_at_random();

    let mut batch_sim = simulate(
        genetic_algorithm()
            .with_evaluation(fitness_calc)
            .with_selection(RouletteWheelSelector::new(
                0.7,
                2
            ))
            .with_crossover(UniformCrossBreeder::new())
            .with_mutation(RandomValueMutator::new(0.05, 0, model.max_batches_num() - 1))
            .with_reinsertion(ElitistReinserter::new(
                fitness_calc,
            true,
        0.7))
            .with_initial_population(initial_population)
            .build()
    )
        .until(GenerationLimit::new(1000))
        .build();

    loop {
        match batch_sim.step() {
            Ok(SimResult::Intermediate(step)) => {
                println!(
                    "Generation {} fitness {}",
                    step.result.best_solution.generation,
                    step.result.best_solution.solution.fitness
                );
            }
            Ok(SimResult::Final(step, time, duration, stop_reason)) => {
                println!(
                    "Generation {} fitness {}",
                    step.result.best_solution.generation,
                    step.result.best_solution.solution.fitness
                );
                println!("Time: {} Duration {} Stop reason {}", time, duration, stop_reason);
                let batch_mapping = step.result.best_solution.solution.genome;
                return BatchedArticles::from_batch_mapping(batch_mapping);

            }
            Err(err) => {
                panic!("{}",err)
            }
        }
    }
}