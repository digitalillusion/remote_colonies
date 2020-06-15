pub mod planet;
pub mod ship;
mod input;

use gdnative::*;

use planet::*;

use std::cell::*;
use std::rc::Rc;

use crate::local::starmap::*;
use crate::local::player::*;
use crate::local::MainLoop;
use self::input::InputHandler2D;

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
}

#[methods]
impl Main {
    
    fn _init(_owner: Node) -> Self {
        Main {
            planet: PackedScene::new()
        }
    }
    
    #[export]
    unsafe fn _ready(&mut self, mut owner: Node) {
        let main_loop = Rc::new(RefCell::new(MainLoop::new()));
        let input_handler = Rc::new(RefCell::new(InputHandler2D::new()));
        let mut starmap = Starmap::new(10)
        .with_generator(|id| {
            let planet_node: Area2D = instance_scene(&self.planet).unwrap();
            owner.add_child(Some(planet_node.to_node()), false);

            Planet::with_mut(planet_node, |planet| {
                planet.set_main_loop(main_loop.clone());
                planet.set_random_features();
                planet.set_id(id);
                planet.set_input_handler(input_handler.clone(), |planet, player_action| {
                    match player_action {
                        PlayerAction::AddShip => planet.add_ship(10.0, planet.get_player()),
                        PlayerAction::MoveShips(from, to) => godot_print!("{} -> {}", from.id, to.id),
                        _ => ()
                    }
                });
            });

            planet_node
        })
        .with_validator(|planet1, planet2| {
            let distance = distance_between(planet1, planet2);
            distance > 100.0 && distance < 800.0
        })
        .with_cleaner(|planet| planet.free())
        .build();

        starmap.get_planets_by_max_distance(5, |planet1, planet2| {
            distance_between(planet1, planet2)
        }).iter()
        .map(|planet_node| **planet_node)
        .for_each(|planet_node| {
            Planet::with_mut(planet_node, |planet| {
                planet.set_resources(100.0, 0.002);
                planet.add_ship(0.0, None);
            });
        });
        
        let mut main_loop = main_loop.borrow_mut();
        main_loop.set_starmap(starmap);
        main_loop.run();
    }
}

unsafe fn distance_between(planet1: &Area2D, planet2: &Area2D) -> f32 {
    let p1pos = Point2::new(planet1.get_position().x, planet1.get_position().y);
    let p2pos = Point2::new(planet2.get_position().x, planet2.get_position().y);
    p1pos.distance_to(p2pos)
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