mod evaluator;
mod mcts;

use crate::local::Difficulty;
use mcts::transposition_table::*;
use mcts::tree_policy::UCTPolicy;
use mcts::{GameState, MctsManager};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use self::evaluator::*;
use super::model::*;
use super::planet::PlanetBusiness;
use super::player::*;

type ContenderVessels = (ContenderProperties, Vec<VesselProperties>);

#[derive(Clone, Debug)]
struct Measure {
    planet_props: CelestialProperties,
    ships_by_player: Vec<ContenderVessels>,
    extracted: f32,
    ships_count: usize,
    allied_ships_count: usize,
    distances: Vec<f32>,
    distance: f32,
}

#[derive(Clone, Debug, Hash)]
struct Metrics {
    enemy_extracted: i64,
    allied_extracted: i64,
    ships_count: i64,
    ships_count_ratio: i64,
    planets_ratio: i64,
}

impl Metrics {
    pub fn evaluate(&self) -> i64 {
        let allied_extracted = self.allied_extracted as f32;
        let enemy_extracted = self.enemy_extracted as f32;
        let ships_count = self.ships_count as f32;

        let planets_ratio = self.planets_ratio as f32 * 0.01;
        let ships_count_ratio = self.ships_count_ratio as f32 * 0.01;

        let strat_more_ships_when_disadvantage =
            10.0f32.powf(ships_count) * (1.0 - ships_count_ratio);
        let strat_more_conquer_planet_when_advantage =
            10.0f32.powf(planets_ratio) * ships_count_ratio;
        let strat_consume_own_and_conquer_most_extracted =
            planets_ratio * ships_count_ratio * 10.0f32.powf(enemy_extracted)
                / 1.0001f32.powf(allied_extracted);
        let benefit = strat_more_ships_when_disadvantage
            * strat_more_conquer_planet_when_advantage
            * strat_consume_own_and_conquer_most_extracted;
        (100.0 * benefit).round() as i64
    }
}

#[derive(Clone, Debug)]
pub struct AiState {
    player: ContenderProperties,
    metrics: Metrics,
    measures: Vec<Measure>,
    difficulty: Difficulty,
}

impl AiState {
    pub fn new(player: ContenderProperties, difficulty: Difficulty) -> Self {
        AiState {
            player,
            metrics: Metrics {
                enemy_extracted: 0,
                allied_extracted: 0,
                ships_count: 0,
                ships_count_ratio: 0,
                planets_ratio: 0,
            },
            measures: vec![],
            difficulty,
        }
    }

    pub fn get_player(&self) -> ContenderProperties {
        self.player
    }

    pub fn get_best_move(&self) -> PlayerAction {
        let mut mcts = MctsManager::new(
            self.clone(),
            MyMcts,
            MyEvaluator,
            UCTPolicy::new(match &self.difficulty {
                Difficulty::Easy => 0.2,
                Difficulty::Medium => 0.6,
                Difficulty::Hard => 1.0,
            }),
            ApproxTable::new(2048),
        );
        mcts.playout_n(512);
        mcts.best_move().unwrap_or(PlayerAction::Wait)
    }

    pub fn refresh_measures(
        &mut self,
        planet_distances: &[Vec<f32>],
        players: &[ContenderProperties],
        ships_by_player_by_planet: Vec<(CelestialProperties, Vec<ContenderVessels>)>,
    ) {
        let measures = ships_by_player_by_planet.to_vec();
        self.measures = measures
            .iter()
            .enumerate()
            .map(|(planet_id, (planet, ships_by_player))| {
                let ships_by_player =
                    if planet.contender_id == usize::MAX || planet.contender_id == self.player.id {
                        ships_by_player.clone()
                    } else {
                        let enemy_ships_from_extracted = (0..(planet.extracted as usize))
                            .step_by(Consts::ADD_SHIP_RESOURCE_COST as usize)
                            .map(|_| VesselProperties {
                                id: usize::MAX,
                                contender_id: planet.contender_id,
                                celestial_id: planet_id,
                            })
                            .collect();
                        let enemy_ships_from_extracted = [(
                            *players
                                .iter()
                                .find(|player| player.id == planet.contender_id)
                                .unwrap(),
                            enemy_ships_from_extracted,
                        )]
                        .to_vec();
                        [ships_by_player.clone(), enemy_ships_from_extracted].concat()
                    };
                Measure {
                    planet_props: *planet,
                    distances: planet_distances.get(planet_id).unwrap().to_vec(),
                    distance: f32::INFINITY,
                    ships_by_player: ships_by_player.clone(),
                    extracted: planet.extracted,
                    ships_count: ships_by_player
                        .iter()
                        .fold(0, |acc, (_, ships)| acc + ships.len()),
                    allied_ships_count: ships_by_player.iter().fold(0, |acc, (player, ships)| {
                        if player.id == self.player.id {
                            return acc + ships.len();
                        }
                        acc
                    }),
                }
            })
            .collect();
    }

