pub mod planet;
pub mod ship;
mod input;
mod player;
mod starmap;

use gdnative::*;

use planet::*;

use std::cell::*;
use std::rc::Rc;
use std::time::SystemTime;

use crate::local::starmap::*;
use crate::local::player::*;
use crate::local::GameState;
use crate::local::model::*;
use self::input::InputHandler2D;
use self::starmap::Starmap2D;
use self::player::Player2D;

#[derive(Debug, Clone, PartialEq)]
pub enum ManageErrs {
    CouldNotMakeInstance,
    RootClassInvalid(String),
}

#[derive(NativeClass)]
#[inherit(Node)]
#[user_data(user_data::LocalCellData<Main>)]
pub struct Main {
    #[property]
    planet: PackedScene,

    game_state: Rc<RefCell<GameState<Starmap2D, Player2D>>>,
}

#[methods]
impl Main {
    
    fn _init(_owner: Node) -> Self {
        Main {
            planet: PackedScene::new(),
            game_state: Rc::new(RefCell::new(GameState::new())),
        }
    }
    
    #[export]
    unsafe fn _ready(&mut self, mut owner: Node) {
        let input_handler = Rc::new(RefCell::new(InputHandler2D::new()));
        let mut starmap = Starmap2D::new(15)
        .with_generator(|id| {
            let planet_node: Node2D = instance_scene(&self.planet).unwrap();
            owner.add_child(Some(planet_node.to_node()), false);

            Planet::with_mut(planet_node, |planet| {
                planet.set_game_state(self.game_state.clone());
                planet.set_random_features();
                planet.set_id(id);
                planet.set_input_handler(input_handler.clone(), |planet, player_action| {
                    let game_state = planet.get_game_state();
                    let planets = game_state.get_starmap().get_planets();
                    let current_player = game_state.get_current_player();
                    Main::perform_action(planets, current_player, player_action);
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

        starmap.get_planets_by_max_distance(2).iter()
        .map(|planet_node| **planet_node)
        .enumerate()
        .for_each(|(index, planet_node)| {
            Planet::with_mut(planet_node, |planet| {
                planet.set_resources(Consts::ADD_PLAYER_RESOURCES_INIT, Consts::ADD_PLAYER_RESOURCES_INC);
                planet.add_player(index > 0);
            });
        });
        
        let mut game_state = self.game_state.borrow_mut();
        game_state.set_starmap(starmap);
    }

    #[export]
    pub unsafe fn _process(&mut self, _owner: Node, delta: f64) {
        let start_time = SystemTime::now();
        
        self.perform_update_time(delta);
        
        self.perform_update_ai();
           
        
        let process_millis = SystemTime::now().duration_since(start_time).unwrap().as_millis();
        let delta_millis = (delta * 1000.0).floor() as u128;
        if  process_millis > 100 * delta_millis  {
            godot_print!("WARNING: slow _process() took {} ms (cycle is {} ms)", process_millis, delta_millis);
        }
    }

    fn perform_update_time(&self, delta: f64) {
        let mut game_state = self.game_state.borrow_mut();
        game_state.add_time_delta(delta);
    }

    unsafe fn perform_update_ai(&self) -> () {
        let mut game_state = self.game_state.borrow_mut();
        game_state.update_ai().iter()
            .for_each(|(ai_player, ai_move)| {
                let planets = game_state.get_starmap().get_planets();
                let player = game_state.get_players().iter()
                    .find(|p| p.properties().id == ai_player.id)
                    .unwrap();
                Main::perform_action(planets, player, *ai_move);
            });
    }

    unsafe fn perform_action(planets: &Vec<Rc<Node2D>>, player: &Player2D, player_action: PlayerAction) {
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

pub unsafe fn instance_scene<Root>(scene: &PackedScene) -> Result<Root, ManageErrs>
where
    Root: gdnative::GodotObject,
{
    let inst_option = scene.instance(PackedScene::GEN_EDIT_STATE_DISABLED);

    if let Some(instance) = inst_option {
        if let Some(instance_root) = instance.cast::<Root>() {
            Ok(instance_root)
        } else {
            Err(ManageErrs::RootClassInvalid(
                instance.get_name().to_string(),
            ))
        }
    } else {
        Err(ManageErrs::CouldNotMakeInstance)
    }
}