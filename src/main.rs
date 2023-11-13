mod component;
mod math;
mod runner;

use std::collections::HashMap;
use std::env;
use std::hash::Hash;

use component::Component;
use linked_hash_map::LinkedHashMap;
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use rand::Rng;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use serde::Deserialize;

use crate::runner::Runner;

const STRONG_TEAM_ADVANTAGE: f64 = 0.10;

type Score = f64;
type ComponentName = String;
type PlacementName = String;

#[derive(Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
struct Placement {
    component: usize,
    position: usize,
}

#[derive(Deserialize, Debug, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum TeamIdentifier {
    Team(usize),
    FromPreviousComponent(PlacementName, ComponentName),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Team {
    index: usize,
    strong: bool,
}

impl Team {
    fn probability_to_win_against(&self, other: &Self) -> f64 {
        if self.strong {
            0.5 + STRONG_TEAM_ADVANTAGE
        } else if other.strong {
            0.5 - STRONG_TEAM_ADVANTAGE
        } else {
            0.5
        }
    }

    fn wins_against(&self, other: &Self, rng: &mut ThreadRng) -> bool {
        let p = self.probability_to_win_against(other);
        rng.gen_range(0.0..=1.0) < p
    }
}

#[derive(Deserialize, Debug)]
pub struct Tournament {
    // Linked hash map is used here to preserve order of the components during
    // deserialization.
    components: LinkedHashMap<ComponentName, Component<TeamIdentifier>>,
    scoring: HashMap<TeamIdentifier, Score>,
}

impl Tournament {
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

#[derive(Default, Debug)]
pub struct ScoreResult {
    strong_team: Score,
    all_teams: Score,
}

impl std::iter::Sum for ScoreResult {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = Self::default();
        for item in iter {
            total.strong_team += item.strong_team;
            total.all_teams += item.all_teams;
        }
        total
    }
}

fn read_tournament(fname: &str) -> Tournament {
    let contents = std::fs::read_to_string(fname).unwrap();
    serde_yaml::from_str(&contents).unwrap()
}

fn get_teams(num: usize, rng: &mut ThreadRng) -> Vec<Team> {
    let mut teams: Vec<Team> = (0..num)
        .map(|index| Team {
            index,
            strong: false,
        })
        .collect();
    teams.iter_mut().choose(rng).unwrap().strong = true;
    teams
}

fn run_tournament_for_file(file: &str) {
    println!("{file}");
    let t = read_tournament(&file);
    let num_teams = t.num_teams();
    let runner = Runner::new(t);
    let num_runs = 10000000;
    let score: ScoreResult = (0..num_runs)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let mut runner = runner.clone();
            let teams = get_teams(num_teams, &mut rng);
            runner.get_score_result(teams, &mut rng)
        })
        .sum();
    dbg!(&score);
    let expected_score = score.all_teams / num_runs as f64 / num_teams as f64;
    let strong_team_score_advantage = score.strong_team / num_runs as f64 - expected_score;
    println!(
        "Advantage: {:.3}",
        strong_team_score_advantage / STRONG_TEAM_ADVANTAGE
    );
}

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    for file in files {
        run_tournament_for_file(&file);
    }
}