    fn make_move_ships(player_id: usize, from: &mut Measure, to: &mut Measure) {
        let planet_business = PlanetBusiness::new();
        let allied_ships: &mut Vec<VesselProperties> = from
            .ships_by_player
            .iter_mut()
            .find_map(|(player, ships)| {
                if player.id == player_id {
                    return Some(ships);
                }
                None
            })
            .unwrap();
        let count: usize = planet_business
            .count_ships_to_move(allied_ships.len(), Consts::MOVE_SHIP_FLEET_PERCENT);
        if count > 0 {
            let (_, allied_ships_on_planet) = to
                .ships_by_player
                .iter_mut()
                .find(|(player, _)| player.id == player_id)
                .unwrap();

            allied_ships
                .drain(0..count)
                .for_each(|allied_ship| allied_ships_on_planet.push(allied_ship));

            let (winner, casualties) = planet_business.battle(to.ships_by_player.to_vec());
            from.distance = 0.0;
            to.distance = *to.distances.get(from.planet_props.id).unwrap();
            from.ships_count = from
                .ships_by_player
                .iter()
                .fold(0, |acc, (_, ships)| acc + ships.len());
            from.allied_ships_count =
                from.ships_by_player.iter().fold(0, |acc, (player, ships)| {
                    if player.id == player_id {
                        return acc + ships.len();
                    }
                    acc
                }) - count;
            to.ships_count = to
                .ships_by_player
                .iter()
                .fold(0, |acc, (_, ships)| acc + ships.len())
                - casualties.len();
            to.allied_ships_count = to.ships_by_player.iter().fold(0, |acc, (player, ships)| {
                if player.id == player_id {
                    return acc + ships.len();
                }
                acc
            }) - casualties
                .iter()
                .filter(|c| c.contender_id == player_id)
                .count();
            if let Some(winner) = winner {
                if winner.id == player_id {
                    to.planet_props.contender_id = winner.id;
                }
            }
        }
    }
}

impl GameState for AiState {
    type Move = PlayerAction;
    type Player = ContenderProperties;
    type MoveList = Vec<PlayerAction>;

