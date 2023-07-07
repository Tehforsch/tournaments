mod component;
mod math;

use std::collections::HashMap;
use std::env;
use std::hash::Hash;

use component::Component;
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use serde::Deserialize;
use serde::Serialize;

const STRONG_TEAM_ADVANTAGE: f32 = 0.05;

type Score = f32;
type ComponentName = String;
type PlacementName = String;
type NamedPlacement = (PlacementName, ComponentName);

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
struct Placement {
    component: usize,
    position: usize,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum TeamIdentifier<P> {
    Team(usize),
    FromPreviousComponent(P),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Team {
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
struct Tournament {
    components: Vec<(ComponentName, Component<TeamIdentifier<NamedPlacement>>)>,
    scoring: HashMap<TeamIdentifier<NamedPlacement>, Score>,
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

#[derive(Debug, Clone)]
struct Runner {
    placements: Vec<Vec<Team>>,
    components: Vec<Component<Placement>>,
    scoring: Vec<(Placement, Score)>,
}

impl Runner {
    fn named_placement_to_placement(
        tournament: &Tournament,
        team: &TeamIdentifier<NamedPlacement>,
    ) -> Placement {
        let (component_index, placement_index) = match team {
            TeamIdentifier::Team(num) => (0, *num),
            TeamIdentifier::FromPreviousComponent((placement, component)) => {
                let (index, component) = tournament
                    .components
                    .iter()
                    .enumerate()
                    .find(|(_, (comp_name, _))| component == comp_name)
                    .map(|(i, (_, component))| (i, component))
                    .unwrap();
                let placement_index = component.get_placement_index_from_placement_name(placement);
                // The 0-index component is at position 1 in the placements vec because the first entry
                // is the incoming teams.
                (index + 1, placement_index)
            }
        };
        Placement {
            position: placement_index,
            component: component_index,
        }
    }

    fn new(tournament: Tournament) -> Self {
        let components = tournament
            .components
            .iter()
            .map(|(_, comp)| {
                let teams = comp
                    .teams
                    .iter()
                    .map(|team| Self::named_placement_to_placement(&tournament, team))
                    .collect();
                Component {
                    r#type: comp.r#type,
                    teams,
                }
            })
            .collect();

        let scoring = tournament
            .scoring
            .iter()
            .map(|(team, score)| {
                (
                    Self::named_placement_to_placement(&tournament, team),
                    *score,
                )
            })
            .collect();
        Self {
            placements: vec![],
            components,
            scoring,
        }
    }

    fn get_score_of_strong_team(&mut self, teams: Vec<Team>, rng: &mut ThreadRng) -> f32 {
        self.placements.push(teams);
        for component in self.components.iter() {
            let teams_this_component = component
                .teams
                .iter()
                .map(|team| self.placements[team.component][team.position])
                .collect();
            let outcome = component.run(teams_this_component, rng);
            self.placements.push(outcome);
        }
        self.scoring
            .iter()
            .map(|(placement, score)| {
                if self.placements[placement.component][placement.position].strong {
                    *score
                } else {
                    0.0
                }
            })
            .sum()
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

fn run_tournament_for_file(file: &str, rng: &mut ThreadRng) {
    println!("{file}");
    let t = read_tournament(&file);
    let mut score = 0.0;
    let num_teams = t.num_teams();
    let runner = Runner::new(t);
    let num_runs = 10000000;
    for _ in 0..num_runs {
        let mut runner = runner.clone();
        let teams = get_teams(num_teams, rng);
        score += runner.get_score_of_strong_team(teams, rng);
    }
    let expected_score = 1.0 / num_teams as f32;
    let strong_team_score_advantage = score / num_runs as f32 - expected_score;
    println!(
        "Advantage: {:.3}",
        strong_team_score_advantage / STRONG_TEAM_ADVANTAGE
    );
}

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    let mut rng = thread_rng();
    for file in files {
        run_tournament_for_file(&file, &mut rng);
    }
}
