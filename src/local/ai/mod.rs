use mcts::*;
use mcts::transposition_table::*;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use super::planet::PlanetBusiness;
use super::model::*;
use super::player::*;

#[derive(Clone, Debug)]
struct Measure {
    planet_props: CelestialProperties,
    ships_by_player: Vec<(ContenderProperties, Vec<VesselProperties>)>, 
    resources: f32,
    ships_count: usize
}

#[derive(Clone, Debug, Hash)]
struct Metrics {
    resources: usize,
    ships_count: usize,
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
                resources: 0,
                ships_count: 0
            },
            measures: vec!()
        }
    }

    pub fn refresh_measures(&mut self, ships_by_player_by_planet: Vec<(CelestialProperties, Vec<(ContenderProperties, Vec<VesselProperties>)>)>) {
        let measures = ships_by_player_by_planet.to_vec();
        self.measures = measures.iter()
        .map(|(planet, ships_by_player)| {
            Measure {
                planet_props: *planet,
                ships_by_player: ships_by_player.to_vec(),
                resources: planet.resources + planet.extracted,
                ships_count: ships_by_player.iter()
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

    fn make_move_ships (&self, from: &mut Measure, to: &mut Measure) {
        let planet_business = PlanetBusiness::new();
        let mut allied_ships: Vec<VesselProperties> = from.ships_by_player.iter()
            .find_map(|(player, ships)| {
                if player.id == self.player.id {
                    return Some(ships.to_vec())
                }
                None
            }).unwrap();
        let count: usize = planet_business.count_ships_to_move(allied_ships.len(), Consts::MOVE_SHIP_FLEET_PERCENT);
        let (_, allied_ships_on_planet) = to.ships_by_player.iter_mut()
            .find(|(player, _)| player.id == self.player.id )
            .unwrap();
        allied_ships.drain(0..count)
            .for_each(|allied_ship| allied_ships_on_planet.push(allied_ship));

        let (_, casualties) = planet_business.battle(to.ships_by_player.to_vec());
        from.ships_count -= count;
        to.ships_count += count - casualties.iter()
            .filter(|c| c.contender_id == self.player.id)
            .count();
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
            .filter(|m| m.planet_props.id == self.player.id)
            .collect();
        let enemy_planets: Vec<&Measure> = self.measures.iter()
            .filter(|m| m.planet_props.id != self.player.id)
            .collect();

        allied_planets.iter()
            .for_each(|planet| moves.push(PlayerAction::AddShip(planet.planet_props)));
        for i in 0..allied_planets.len() {
            for j in i..allied_planets.len() {
                moves.push(PlayerAction::MoveShips(allied_planets[i].planet_props, allied_planets[j].planet_props));
            }
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
                measure.resources -= Consts::ADD_SHIP_RESOURCE_COST;
                measure.ships_count += 1;
            },
            PlayerAction::MoveShips(from, to) => {
                let mut measures = self.measures.to_vec();
                let measure_from = measures.iter_mut()
                    .find(|m| m.planet_props.id == from.id).unwrap();
                let mut measures = self.measures.to_vec();
                let measure_to = measures.iter_mut()
                    .find(|m| m.planet_props.id == to.id).unwrap();
                
                self.make_move_ships(measure_from, measure_to);
            },
            PlayerAction::Wait => ()
        }

        self.metrics = Metrics {
            resources: self.measures.iter()
                .map(|m| m.resources)
                .fold(0, |acc, r| acc + r.floor() as usize),
            ships_count: self.measures.iter()
                .map(|m| m.ships_count)
                .fold(0, |acc, s| acc + s),
        }
    }

    
}

impl TranspositionHash for AiState {
    fn hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.metrics.hash(&mut s);
        s.finish()
    }
}