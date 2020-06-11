use gdnative::*;

use rand::*;

use std::cell::RefCell;


pub struct PlanetProperties {
    pub owner: RigidBody2D,
    pub id: usize,
    pub radius: f32,
    pub resources: f32,
    pub resources_increase_ratio: f32,
}

#[derive(NativeClass)]
#[inherit(RigidBody2D)]
pub struct Planet {
    properties: RefCell<PlanetProperties>
}

#[methods]
impl Planet {
    
    fn _init(owner: RigidBody2D) -> Planet {
        let mut rng = rand::thread_rng();

        let properties = PlanetProperties {
            owner,
            id: 0,
            radius: 0.0,
            resources: rng.gen_range(10.0, 250.0),
            resources_increase_ratio: 1.0 + rng.gen_range(0.0002, 0.005)
        };
        Planet {
            properties: RefCell::new(properties)
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: RigidBody2D) {
    }

    #[export]
    pub unsafe fn on_resource_timer_timeout(&self, owner: RigidBody2D) {
        let mut props = self.properties.borrow_mut();
        props.resources *= props.resources_increase_ratio;
        
        let label = &format!("{} - {}", props.id.to_string(), (props.resources as usize).to_string());
        let mut planet_label: Label = owner
            .find_node(GodotString::from_str("Label"), false, true)
            .expect("Unable to find planet/Label")
            .cast()
            .expect("Unable to cast to Label");
        planet_label.set_text(GodotString::from_str(label));
    }

    #[export]
    pub unsafe fn on_orbiters_timer_timeout(&self, owner: RigidBody2D) {
        let mut planet_orbiters: Node2D = owner
            .find_node(GodotString::from_str("Orbiters"), false, true)
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        let rotation = planet_orbiters.get_global_rotation();
        planet_orbiters.set_global_rotation(rotation - 0.01);
    }

    pub unsafe fn put_in_orbit(&self, ship_node: &RigidBody2D) {
        let props = self.properties.borrow_mut();
        let mut planet_orbiters: Node2D = props.owner
            .find_node(GodotString::from_str("Orbiters"), false, true)
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        planet_orbiters.add_child(Some(ship_node.to_node()), false);
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

    pub unsafe fn orbit(&self, planet: &Planet) {
        let mut props = self.properties.borrow_mut();
        let planet_props = planet.properties().borrow();
        let planet_orbit_pivot = planet_props.owner.get_global_transform_with_canvas();
        props.owner.set_position(Vector2::new(planet_orbit_pivot.m31 + planet_props.radius + 15.0, planet_orbit_pivot.m32));

        props.owner.apply_central_impulse(Vector2::new(0.0, -10.0));
    }

    pub fn set_id(&mut self, id: usize) {
        let mut props = self.properties.borrow_mut();
        props.id = id;
    }

    pub fn set_resources(&self, initial: f32, inc: f32) {
        let mut props = self.properties.borrow_mut();
        props.resources = initial;
        props.resources_increase_ratio = inc;
    }

    pub unsafe fn with_mut<F>(node: RigidBody2D, mut with_fn: F)
    where
        F: FnMut(&mut Planet) -> ()
    {
        let instance = Instance::<Planet>::try_from_base(node).unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap();
    }


    pub unsafe fn with<F>(node: RigidBody2D, with_fn: F)
    where
        F: Fn(&Planet) -> ()
    {
        let instance = Instance::<Planet>::try_from_base(node).unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap();
    }
    
    
}