pub mod planet;
pub mod ship;
pub mod hud;
mod game;
mod input;
mod player;
mod starmap;

use gdnative::*;

use std::cell::*;
use std::rc::Rc;
use std::time::SystemTime;
use rand::*;


use crate::local::starmap::*;
use crate::local::player::*;
use crate::local::GameState;
use crate::local::model::*;
use self::starmap::Starmap2D;
use self::player::Player2D;
use self::game::Game;
use self::hud::HUD;

#[derive(Debug, Clone, PartialEq)]
pub enum ManageErrs {
    CouldNotMakeInstance,
    RootClassInvalid(String),
}

#[derive(NativeClass)]
#[inherit(Node)]
#[user_data(user_data::LocalCellData<Main>)]
#[register_with(Self::register_signals)]
pub struct Main {
    #[property]
    planet: PackedScene,
    #[property]
    hud: PackedScene,

    game_state: Rc<RefCell<GameState<Starmap2D, Player2D>>>,
    game: Game,

    hud_node: Option<Node2D>
}

#[methods]
impl Main {
    
    fn _init(_owner: Node) -> Self {
        Main {
            planet: PackedScene::new(),
            hud: PackedScene::new(),
            game_state: Rc::new(RefCell::new(GameState::new())),
            game: Game::demo(),
            hud_node: None
        }
    }
    
    #[export]
    unsafe fn _ready(&mut self, mut owner: Node) {
        self._on_game_start(owner, self.game.get_ais_count(), self.game.get_planets_count(), self.game.is_demo());
        
        let hud_node: Node2D = instance_scene(&self.hud).unwrap();
        owner.add_child(Some(hud_node.to_node()), false);
        self.hud_node = Some(hud_node);
    }

    #[export]
    pub unsafe fn _process(&mut self, owner: Node, delta: f64) {
        let start_time = SystemTime::now();
        
        self.perform_update_time(delta);
        
        self.perform_update_ai();

        let (winner, losers) = self.perform_check_game_over(); 
        if self.game.is_demo() && winner.is_some() {
            self._on_game_start(owner, self.game.get_ais_count(), self.game.get_planets_count(), self.game.is_demo());
        } else if !self.game.is_demo() {
            let game_state = self.game_state.borrow();
            let current_player = game_state.get_current_player().unwrap();
            if losers.iter()
                .any(|l| l.properties().id == current_player.properties().id) {
                HUD::with(self.hud_node.unwrap(), |hud| hud.game_over(false));
            } else if let Some(winner) = winner {
                if winner.properties().id == current_player.properties().id {
                    HUD::with(self.hud_node.unwrap(), |hud| hud.game_over(true));
                }
            }
        }
        
        let process_millis = SystemTime::now().duration_since(start_time).unwrap().as_millis();
        let delta_millis = (delta * 1000.0).floor() as u128;
        if  delta_millis > 0 && process_millis > 100 * delta_millis  {
            godot_print!("WARNING: slow _process() took {} ms (cycle is {} ms)", process_millis, delta_millis);
        }
    }

    #[export]
    pub unsafe fn _on_game_start(&mut self, mut owner: Node, ais_count: usize, planets_count: usize, demo: bool) { 
        let mut background: AnimatedSprite = owner
            .get_node(NodePath::from_str("Background"))
            .expect("Unable to find planet/Background")
            .cast()
            .expect("Unable to cast to AnimatedSprite");
        let bg_count = background.get_sprite_frames().unwrap().get_frame_count("default".into());
        let mut bg_index = background.get_index();
        while bg_index == background.get_index() {
            bg_index = rand::thread_rng().gen_range(-1, bg_count );
        }
        background.set_frame(bg_index);
        
        self.game = if demo { Game::demo() } else { Game::new(ais_count, planets_count) };
        let planet_create_fn = || {
            let planet_node: Node2D = instance_scene(&self.planet).unwrap();
            owner.add_child(Some(planet_node.to_node()), false);
            planet_node
        };
        self.game.start(self.game_state.clone(), planet_create_fn);
    }

    fn register_signals(builder: &init::ClassBuilder<Self>) {
        builder.add_signal(init::Signal {
            name: "start_game",
            args: &[init::SignalArgument {
                default: Variant::from_u64(10),
                export_info: init::ExportInfo::new(VariantType::I64),
                name: "ais_count",
                usage: init::PropertyUsage::SCRIPT_VARIABLE
            }, init::SignalArgument {
                default: Variant::from_u64(15),
                export_info: init::ExportInfo::new(VariantType::I64),
                name: "planets_count",
                usage: init::PropertyUsage::SCRIPT_VARIABLE
            }, init::SignalArgument {
                default: Variant::from_bool(true),
                export_info: init::ExportInfo::new(VariantType::Bool),
                name: "demo",
                usage: init::PropertyUsage::SCRIPT_VARIABLE
            }],
        });
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