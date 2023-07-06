mod math;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hash,
};

use math::binomial_distribution;
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;

type ComponentName = String;
type PlacementName = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum TeamIdentifier {
    New(String),
    FromPreviousComponent(PlacementName, ComponentName),
}

struct Team;

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
    let total: f32 = (0..(n - 1) / 2)
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
}

impl Tournament {
    fn run(&self) {
        let teams = self.num_participating_teams();
        for component in self.components.iter() {}
    }

    fn num_participating_teams(&self) -> usize {
        self.components
            .iter()
            .flat_map(|(_, component)| {
                component.teams.iter().map(|team| match team {
                    TeamIdentifier::New(_) => 1,

                    _ => 0,
                })
            })
            .sum()
    }
}

fn read_tournament(fname: &str) -> Tournament {
    let contents = std::fs::read_to_string(fname).unwrap();
    // Keep items in order by not deserializing to a hashmap
    let mapping: Mapping = serde_yaml::from_str(&contents).unwrap();
    let components = mapping
        .into_iter()
        .map(|item| {
            let component_name: ComponentName = serde_yaml::from_value(item.0).unwrap();
            let component: Component = serde_yaml::from_value(item.1).unwrap();
            (component_name, component)
        })
        .collect();
    Tournament { components }
}

fn main() {
    let t = read_tournament("tournament.yml");
    dbg!(&t);
    t.run();
}

#[cfg(test)]
mod tests {
    use crate::{Component, ComponentType, TeamIdentifier, Tournament};

    #[test]
    fn a() {
        dbg!(serde_yaml::to_string(&Tournament {
            components: [(
                "1".into(),
                Component {
                    r#type: ComponentType::BestOfN(1),
                    teams: vec![]
                }
            )]
            .into_iter()
            .collect()
        }));
        panic!();
    }
}
