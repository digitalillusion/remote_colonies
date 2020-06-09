pub mod planet;

use gdnative::*;

use planet::Planet;
use crate::local::starmap::*;

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
    starmap: Option<Starmap<Node2D>>,
    players: List<Player<Node2D>>,
}

#[methods]
impl Main {
    
    fn _init(_owner: Node) -> Self {
        Main {
            planet: PackedScene::new(),
            starmap: None
        }
    }
    
    #[export]
    unsafe fn _ready(&mut self, mut owner: Node) {
        let starmap = Starmap::new(10)
            .with_generator(|id| {
                let planet_node: Node2D = instance_scene(&self.planet).unwrap();
                owner.add_child(Some(planet_node.to_node()), false);

                let planet_instance = Instance::<Planet>::try_from_unsafe_base(planet_node).unwrap();
                planet_instance.map_mut(|planet, planet_owner| {
                    planet.set_random_features(planet_owner);
                    planet.set_id(id);
                }).unwrap();

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
            let planet_instance = Instance::<Planet>::try_from_unsafe_base(**planet_node).unwrap();
                planet_instance.map_mut(|planet, _planet_owner| {
                    planet.set_resources(100.0, 1.02);
                }).unwrap();

            let ship_node: Node2D = instance_scene(&self.planet).unwrap();
            owner.add_child(Some(planet_node.to_node()), false);

            let planet_instance = Instance::<Planet>::try_from_unsafe_base(planet_node).unwrap();
            planet_instance.map_mut(|planet, planet_owner| {
                planet.set_random_features(planet_owner);
                planet.set_id(id);
            }).unwrap();
        });
        self.starmap = Some(starmap);
    }


    unsafe fn distance_between(planet1: &Node2D, planet2: &Node2D) -> f32 {
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
