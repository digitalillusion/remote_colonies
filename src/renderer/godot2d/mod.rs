pub mod planet;
pub mod ship;

use gdnative::*;

use planet::Planet;
use ship::Ship;

use crate::local::starmap::*;
use crate::local::player::*;
use crate::local::MainLoop;

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
    #[property]
    ship: PackedScene,

    main_loop: MainLoop<RigidBody2D, RigidBody2D>,
}


#[methods]
impl Main {
    
    fn _init(_owner: Node) -> Self {
        Main {
            planet: PackedScene::new(),
            ship: PackedScene::new(),
            main_loop: MainLoop::new(),
        }
    }
    
    #[export]
    unsafe fn _ready(&mut self, mut owner: Node) {
        let mut starmap = Starmap::new(10)
            .with_generator(|id| {
                let planet_node: RigidBody2D = instance_scene(&self.planet).unwrap();
                owner.add_child(Some(planet_node.to_node()), false);

                Planet::with_mut(planet_node, |planet| {
                    planet.set_random_features();
                    planet.set_id(id);
                });

                planet_node
            })
            .with_validator(|planet1, planet2| {
                let distance = Main::distance_between(planet1, planet2);
                distance > 100.0 && distance < 800.0
            })
            .with_cleaner(|planet| planet.free())
            .build();

        starmap.get_planets_by_max_distance(2, |planet1, planet2| {
            Main::distance_between(planet1, planet2)
        }).iter().for_each(|planet_node| {
                let planet_node = **planet_node;
                let ship_node: RigidBody2D = instance_scene(&self.ship).unwrap();
                self.main_loop.add_player(Player::new(planet_node, ship_node));

                Planet::with_mut(planet_node, |planet| {
                    planet.set_resources(100.0, 1.002);
                    planet.put_in_orbit(&ship_node);
                    Ship::with(ship_node, |ship| {
                        ship.orbit(planet);
                    });
                });
        });
        
        self.main_loop.set_starmap(starmap);
    }


    unsafe fn distance_between(planet1: &RigidBody2D, planet2: &RigidBody2D) -> f32 {
        let p1pos = Point2::new(planet1.get_position().x, planet1.get_position().y);
        let p2pos = Point2::new(planet2.get_position().x, planet2.get_position().y);
        p1pos.distance_to(p2pos)
    }
}

unsafe fn instance_scene<Root>(scene: &PackedScene) -> Result<Root, ManageErrs>
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
