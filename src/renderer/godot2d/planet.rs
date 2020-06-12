use gdnative::*;

use rand::*;

use std::cell::*;

use crate::local::model::Celestial;
use super::instance_scene;

pub struct PlanetProperties {
    pub owner: Area2D,
    pub id: usize,
    pub radius: f32,
    pub resources: f32,
    pub resources_increase: f32,
}

#[derive(NativeClass)]
#[inherit(Area2D)]
pub struct Planet {

    #[property]
    ship: PackedScene,

    properties: RefCell<PlanetProperties>,
    input_handler: Option<Box<dyn Fn(Box<&Planet>, InputEvent) -> ()>>,
}

impl Celestial for Planet {
    
}

#[methods]
impl Planet {
    
    fn _init(owner: Area2D) -> Planet {
        let mut rng = rand::thread_rng();
        let resources_initial  = rng.gen_range(10.0, 250.0);

        let properties = PlanetProperties {
            owner,
            id: 0,
            radius: 0.0,
            resources: resources_initial,
            resources_increase: resources_initial * rng.gen_range(0.0002, 0.005),
        };
        Planet {
            ship: PackedScene::new(),
            properties: RefCell::new(properties),
            input_handler: None
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: Area2D) {
    }

    #[export]
    pub unsafe fn _on_planet_gui_input(&self, _owner: Area2D, event: InputEvent) {
        self.input_handler.as_ref().unwrap()(Box::new(self), event);
    }

    #[export]
    pub unsafe fn on_resource_timer_timeout(&self, owner: Area2D) {
        let mut props = self.properties.borrow_mut();
        props.resources += props.resources_increase;
        
        let label = &format!("{} - {}", props.id.to_string(), (props.resources as usize).to_string());
        let mut planet_label: Label = owner
            .find_node(GodotString::from_str("Label"), false, true)
            .expect("Unable to find planet/Label")
            .cast()
            .expect("Unable to cast to Label");
        planet_label.set_text(GodotString::from_str(label));
    }

    #[export]
    pub unsafe fn on_orbiters_timer_timeout(&self, owner: Area2D) {
        let mut planet_orbiters: Node2D = owner
            .find_node(GodotString::from_str("Orbiters"), false, true)
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        let rotation = planet_orbiters.get_global_rotation();
        planet_orbiters.set_global_rotation(rotation - 0.01);
    }

    pub unsafe fn add_ship(&self, resources_cost: f32) -> Option<Area2D> {
        let mut props = self.properties.borrow_mut();

        if props.resources - resources_cost < 0.0 {
            return None
        }

        let mut ship_node: Area2D = instance_scene(&self.ship).unwrap();

        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0, 360.0);
        let position = Vector2::new(props.radius + 5.0, 0.0).rotated(Angle::radians(angle));
        ship_node.set_global_rotation(angle.into());
        ship_node.set_position(position);
        
        let mut planet_orbiters: Node2D = props.owner
            .find_node(GodotString::from_str("Orbiters"), false, true)
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        planet_orbiters.add_child(Some(ship_node.to_node()), false);
        
        props.resources -= resources_cost;
        Some(ship_node)
    }

    pub fn properties(&self) -> &RefCell<PlanetProperties> {
        &self.properties
    }

    pub unsafe fn set_random_features(&self) {
        let mut props = self.properties.borrow_mut();

        let viewport_rect: Rect2 = props.owner.get_viewport_rect();
        let viewport_width = viewport_rect.width();
        let viewport_height = viewport_rect.height();

        let mut rng = rand::thread_rng();
        let mut planet_sprite: Sprite = props.owner
            .find_node(GodotString::from_str("Sprite"), false, true)
            .expect("Unable to find planet/Shape")
            .cast()
            .expect("Unable to cast to Sprite");
        let mut planet_collision_shape: CollisionShape2D = props.owner
            .find_node(GodotString::from_str("CollisionShape2D"), false, true)
            .expect("Unable to find planet/CollisionShape2D")
            .cast()
            .expect("Unable to cast to CollisionShape2D");
        let size = planet_sprite.get_texture()
            .expect("Unable to get Texture")
            .get_width() as f32 * 0.5;

        let scale = rng.gen_range(0.2, 1.0) * planet_sprite.get_scale().x;
        let scale_vector = Vector2::new(scale, scale);
        planet_collision_shape.set_scale(scale_vector);
        planet_sprite.set_scale(scale_vector);

        props.radius = scale * size;
        let diameter = 2.0 * props.radius;
        let x_offset = (rng.gen_range(0.0, 1.0) * viewport_width).clamp(diameter, viewport_width - diameter);
        let y_offset = (rng.gen_range(0.0, 1.0) * viewport_height).clamp(diameter, viewport_height - diameter);
        props.owner.set_position(Vector2::new(x_offset, y_offset));
    }

    pub fn set_id(&self, id: usize) {
        let mut props = self.properties.borrow_mut();
        props.id = id;
    }

    pub fn set_input_handler<F: 'static>(&mut self, input_handler: F) 
    where 
        F: Fn(Box<&Planet>, InputEvent) -> ()
    {
        self.input_handler = Some(Box::new(input_handler));
    }

    pub fn set_resources(&self, initial: f32, inc: f32) {
        let mut props = self.properties.borrow_mut();
        props.resources = initial;
        props.resources_increase = initial * inc;
    }

    pub unsafe fn with_mut<F, T>(node: Area2D, mut with_fn: F) -> T
    where
        F: FnMut(&mut Planet) -> T
    {
        let instance = Instance::<Planet>::try_from_base(node).unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap()
    }

    pub unsafe fn with<F, T>(node: Area2D, with_fn: F) -> T
    where
        F: Fn(&Planet) -> T
    {
        let instance = Instance::<Planet>::try_from_base(node).unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap()
    }
}