    fn current_player(&self) -> Self::Player {
        self.player
    }
    fn available_moves(&self) -> Vec<PlayerAction> {
        let mut moves = vec![];
        let allied_planets: Vec<&Measure> = self
            .measures
            .iter()
            .filter(|m| m.planet_props.contender_id == self.player.id)
            .collect();
        let enemy_planets: Vec<&Measure> = self
            .measures
            .iter()
            .filter(|m| m.planet_props.contender_id != self.player.id)
            .collect();

        if allied_planets
            .iter()
            .all(|planet| planet.extracted < Consts::ADD_SHIP_RESOURCE_COST)
        {
            moves.push(PlayerAction::Wait);
        }

        allied_planets.iter().for_each(|planet| {
            if planet.extracted > Consts::ADD_SHIP_RESOURCE_COST {
                moves.push(PlayerAction::AddShip(planet.planet_props))
            }
        });
        for i in 0..allied_planets.len() {
            for j in (i + 1)..allied_planets.len() {
                let allied_ships_on_planet: usize = allied_planets[i]
                    .ships_by_player
                    .iter()
                    .filter_map(|(player, ships)| {
                        if player.id == allied_planets[i].planet_props.contender_id {
                            Some(ships.len())
                        } else {
                            None
                        }
                    })
                    .sum();
                let enemy_ships_on_planet: usize = allied_planets[i]
                    .ships_by_player
                    .iter()
                    .filter_map(|(player, ships)| {
                        if player.id != allied_planets[i].planet_props.contender_id {
                            Some(ships.len())
                        } else {
                            None
                        }
                    })
                    .sum();
                if allied_ships_on_planet > 1 && enemy_ships_on_planet < 2 * allied_ships_on_planet
                {
                    moves.push(PlayerAction::MoveShips(
                        allied_planets[i].planet_props,
                        allied_planets[j].planet_props,
                    ));
                }
            }
        }
        for allied_planet in &allied_planets {
            for enemy_planet in &enemy_planets {
                moves.push(PlayerAction::MoveShips(
                    allied_planet.planet_props,
                    enemy_planet.planet_props,
                ));
            }
        }

        moves
    }
    fn make_move(&mut self, mov: &Self::Move) {
        match *mov {
            PlayerAction::AddShip(on) => {
                let measure = self
                    .measures
                    .iter_mut()
                    .find(|m| m.planet_props.id == on.id)
                    .unwrap();
                measure.extracted -= Consts::ADD_SHIP_RESOURCE_COST;
                measure.distance = 0.1;
                measure.ships_count += 1;
                measure.allied_ships_count += 1;
            }
            PlayerAction::MoveShips(from, to) => {
                let measures = &self.measures;
                let measure_from = measures
                    .iter()
                    .position(|m| m.planet_props.id == from.id)
                    .unwrap();
                let measure_to = measures
                    .iter()
                    .position(|m| m.planet_props.id == to.id)
                    .unwrap();
                let first_index = if measure_from < measure_to {
                    measure_from
                } else {
                    measure_to
                };
                let second_index = if measure_from < measure_to {
                    measure_to
                } else {
                    measure_from
                };
                let (head, tail) = self.measures.split_at_mut(first_index + 1);
                let first_measure = &mut head[first_index];
                let second_measure = &mut tail[second_index - first_index - 1];
                if measure_from < measure_to {
                    Self::make_move_ships(self.player.id, first_measure, second_measure);
                } else {
                    Self::make_move_ships(self.player.id, second_measure, first_measure);
                }
            }
            PlayerAction::Wait => (),
        }

        let measures = &self.measures;
        let allied_measures: Vec<&Measure> = measures
            .iter()
            .filter(|m| m.planet_props.contender_id == self.player.id)
            .collect();
        let enemy_measures: Vec<&Measure> = measures
            .iter()
            .filter(|m| m.planet_props.contender_id != self.player.id)
            .collect();

        let allied_extracted = allied_measures
            .iter()
            .map(|m| m.extracted as f32)
            .fold(0.0, |acc, r| acc + r.floor());
        let enemy_extracted = enemy_measures
            .iter()
            .map(|m| (m.planet_props.extracted + m.planet_props.resources) / m.distance)
            .fold(0.0, |acc, r| acc + r.floor());
        let allied_ships_count = measures
            .iter()
            .map(|m| m.allied_ships_count as f32)
            .fold(0.0, |acc, s| acc + s);
        let total_ships_count = measures
            .iter()
            .map(|m| m.ships_count)
            .fold(0.0, |acc, s| acc + s as f32);
        self.metrics = Metrics {
            allied_extracted: allied_extracted.floor() as i64,
            enemy_extracted: enemy_extracted.floor() as i64,
            ships_count: allied_ships_count.floor() as i64,
            ships_count_ratio: (100.0 * allied_ships_count / total_ships_count).floor() as i64,
            planets_ratio: (100.0 * allied_measures.len() as f32 / measures.len() as f32) as i64,
        };
    }
}

impl TranspositionHash for AiState {
    fn hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.metrics.hash(&mut s);
        s.finish()
    }
}
