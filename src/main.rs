mod math;

use std::{collections::HashMap, hash::Hash};

use math::binomial_distribution;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use serde::{Deserialize, Serialize};

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
struct Team(usize);

impl Team {
    fn probability_to_win_against(&self, other: &Self) -> f32 {
        0.5
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
    fn run(&self) -> HashMap<Team, f32> {
        let mut rng = thread_rng();
        let mut teams: HashMap<TeamIdentifier, Team> = self
            .get_team_numbers()
            .map(|num| (TeamIdentifier::Team(num), Team(num)))
            .collect();
        for (component_name, component) in self.components.iter() {
            let teams_this_component = component
                .teams
                .iter()
                .enumerate()
                .map(|(i, team)| teams[&team])
                .collect();
            let outcome = component.run(teams_this_component, &mut rng);
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
}

fn read_tournament(fname: &str) -> Tournament {
    let contents = std::fs::read_to_string(fname).unwrap();
    serde_yaml::from_str(&contents).unwrap()
}

fn main() {
    let t = read_tournament("tournament.yml");
    let mut total_score: HashMap<Team, f32> = HashMap::default();

    for _ in 0..1000000 {
        for (team, score) in t.run() {
            let prev_score = *total_score.get(&team).unwrap_or(&0.0);
            total_score.insert(team, score + prev_score);
        }
    }
}
