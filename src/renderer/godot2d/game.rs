use std::cell::*;
use std::rc::Rc;

use super::planet::*;

use super::input::InputHandler2D;
use super::player::Player2D;
use super::starmap::Starmap2D;
use super::*;
use crate::local::model::*;
use crate::local::starmap::*;
use crate::local::GameState;

#[derive(Debug, Copy, Clone)]
pub struct Game {
    players_count: usize,
    planets_count: usize,
    difficulty: usize,
    demo: bool,
}

impl Game {
    pub fn demo() -> Self {
        Game {
            planets_count: 15,
            demo: true,
            players_count: 10,
            difficulty: 2,
        }
    }

    pub fn new(ais_count: usize, planets_count: usize, difficulty: usize) -> Self {
        Game {
            planets_count,
            demo: false,
            players_count: ais_count + 1,
            difficulty,
        }
    }

    pub fn is_demo(&self) -> bool {
        self.demo
    }

    pub fn get_ais_count(&self) -> usize {
        self.players_count - 1
    }

    pub fn get_planets_count(&self) -> usize {
        self.planets_count
    }

    pub fn get_difficulty(&self) -> usize {
        self.difficulty
    }

    pub fn start<F>(
        &self,
        game_state: Rc<RefCell<GameState<Starmap2D, Player2D>>>,
        mut planet_create_fn: F,
    ) where
        F: FnMut() -> RefPlanetNode2D,
    {
        game_state.borrow_mut().reset();
        let input_handler = Rc::new(RefCell::new(InputHandler2D::new()));
        let mut starmap = Starmap2D::new(self.planets_count)
            .with_generator(|id| {
                let planet_node = planet_create_fn();
                Planet::with_mut(&planet_node, |planet| {
                    planet.set_game_state(game_state.clone());
                    planet.set_random_features();
                    planet.set_id(id);
                    planet.set_input_handler(input_handler.clone(), |planet, player_action| {
                        let game_state = planet.get_game_state();
                        if let Some(current_player) = game_state.get_current_player() {
                            let planets = game_state.get_starmap().get_planets();
                            Game::perform_action(&planets, current_player, player_action);
                        }
                    });
                });

                planet_node
            })
            .with_validator(|planet1, planet2| {
                let distance = Starmap2D::get_distance_between(planet1, planet2);
                distance > 100.0 && distance < 2000.0
            })
            .with_cleaner(|planet| unsafe { planet.assume_safe().queue_free() })
            .build();

        starmap
            .get_planets_by_max_distance(self.players_count)
            .iter()
            .enumerate()
            .for_each(|(index, planet_node)| {
                Planet::with_mut(planet_node, |planet| {
                    planet.set_resources(
                        Consts::ADD_PLAYER_RESOURCES_INIT,
                        Consts::ADD_PLAYER_RESOURCES_INC,
                    );
                    planet.add_player(index > 0 || self.demo);
                });
            });

        let mut game_state = game_state.borrow_mut();
        game_state.set_starmap(starmap);
    }

    pub fn perform_action(
        planets: &[RefPlanetNode2D],
        player: &Player2D,
        player_action: PlayerAction,
    ) {
        match player_action {
            PlayerAction::AddShip(on) => {
                let planet_on = Planet::get_by_id(planets, on.id);
                Planet::with(planet_on, |planet| {
                    planet.add_ship(Consts::ADD_SHIP_RESOURCE_COST, player)
                });
            }
            PlayerAction::MoveShips(from, to) => {
                let planet_from = Planet::get_by_id(planets, from.id);
                let planet_to = Planet::get_by_id(planets, to.id);

                Planet::with(planet_from, |planet| {
                    planet.move_ships(Consts::MOVE_SHIP_FLEET_PERCENT, player, planet_to);
                });
            }
            _ => (),
        }
    }
}
