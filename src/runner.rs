use rand::rngs::ThreadRng;

use crate::component::Component;
use crate::Placement;
use crate::Score;
use crate::ScoreResult;
use crate::Team;
use crate::TeamIdentifier;
use crate::Tournament;

#[derive(Debug, Clone)]
pub struct Runner {
    placements: Vec<Vec<Team>>,
    components: Vec<Component<Placement>>,
    scoring: Vec<(Placement, Score)>,
}

impl Runner {
    fn named_placement_to_placement(tournament: &Tournament, team: &TeamIdentifier) -> Placement {
        let (component_index, placement_index) = match team {
            TeamIdentifier::Team(num) => (0, *num),
            TeamIdentifier::FromPreviousComponent(placement, component) => {
                let (index, component) = tournament
                    .components
                    .iter()
                    .enumerate()
                    .find(|(_, (comp_name, _))| &component == comp_name)
                    .map(|(i, (_, component))| (i, component))
                    .unwrap_or_else(|| panic!("Component not found: {}", component));
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

    pub fn new(tournament: Tournament) -> Self {
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

    fn run(&mut self, teams: Vec<Team>, rng: &mut ThreadRng) {
        self.placements.push(teams);
        for component in self.components.iter() {
            let mut teams_this_component: Vec<_> = component
                .teams
                .iter()
                .map(|team| self.placements[team.component][team.position])
                .collect();
            component.run(&mut teams_this_component, rng);
            self.placements.push(teams_this_component);
        }
    }

    pub fn get_score_result(&mut self, teams: Vec<Team>, rng: &mut ThreadRng) -> ScoreResult {
        self.run(teams, rng);
        self.scoring
            .iter()
            .map(|(placement, score)| {
                let strong_team_score =
                    if self.placements[placement.component][placement.position].strong {
                        *score
                    } else {
                        0.0
                    };
                ScoreResult {
                    all_teams: *score,
                    strong_team: strong_team_score,
                }
            })
            .sum()
    }
}
