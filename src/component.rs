use hashbrown::HashMap;
use itertools::Itertools;
use rand::rngs::ThreadRng;
use rand::Rng;
use serde::Deserialize;

use crate::math::binomial_distribution;
use crate::Team;

type TeamIndex = usize;

#[derive(Deserialize, Debug, Clone)]
pub struct Component<P> {
    pub r#type: ComponentType,
    pub teams: Vec<P>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum ComponentType {
    // For convenience
    BestOf1,
    BestOf3,
    BestOf5,
    BestOf7,
    BestOfN(usize),
    GroupStage(GroupStage),
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct BestOfN {
    num_games: usize,
}

impl BestOfN {
    fn run(&self, input: &mut [Team], rng: &mut ThreadRng) {
        assert_eq!(input.len(), 2);
        let p = input[0].probability_to_win_against(&input[1]);
        let total: f64 = (0..=(self.num_games - 1) / 2)
            .map(|k| binomial_distribution(p, self.num_games, k))
            .sum();
        if rng.gen_range(0.0..=1.0) > total {
            input.swap(0, 1);
        }
    }
}

impl<P: std::fmt::Debug> Component<P> {
    pub fn run(&self, input: &mut [Team], rng: &mut ThreadRng) {
        match self.r#type {
            ComponentType::BestOf1 => BestOfN { num_games: 1 }.run(input, rng),
            ComponentType::BestOf3 => BestOfN { num_games: 3 }.run(input, rng),
            ComponentType::BestOf5 => BestOfN { num_games: 5 }.run(input, rng),
            ComponentType::BestOf7 => BestOfN { num_games: 7 }.run(input, rng),
            ComponentType::BestOfN(n) => BestOfN { num_games: n }.run(input, rng),
            ComponentType::GroupStage(group) => group.run(input, rng),
        }
    }

    pub fn get_placement_index_from_placement_name(&self, placement: &str) -> usize {
        use ComponentType::*;
        match self.r#type {
            BestOf1 | BestOf3 | BestOf5 | BestOf7 | BestOfN(_) => match placement {
                "winner" => 0,
                "loser" => 1,
                _ => panic!(
                    "Wrong placement name: {} in component: {:?}",
                    placement, self
                ),
            },
            GroupStage(group) => group.get_placement_index_from_placement_name(placement),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct GroupStage {
    num_games_per_series: usize,
}

impl GroupStage {
    fn run(&self, input: &mut [Team], rng: &mut ThreadRng) {
        let mut num_games_won: HashMap<TeamIndex, i32> =
            input.iter().map(|team| (team.index, 0)).collect();
        for (i, team1) in input.iter().enumerate() {
            for team2 in input[i + 1..].iter() {
                for _ in 0..self.num_games_per_series {
                    if team1.wins_against(&team2, rng) {
                        *num_games_won.get_mut(&team1.index).unwrap() += 1;
                    } else {
                        *num_games_won.get_mut(&team2.index).unwrap() += 1;
                    }
                }
            }
        }
        input.sort_by_key(|team| -num_games_won[&team.index]);
        self.tiebreak(&num_games_won, input, rng);
    }

    fn tiebreak(
        &self,
        num_games_won: &HashMap<TeamIndex, i32>,
        input: &mut [Team],
        rng: &mut ThreadRng,
    ) {
        let ties = identify_tied_teams(input, num_games_won);
        if ties.len() > 0 {
            for tie in ties {
                GroupStage {
                    num_games_per_series: self.num_games_per_series,
                }
                .run(&mut input[tie.start_index..=tie.end_index], rng);
            }
        }
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

#[derive(Debug, PartialEq, Eq)]
struct TiedTeams {
    teams: Vec<TeamIndex>,
    start_index: usize,
    end_index: usize,
}

fn identify_tied_teams<'a>(
    teams: &'a [Team],
    num_games_won: &'a HashMap<TeamIndex, i32>,
) -> Vec<TiedTeams> {
    debug_assert!(sorted(teams, num_games_won));
    teams
        .iter()
        .enumerate()
        .group_by(|(_, team)| num_games_won[&team.index])
        .into_iter()
        .map(|(_, group)| group.collect::<Vec<_>>())
        .filter(|group| group.len() > 1)
        .map(|group| TiedTeams {
            start_index: *group.iter().map(|(index, _)| index).min().unwrap(),
            end_index: *group.iter().map(|(index, _)| index).max().unwrap(),
            teams: group.iter().map(|(_, team)| team.index).collect(),
        })
        .collect()
}

fn sorted(teams: &[Team], num_games_won: &HashMap<TeamIndex, i32>) -> bool {
    teams
        .windows(2)
        .all(|ts| num_games_won[&ts[0].index] >= num_games_won[&ts[1].index])
}

#[cfg(test)]
mod tests {
    use super::TiedTeams;
    use crate::Team;

    #[test]
    fn identify_tied_teams() {
        let make_teams = |indices: &[usize]| {
            indices
                .into_iter()
                .map(|index| Team {
                    index: *index,
                    strong: false,
                })
                .collect::<Vec<_>>()
        };
        let teams = make_teams(&[10, 11, 12, 13]);
        let num_games_won = [(10, 2), (11, 2), (12, 2), (13, 2)].into_iter().collect();
        assert_eq!(
            super::identify_tied_teams(&teams, &num_games_won),
            vec![TiedTeams {
                teams: vec![10, 11, 12, 13],
                start_index: 0,
                end_index: 3
            },]
        );
        let teams = make_teams(&[10, 12, 13, 11]);
        let num_games_won = [(10, 3), (12, 2), (13, 2), (11, 1)].into_iter().collect();
        assert_eq!(
            super::identify_tied_teams(&teams, &num_games_won),
            vec![TiedTeams {
                teams: vec![12, 13],
                start_index: 1,
                end_index: 2,
            },]
        );
        let teams = make_teams(&[10, 12, 13, 11]);
        let num_games_won = [(10, 3), (12, 3), (13, 1), (11, 1)].into_iter().collect();
        assert_eq!(
            super::identify_tied_teams(&teams, &num_games_won),
            vec![
                TiedTeams {
                    teams: vec![10, 12],
                    start_index: 0,
                    end_index: 1,
                },
                TiedTeams {
                    teams: vec![13, 11],
                    start_index: 2,
                    end_index: 3,
                },
            ]
        );
    }
}
