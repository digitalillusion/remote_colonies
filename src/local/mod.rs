pub mod ai;
pub mod input;
pub mod model;
pub mod planet;
pub mod player;
pub mod starmap;

use std::rc::Rc;

use self::player::*;
use self::starmap::Starmap;
use crate::local::ai::*;
use crate::local::model::*;

pub struct GameState<T, U>
where
    T: Starmap,
    U: Player,
{
    starmap: Option<T>,
    players: Vec<Rc<U>>,
    ais: Vec<AiState>,
    time: f64,
}

impl<T, U> Default for GameState<T, U>
where
    T: Starmap,
    U: Player,
{
    fn default() -> Self {
        GameState {
            starmap: None,
            players: vec![],
            ais: vec![],
            time: 0.0,
        }
    }
}

impl<T, U> GameState<T, U>
where
    T: Starmap,
    U: Player,
{
    pub fn new() -> GameState<T, U> {
        GameState::default()
    }

    pub fn reset(&mut self) {
        self.players.iter().for_each(|p| p.destroy());
        self.players.clear();
        self.ais.clear();
        if let Some(starmap) = &mut self.starmap {
            starmap.destroy();
        }
        self.time = 0.0;
    }

    pub fn set_starmap(&mut self, starmap: T) {
        self.starmap = Some(starmap);
    }

    pub fn add_player(&mut self, player: Rc<U>) {
        self.players.push(player.clone());
        let player_props = player.properties();
        if player_props.bot {
            self.ais.push(AiState::new(player_props));
        }
    }

    pub fn add_time_delta(&mut self, delta: f64) {
        self.time += delta;
    }

    pub fn get_current_player(&self) -> Option<&Rc<U>> {
        self.players.iter().find(|p| !p.properties().bot)
    }

    pub fn get_starmap(&self) -> &T {
        self.starmap.as_ref().unwrap()
    }

    pub fn get_players(&self) -> &Vec<Rc<U>> {
        &self.players
    }

    pub fn get_ships_by_player_on_planet(
        &self,
        planet: CelestialProperties,
    ) -> Vec<(ContenderProperties, Vec<VesselProperties>)> {
        let mut ships_by_player: Vec<(ContenderProperties, Vec<VesselProperties>)> = vec![];
        self.players.iter().for_each(|player| {
            let ships_on_planet = player.get_ships_on_planet(planet);
            let tuple = (player.properties(), ships_on_planet);
            ships_by_player.push(tuple);
        });
        ships_by_player
    }

    pub fn update_ai(&mut self) -> Vec<(ContenderProperties, PlayerAction)> {
        let mut ai_moves = vec![];
        let mut ships_by_player_by_planet = vec![];
        let starmap = self.starmap.as_ref().unwrap();
        let planets = starmap.get_planets();
        let mut planet_distances = vec![];
        planets
            .iter()
            .enumerate()
            .for_each(|(planet_id, planet_node)| {
                let mut distances = vec![];
                planets.iter().for_each(|pn| {
                    let dist = T::get_distance_between(planet_node, pn);
                    distances.push(dist);
                });
                planet_distances.push(distances);
                let planet_props = starmap.get_planet_properties(planet_id);
                let ships_by_player = self.get_ships_by_player_on_planet(planet_props);
                ships_by_player_by_planet.push((planet_props, ships_by_player));
            });
        self.ais.iter_mut().for_each(|ai| {
            ai.refresh_measures(&planet_distances, ships_by_player_by_planet.to_vec());
            let tuple = (ai.get_player(), ai.get_best_move());
            ai_moves.push(tuple);
        });

        ai_moves
    }

    pub fn check_game_over(&self) -> (Option<Rc<U>>, Vec<Rc<U>>) {
        let playing: Vec<Rc<U>> = self
            .players
            .iter()
            .filter_map(|p| {
                if p.is_playing() {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect();
        let not_playing: Vec<Rc<U>> = self
            .players
            .iter()
            .filter_map(|p| {
                if !p.is_playing() {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect();
        if playing.len() == 1 {
            return (Some(playing.get(0).unwrap().clone()), not_playing);
        }
        (None, not_playing)
    }
}
