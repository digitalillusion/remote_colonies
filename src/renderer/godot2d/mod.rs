pub mod planet;
pub mod ship;
mod game;
mod input;
mod player;
mod starmap;

use gdnative::*;

use std::cell::*;
use std::rc::Rc;
use std::time::SystemTime;

use crate::local::starmap::*;
use crate::local::player::*;
use crate::local::GameState;
use crate::local::model::*;
use self::starmap::Starmap2D;
use self::player::Player2D;
use self::game::Game;

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
    game: Game
}

#[methods]
impl Main {
    
    fn _init(_owner: Node) -> Self {
        Main {
            planet: PackedScene::new(),
            game_state: Rc::new(RefCell::new(GameState::new())),
            game: Game::demo()
        }
    }
    
    #[export]
    unsafe fn _ready(&mut self, mut owner: Node) {
        self.game.start(self.game_state.clone(), || {
            let planet_node: Node2D = instance_scene(&self.planet).unwrap();
            owner.add_child(Some(planet_node.to_node()), false);
            planet_node
        });
    }

    #[export]
    pub unsafe fn _process(&mut self, owner: Node, delta: f64) {
        let start_time = SystemTime::now();
        
        self.perform_update_time(delta);
        
        self.perform_update_ai();

        let (winner, losers) = self.perform_check_game_over(); 
        if self.game.is_demo() && winner.is_some() {
            self.game = Game::demo();
            self._ready(owner);
        } else if !self.game.is_demo() {
            losers.iter();
        }
           
        
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
                Game::perform_action(planets, player, *ai_move);
            });
    }

    fn perform_check_game_over(&self) -> (Option<Rc<Player2D>>, Vec<Rc<Player2D>>) {
        let game_state = self.game_state.borrow();
        game_state.check_game_over()
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