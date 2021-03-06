mod mcst;

use mcts::*;
use mcts::tree_policy::UCTPolicy;
use mcts::transposition_table::*;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use super::planet::PlanetBusiness;
use super::model::*;
use super::player::*;
use self::mcst::*;

#[derive(Clone, Debug)]
struct Measure {
    planet_props: CelestialProperties,
    ships_by_player: Vec<(ContenderProperties, Vec<VesselProperties>)>, 
    resources: f32,
    extracted: f32,
    ships_count: usize,
    allied_ships_count: usize,
    distances: Vec<f32>,
    distance: f32,
}

#[derive(Clone, Debug, Hash)]
struct Metrics {
    extracted: i64,
    ships_count: i64,
    ships_count_ratio: i64,
    planets_ratio: i64
}

impl Metrics {
    pub fn evaluate(&self) -> i64 {
        let extracted = self.extracted as f32;
        let ships_count = self.ships_count as f32;

        let planets_ratio = self.planets_ratio as f32 * 0.01;
        let ships_count_ratio = self.ships_count_ratio as f32 * 0.01;
        
        let strat_more_ships_when_disadvantage = ships_count * (1.0 - ships_count_ratio);
        let strat_more_conquer_planet_when_advantage = planets_ratio * 10.0f32.powf(ships_count_ratio);
        let benefit = strat_more_ships_when_disadvantage * strat_more_conquer_planet_when_advantage / 1.000001f32.powf(extracted);
        (100.0 * benefit).round() as i64
    }
}

#[derive(Clone, Debug)]
pub struct AiState {
    player: ContenderProperties,

    metrics: Metrics,

    measures: Vec<Measure>
}

impl AiState {
    pub fn new(player: ContenderProperties) -> Self {
        AiState {
            player,
            metrics: Metrics {
                extracted: 0,
                ships_count: 0,
                ships_count_ratio: 0,
                planets_ratio: 0
            },
            measures: vec!()
        }
    }

    pub fn get_player(&self) -> ContenderProperties {
        self.player
    }

    pub fn get_best_move(&self) -> PlayerAction {
        let mut mcts = MCTSManager::new(self.clone(), MyMCTS, MyEvaluator, UCTPolicy::new(0.5), ApproxTable::new(1024));
        mcts.playout_n_parallel(100, 4);
        mcts.best_move().unwrap()
    }

    pub fn refresh_measures(&mut self, planet_distances: &Vec<Vec<f32>>, ships_by_player_by_planet: Vec<(CelestialProperties, Vec<(ContenderProperties, Vec<VesselProperties>)>)>) {
        let measures = ships_by_player_by_planet.to_vec();
        self.measures = measures.iter()
        .enumerate()
        .map(|(planet_id, (planet, ships_by_player))| {
            Measure {
                planet_props: *planet,
                distances: planet_distances.get(planet_id).unwrap().to_vec(),
                distance: f32::INFINITY,
                ships_by_player: ships_by_player.to_vec(),
                resources: planet.resources + planet.extracted,
                extracted: planet.extracted,
                ships_count: ships_by_player.iter()
                    .fold(0, |acc, (_, ships)| acc + ships.len()),
                allied_ships_count: ships_by_player.iter()
                    .fold(0, |acc, (player, ships)| {
                        if player.id == self.player.id {
                            return acc + ships.len()
                        }
                        acc
                    })
            }
        })
        .collect();
    }

