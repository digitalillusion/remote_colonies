use std::cell::*;
use std::rc::Rc;

use gdnative::*;

use super::planet::*;

use crate::local::starmap::*;
use crate::local::GameState;
use crate::local::model::*;
use super::input::InputHandler2D;
use super::starmap::Starmap2D;
use super::player::Player2D;
use super::*;


pub struct Game {
    players_count: usize,
    planets_count: usize,
    demo: bool
}

impl Game {

    pub fn demo() -> Self {
        Game {
            planets_count: 15,
            players_count: 10,
            demo: true
        }
    }

    pub fn is_demo(&self) -> bool {
        self.demo
    }

    pub unsafe fn start<F>(&self, game_state: Rc<RefCell<GameState<Starmap2D, Player2D>>>, mut planet_create_fn: F)
    where 
        F: FnMut() -> Node2D
    {
        game_state.borrow_mut().reset();
        let input_handler = Rc::new(RefCell::new(InputHandler2D::new()));
            let mut starmap = Starmap2D::new(self.planets_count)
            .with_generator(|id| {
                let planet_node = planet_create_fn();
                Planet::with_mut(planet_node, |planet| {
                    planet.set_game_state(game_state.clone());
                    planet.set_random_features();
                    planet.set_id(id);
                    planet.set_input_handler(input_handler.clone(), |planet, player_action| {
                        let game_state = planet.get_game_state();
                        let planets = game_state.get_starmap().get_planets();
                        let current_player = game_state.get_current_player();
                        Game::perform_action(planets, current_player, player_action);
                    });
                });

                planet_node
            })
            .with_validator(|planet1, planet2| {
                let distance = Starmap2D::get_distance_between(planet1, planet2);
                distance > 100.0 && distance < 1000.0
            })
            .with_cleaner(|planet| planet.free())
            .build();

            starmap.get_planets_by_max_distance(self.players_count).iter()
                .map(|planet_node| **planet_node)
                .enumerate()
                .for_each(|(index, planet_node)| {
                    Planet::with_mut(planet_node, |planet| {
                        planet.set_resources(Consts::ADD_PLAYER_RESOURCES_INIT, Consts::ADD_PLAYER_RESOURCES_INC);
                        planet.add_player(index > 0 || self.demo);
                    });
                });
            
            let mut game_state = game_state.borrow_mut();
            game_state.set_starmap(starmap);
    }

    pub unsafe fn perform_action(planets: &Vec<Rc<Node2D>>, player: &Player2D, player_action: PlayerAction) {
        match player_action {
            PlayerAction::AddShip(on) => {
                let planet_on = Planet::get_by_id(planets, on.id);
                Planet::with(**planet_on, |planet| {
                    planet.add_ship(Consts::ADD_SHIP_RESOURCE_COST, player)
                });
            },
            PlayerAction::MoveShips(from, to) => {
                let planet_from = Planet::get_by_id(planets, from.id);
                let planet_to = Planet::get_by_id(planets, to.id);

                Planet::with(**planet_from, |planet| {
                    planet.move_ships(Consts::MOVE_SHIP_FLEET_PERCENT, player, planet_to);
                });
            },
            _ => ()
        }
    }
}