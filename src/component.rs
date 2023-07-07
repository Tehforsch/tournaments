use rand::rngs::ThreadRng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

use crate::math::binomial_distribution;
use crate::Team;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Component<P> {
    pub r#type: ComponentType,
    pub teams: Vec<P>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ComponentType {
    BestOfN(usize),
}

fn run_best_of_n(n: usize, mut input: Vec<Team>, rng: &mut ThreadRng) -> Vec<Team> {
    assert_eq!(input.len(), 2);
    let p = input[0].probability_to_win_against(&input[1]);
    let total: f64 = (0..=(n - 1) / 2)
        .map(|k| binomial_distribution(p, n, k))
        .sum();
    if rng.gen_range(0.0..=1.0) > total {
        input.swap(0, 1);
    }
    input
}

impl<P> Component<P> {
    pub fn run(&self, input: Vec<Team>, rng: &mut ThreadRng) -> Vec<Team> {
        match self.r#type {
            ComponentType::BestOfN(n) => run_best_of_n(n, input, rng),
        }
    }

    pub fn get_placement_index_from_placement_name(&self, placement: &str) -> usize {
        match self.r#type {
            ComponentType::BestOfN(_) => match placement {
                "winner" => 0,
                "loser" => 1,
                _ => panic!("Wrong placement name: {}", placement),
            },
        }
    }
}