    fn make_move_ships (player_id: usize, from: &mut Measure, to: &mut Measure) {
        let planet_business = PlanetBusiness::new();
        let allied_ships: &mut Vec<VesselProperties> = from.ships_by_player.iter_mut()
            .find_map(|(player, ships)| {
                if player.id == player_id {
                    return Some(ships)
                }
                None
            }).unwrap();
        let count: usize = planet_business.count_ships_to_move(allied_ships.len(), Consts::MOVE_SHIP_FLEET_PERCENT);
        if count > 0 {
            let (_, allied_ships_on_planet) = to.ships_by_player.iter_mut()
            .find(|(player, _)| player.id == player_id)
            .unwrap();
            
            allied_ships.drain(0..count)
                .for_each(|allied_ship| allied_ships_on_planet.push(allied_ship));
            
            let (winner, casualties) = planet_business.battle(to.ships_by_player.to_vec());
            from.distance = 0.0;
            to.distance = *to.distances.get(from.planet_props.id).unwrap();
            from.ships_count = from.ships_by_player.iter()
                .fold(0, |acc, (_, ships)| acc + ships.len());
            from.allied_ships_count = from.ships_by_player.iter()
                .fold(0, |acc, (player, ships)| {
                    if player.id == player_id {
                        return acc + ships.len()
                    }
                    acc
                });
            to.ships_count = to.ships_by_player.iter()
                .fold(0, |acc, (_, ships)| acc + ships.len()) - casualties.len();
            to.allied_ships_count = to.ships_by_player.iter()
                .fold(0, |acc, (player, ships)| {
                    if player.id == player_id {
                        return acc + ships.len()
                    }
                    acc
                }) - casualties.iter()
                .filter(|c| c.contender_id == player_id)
                .count();
            if let Some(winner) = winner {
                if winner.id == player_id {
                    to.planet_props.contender_id = winner.id
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
        let mut moves = vec![PlayerAction::Wait];
        let allied_planets: Vec<&Measure> = self.measures.iter()
            .filter(|m| m.planet_props.contender_id == self.player.id)
            .collect();
        let enemy_planets: Vec<&Measure> = self.measures.iter()
            .filter(|m| m.planet_props.contender_id != self.player.id)
            .collect();

        allied_planets.iter()
            .for_each(|planet| moves.push(PlayerAction::AddShip(planet.planet_props)));
        for i in 0..allied_planets.len() {
            for j in (i + 1)..allied_planets.len() {
                moves.push(PlayerAction::MoveShips(allied_planets[i].planet_props, allied_planets[j].planet_props));
            }
        }
        for i in 0..allied_planets.len() {
            for j in 0..enemy_planets.len() {
                moves.push(PlayerAction::MoveShips(allied_planets[i].planet_props, enemy_planets[j].planet_props));
            }
        }

        moves
    }
    fn make_move(&mut self, mov: &Self::Move) {
        match *mov {
            PlayerAction::AddShip(on) => {
                let measure = self.measures.iter_mut()
                    .find(|m| m.planet_props.id == on.id).unwrap();
                measure.extracted -= Consts::ADD_SHIP_RESOURCE_COST;
                measure.distance = 0.0;
                measure.ships_count += 1;
                measure.allied_ships_count += 1;
            },
            PlayerAction::MoveShips(from, to) => {
                let measures = &self.measures;
                let measure_from = measures.iter()
                    .position(|m| m.planet_props.id == from.id).unwrap();
                let measure_to = measures.iter()
                    .position(|m| m.planet_props.id == to.id).unwrap();
                let first_index = if measure_from < measure_to { measure_from } else { measure_to };
                let second_index = if measure_from < measure_to { measure_to } else { measure_from };
                let (head, tail) = self.measures.split_at_mut(first_index + 1);
                let first_measure = &mut head[first_index];
                let second_measure = &mut tail[second_index - first_index - 1];
                if measure_from < measure_to { 
                    Self::make_move_ships(self.player.id, first_measure, second_measure);
                } else {
                    Self::make_move_ships(self.player.id, second_measure, first_measure);
                }
            },
            PlayerAction::Wait => ()
        }

        let measures = &self.measures;
        let allied_measures: Vec<&Measure> = measures.iter()
            .filter(|m| m.planet_props.contender_id == self.player.id)
            .collect();

        let allied_extracted = allied_measures.iter()
            .map(|m| m.extracted)
            .fold(0.0, |acc, r| acc + r.floor());
        let allied_ships_count = measures.iter()
            .map(|m| m.allied_ships_count)
            .fold(0.0, |acc, s| acc + s as f32);
        let total_ships_count = measures.iter()
            .map(|m| m.ships_count)
            .fold(0.0, |acc, s| acc + s as f32);
        self.metrics = Metrics {
            extracted: allied_extracted.floor() as i64,
            ships_count: allied_ships_count.floor() as i64,
            ships_count_ratio: (100.0 * allied_ships_count / total_ships_count).floor() as i64,
            planets_ratio: (100.0 * allied_measures.iter().count() as f32 / measures.iter().count() as f32) as i64
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