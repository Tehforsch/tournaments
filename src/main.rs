mod math;

use std::{collections::HashMap, hash::Hash};

use math::binomial_distribution;
use rand::{rngs::ThreadRng, seq::IteratorRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};

const STRONG_TEAM_ADVANTAGE: f32 = 0.05;

type ComponentName = String;
type PlacementName = usize;
type Placement = (PlacementName, ComponentName);

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
#[serde(untagged)]
enum TeamIdentifier {
    Team(usize),
    FromPreviousComponent(Placement),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Team {
    index: usize,
    strong: bool,
}

impl Team {
    fn probability_to_win_against(&self, other: &Self) -> f32 {
        if self.strong {
            0.5 + STRONG_TEAM_ADVANTAGE
        } else if other.strong {
            0.5 - STRONG_TEAM_ADVANTAGE
        } else {
            0.5
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Component {
    r#type: ComponentType,
    teams: Vec<TeamIdentifier>,
}

#[derive(Serialize, Deserialize, Debug)]
enum ComponentType {
    BestOfN(usize),
}

fn run_best_of_n(n: usize, mut input: Vec<Team>, rng: &mut ThreadRng) -> Vec<Team> {
    assert_eq!(input.len(), 2);
    let p = input[0].probability_to_win_against(&input[1]);
    let total: f32 = (0..=(n - 1) / 2)
        .map(|k| binomial_distribution(p, n, k))
        .sum();
    if rng.gen_range(0.0..=1.0) > total {
        input.swap(0, 1);
    }
    input
}

impl Component {
    fn run(&self, input: Vec<Team>, rng: &mut ThreadRng) -> Vec<Team> {
        match self.r#type {
            ComponentType::BestOfN(n) => run_best_of_n(n, input, rng),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Tournament {
    components: Vec<(ComponentName, Component)>,
    scoring: HashMap<Placement, f32>,
}

impl Tournament {
    fn run(&self, rng: &mut ThreadRng) -> HashMap<Team, f32> {
        let mut teams: HashMap<TeamIdentifier, Team> = self
            .get_team_numbers()
            .map(|index| {
                (
                    TeamIdentifier::Team(index),
                    Team {
                        index,
                        strong: false,
                    },
                )
            })
            .collect();
        teams.iter_mut().choose(rng).unwrap().1.strong = true;
        for (component_name, component) in self.components.iter() {
            let teams_this_component = component
                .teams
                .iter()
                .enumerate()
                .map(|(i, team)| teams[&team])
                .collect();
            let outcome = component.run(teams_this_component, rng);
            for (i, team) in outcome.into_iter().enumerate() {
                teams.insert(
                    TeamIdentifier::FromPreviousComponent((i, component_name.clone())),
                    team,
                );
            }
        }
        self.scoring
            .iter()
            .map(|(placement, score)| {
                (
                    teams[&TeamIdentifier::FromPreviousComponent(placement.clone())],
                    *score,
                )
            })
            .collect()
    }

    fn get_team_numbers(&self) -> impl Iterator<Item = usize> + '_ {
        self.components.iter().flat_map(|(_, component)| {
            component.teams.iter().filter_map(|team| match team {
                TeamIdentifier::Team(num) => Some(*num),
                _ => None,
            })
        })
    }

    pub fn num_teams(&self) -> usize {
        self.get_team_numbers().count()
    }
}

fn read_tournament(fname: &str) -> Tournament {
    let contents = std::fs::read_to_string(fname).unwrap();
    serde_yaml::from_str(&contents).unwrap()
}

fn main() {
    let t = read_tournament("tournament.yml");
    let mut score = 0.0;

    let mut rng = thread_rng();
    let num_runs = 100000;
    for _ in 0..num_runs {
        for (team, s) in t.run(&mut rng) {
            if team.strong {
                score += s;
            }
        }
    }
    let expected_score = 1.0 / t.num_teams() as f32;
    let strong_team_score_advantage = score / num_runs as f32 - expected_score;
    println!(
        "Advantage: {:.3}",
        strong_team_score_advantage / STRONG_TEAM_ADVANTAGE
    );
}
