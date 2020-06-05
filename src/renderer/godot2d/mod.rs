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
    starmap: Option<Starmap<Area2D>>,
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
        self.starmap = Some(Starmap::new(
            10, 
            |id| {
                let planet_area2d: Area2D = instance_scene(&self.planet).unwrap();
                owner.add_child(Some(planet_area2d.to_node()), false);

                let planet_instance = Instance::<Planet>::try_from_unsafe_base(planet_area2d).unwrap();
                planet_instance.map(|planet, planet_owner| {
                    planet.set_random_features(planet_owner);
                    planet.set_label(id, planet_owner);
                }).unwrap();

                planet_area2d
            }, 
            |planet1, planet2| {
                let p1pos = Point2::new(planet1.get_position().x, planet1.get_position().y);
                let p2pos = Point2::new(planet2.get_position().x, planet2.get_position().y);
                let distance = p1pos.distance_to(p2pos);
                distance > 100.0 && distance < 800.0
            },
            |planet| planet.free()
        ));
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
