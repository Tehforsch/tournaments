use rand::rngs::ThreadRng;
use rand::Rng;
use serde::Deserialize;

use crate::math::binomial_distribution;
use crate::Team;

#[derive(Deserialize, Debug, Clone)]
pub struct Component<P> {
    pub r#type: ComponentType,
    pub teams: Vec<P>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum ComponentType {
    BestOfN(usize),
    GroupStage(GroupStage),
}

fn run_best_of_n(n: usize, input: &mut [Team], rng: &mut ThreadRng) {
    assert_eq!(input.len(), 2);
    let p = input[0].probability_to_win_against(&input[1]);
    let total: f64 = (0..=(n - 1) / 2)
        .map(|k| binomial_distribution(p, n, k))
        .sum();
    if rng.gen_range(0.0..=1.0) > total {
        input.swap(0, 1);
    }
}

impl<P> Component<P> {
    pub fn run(&self, input: &mut [Team], rng: &mut ThreadRng) {
        match self.r#type {
            ComponentType::BestOfN(n) => run_best_of_n(n, input, rng),
            ComponentType::GroupStage(group) => group.run(input, rng),
        }
    }

    pub fn get_placement_index_from_placement_name(&self, placement: &str) -> usize {
        match self.r#type {
            ComponentType::BestOfN(_) => match placement {
                "winner" => 0,
                "loser" => 1,
                _ => panic!("Wrong placement name: {}", placement),
            },
            ComponentType::GroupStage(group) => {
                group.get_placement_index_from_placement_name(placement)
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct GroupStage {
    num_games_per_series: usize,
}

impl GroupStage {
    fn run(&self, input: &mut [Team], rng: &mut ThreadRng) {
        struct Result {
            team_index: usize,
            num_games_won: i32,
        }
        let mut results: Vec<Result> = input
            .iter()
            .map(|team| Result {
                team_index: team.index,
                num_games_won: 0,
            })
            .collect();
        for (i, team1) in input.iter().enumerate() {
            for (j, team2) in input.iter().enumerate() {
                if i >= j {
                    continue;
                }
                for _ in 0..self.num_games_per_series {
                    if team1.wins_against(&team2, rng) {
                        results[i].num_games_won += 1;
                    } else {
                        results[j].num_games_won += 1;
                    }
                }
            }
        }
        input.sort_by_key(|team| {
            -results
                .iter()
                .find(|result| result.team_index == team.index)
                .unwrap()
                .num_games_won
        });
        self.tiebreak(input, rng)
    }

    fn tiebreak(&self, input: &mut [Team], rng: &mut ThreadRng) {
        todo!()
    }

    fn get_placement_index_from_placement_name(&self, placement: &str) -> usize {
        match placement {
            "1st" => 0,
            "2nd" => 1,
            "3rd" => 2,
            "4th" => 3,
            "5th" => 4,
            "6th" => 5,
            "7th" => 6,
            "8th" => 7,
            _ => panic!("Unknown group placement: {}", placement),
        }
    }
}
