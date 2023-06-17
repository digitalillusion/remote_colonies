mod game;
pub mod hud;
mod input;
pub mod planet;
mod player;
pub mod ship;
mod starmap;

use gdnative::object::*;
use gdnative::prelude::*;
use gdnative_bindings::*;

use rand::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

use self::game::Game;
use self::hud::HUD;
use self::player::Player2D;
use self::starmap::Starmap2D;
use crate::local::model::*;
use crate::local::player::*;
use crate::local::starmap::*;
use crate::local::GameState;
use crate::renderer::godot2d::hud::RefHUDNode;

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
    planet: Ref<PackedScene>,
    #[property]
    hud: Ref<PackedScene>,

    game_state: Rc<RefCell<GameState<Starmap2D, Player2D>>>,
    game: Game,

    hud_node: Option<RefHUDNode>,
}

#[methods]
impl Main {
    fn new(_owner: &Node) -> Self {
        Main {
            planet: PackedScene::new().into_shared(),
            hud: PackedScene::new().into_shared(),
            game_state: Rc::new(RefCell::new(GameState::new())),
            game: Game::demo(),
            hud_node: None,
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        builder
            .signal("start_game")
            .with_param_default("ais_count", 10.to_variant())
            .with_param_default("planets_count", 15.to_variant())
            .with_param_default("demo", true.to_variant())
            .done();
    }

    #[method]
    fn _ready(&mut self, #[base] owner: &Node) {
        self._on_main_start_game(
            owner,
            self.game.get_ais_count(),
            self.game.get_planets_count(),
            self.game.is_demo(),
        );

        let hud_node: Ref<Node2D, _> = instance_scene(&self.hud);
        let hud_node = hud_node.into_shared();
        owner.add_child(hud_node, false);

        self.hud_node = Some(hud_node);
    }

    #[method]
    pub fn _process(&mut self, #[base] owner: &Node, delta: f64) {
        let start_time = SystemTime::now();

        self.perform_update_time(delta);

        self.perform_update_ai();

        let (winner, losers) = self.perform_check_game_over();
        if self.game.is_demo() && winner.is_some() {
            self._on_main_start_game(
                owner,
                self.game.get_ais_count(),
                self.game.get_planets_count(),
                self.game.is_demo(),
            );
        } else if !self.game.is_demo() {
            let game_state = self.game_state.borrow();
            let current_player = game_state.get_current_player().unwrap();
            if losers
                .iter()
                .any(|l| l.properties().id == current_player.properties().id)
            {
                HUD::with(&self.hud_node.unwrap(), |hud| hud.game_over(false));
            } else if let Some(winner) = winner {
                if winner.properties().id == current_player.properties().id {
                    HUD::with(&self.hud_node.unwrap(), |hud| hud.game_over(true));
                }
            }
        }

        let process_millis = SystemTime::now()
            .duration_since(start_time)
            .unwrap()
            .as_millis();
        let delta_millis = (delta * 1000.0).floor() as u128;
        if delta_millis > 0 && process_millis > 100 * delta_millis {
            godot_print!(
                "WARNING: slow _process() took {} ms (cycle is {} ms)",
                process_millis,
                delta_millis
            );
        }
    }

    #[method]
    pub fn _on_main_start_game(
        &mut self,
        #[base] owner: &Node,
        ais_count: usize,
        planets_count: usize,
        demo: bool,
    ) {
        let background = unsafe {
            owner
                .get_node_as::<AnimatedSprite>("Background")
                .expect("Cannot resolve Background")
        };
        let bg_count = background.sprite_frames().unwrap();
        let bg_count = unsafe { bg_count.assume_safe() }
            .as_ref()
            .get_frame_count("default");
        let mut bg_index = background.get_index();
        while bg_index == background.get_index() {
            bg_index = rand::thread_rng().gen_range(-1..bg_count);
        }
        background.set_frame(bg_index);

        self.game = if demo {
            Game::demo()
        } else {
            Game::new(ais_count, planets_count)
        };
        let planet_create_fn = || {
            let planet_node: Ref<Node2D, _> = instance_scene(&self.planet);
            let planet_node = unsafe { planet_node.into_shared().assume_safe() };
            owner.add_child(planet_node, false);
            planet_node.claim()
        };
        self.game.start(self.game_state.clone(), planet_create_fn);
    }

    fn perform_update_time(&self, delta: f64) {
        let mut game_state = self.game_state.borrow_mut();
        game_state.add_time_delta(delta);
    }

    fn perform_update_ai(&self) {
        let mut game_state = self.game_state.borrow_mut();
        game_state
            .update_ai()
            .iter()
            .for_each(|(ai_player, ai_move)| {
                let planets = game_state.get_starmap().get_planets();
                let player = game_state
                    .get_players()
                    .iter()
                    .find(|p| p.properties().id == ai_player.id)
                    .unwrap();
                Game::perform_action(&planets, player, *ai_move);
            });
    }

    fn perform_check_game_over(&self) -> (Option<Rc<Player2D>>, Vec<Rc<Player2D>>) {
        let game_state = self.game_state.borrow();
        game_state.check_game_over()
    }
}

pub fn instance_scene<Root>(scene: &Ref<PackedScene, Shared>) -> Ref<Root, Unique>
where
    Root: GodotObject<Memory = ManuallyManaged> + SubClass<Node>,
{
    let scene = unsafe { scene.assume_safe() };

    let instance = scene
        .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
        .expect("should be able to instance scene");

    let instance = unsafe { instance.assume_unique() };

    instance
        .try_cast::<Root>()
        .expect("root node type should be correct")
}
