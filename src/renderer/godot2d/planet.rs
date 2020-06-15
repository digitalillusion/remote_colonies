use gdnative::*;

use rand::*;

use std::cell::*;
use std::rc::Rc;

use crate::local::player::*;
use crate::local::planet::PlanetBusiness;

use crate::local::model::*;
use crate::local::MainLoop;
use crate::local::input::InputHandler;
use super::instance_scene;
use super::input::InputHandler2D;

#[derive(NativeClass)]
#[inherit(Area2D)]
pub struct Planet {

    #[property]
    ship: PackedScene,

    main_loop: Option<Rc<RefCell<MainLoop<Area2D>>>>,
    business: PlanetBusiness,
    owner: RefCell<Area2D>,
    properties: RefCell<CelestialProperties>,
    input_handler_fn: Option<Box<dyn Fn(Box<&Planet>, PlayerAction) -> ()>>,
    input_handler: Option<Rc<RefCell<InputHandler2D>>>,
}

impl Celestial for Planet {
    fn properties(&self) -> &RefCell<CelestialProperties> {
        &self.properties
    }
}

#[methods]
impl Planet {
    
    fn _init(owner: Area2D) -> Planet {
        let mut rng = rand::thread_rng();
        let resources_initial  = rng.gen_range(10.0, 250.0);

        let properties = CelestialProperties {
            id: 0,
            radius: 0.0,
            resources: resources_initial,
            resources_increase: resources_initial * rng.gen_range(0.0002, 0.005),
            extracted: 0.0,
        };
        Planet {
            ship: PackedScene::new(),
            owner: RefCell::new(owner),
            properties: RefCell::new(properties),
            input_handler_fn: None,
            input_handler: None,
            main_loop: None,
            business: PlanetBusiness::new()
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: Area2D) {
    }

    #[export]
    pub unsafe fn _on_planet_gui_input(&self, _owner: Area2D, _viewport: Node, event: InputEvent, _shape_idx: isize) {
        let target = Box::new(self);
        let player_action = self.input_handler.as_ref().unwrap().borrow_mut()
            .convert(*self.properties.borrow(), event);
        self.input_handler_fn.as_ref().unwrap()(target, player_action);
    }

    #[export]
    pub unsafe fn on_resource_timer_timeout(&self, owner: Area2D) {
        let mut props = self.properties.borrow_mut();
        let planet_orbiters: Node2D = owner
            .find_node(GodotString::from_str("Orbiters"), false, true)
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        let orbiters_count = planet_orbiters.get_children().len();
        self.business.resources_update(&mut props, orbiters_count);

        let label = &format!("{} - {}/{}", 
            props.id.to_string(), 
            (props.resources as usize).to_string(),
            (props.extracted as usize).to_string()
        );
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

    pub unsafe fn add_ship(&self, resources_cost: f32, player: Option<Rc<Player<Area2D>>>) {
        let mut props = self.properties.borrow_mut();
        let owner = self.owner.borrow();

        if self.business.can_add_ship(&mut props, resources_cost) {
            let mut ship_node: Area2D = instance_scene(&self.ship).unwrap();

            let mut rng = rand::thread_rng();
            let angle = rng.gen_range(0.0, 360.0);
            let position = Vector2::new(props.radius + 5.0, 0.0).rotated(Angle::radians(angle));
            ship_node.set_global_rotation(angle.into());
            ship_node.set_position(position);
            
            let mut planet_orbiters: Node2D = owner
                .find_node(GodotString::from_str("Orbiters"), false, true)
                .expect("Unable to find planet/Orbiters")
                .cast()
                .expect("Unable to cast to Node2D");
            planet_orbiters.add_child(Some(ship_node.to_node()), false);
            
            match player {
                Some(player) => player.add_ship(ship_node),
                None => {
                    let player = Player::new(*owner, ship_node);
                    self.get_main_loop().borrow_mut().players.push(Rc::new(player));
                }
            }
        }
    }

    pub unsafe fn set_random_features(&self) {
        let mut props = self.properties.borrow_mut();
        let mut owner = self.owner.borrow_mut();

        let viewport_rect: Rect2 = owner.get_viewport_rect();
        let viewport_width = viewport_rect.width();
        let viewport_height = viewport_rect.height();

        let mut rng = rand::thread_rng();
        let mut planet_sprite: Sprite = owner
            .find_node(GodotString::from_str("Sprite"), false, true)
            .expect("Unable to find planet/Shape")
            .cast()
            .expect("Unable to cast to Sprite");
        let size = planet_sprite.get_texture()
            .expect("Unable to get Texture")
            .get_width() as f32 * 0.5;

        let scale = rng.gen_range(0.2, 1.0) * planet_sprite.get_scale().x;
        let scale_vector = Vector2::new(scale, scale);
        planet_sprite.set_scale(scale_vector);

        props.radius = scale * size;
        let diameter = 2.0 * props.radius;
        let x_offset = (rng.gen_range(0.0, 1.0) * viewport_width).clamp(diameter, viewport_width - diameter);
        let y_offset = (rng.gen_range(0.0, 1.0) * viewport_height).clamp(diameter, viewport_height - diameter);
        owner.set_position(Vector2::new(x_offset, y_offset));
    }

    pub fn set_id(&self, id: usize) {
        let mut props = self.properties.borrow_mut();
        props.id = id;
    }

    pub fn get_main_loop(&self) -> &Rc<RefCell<MainLoop<Area2D>>> {
        &self.main_loop.as_ref().unwrap()
    }

    pub unsafe fn get_player(&self) -> Option<Rc<Player<Area2D>>> {       
        for player in &self.get_main_loop().borrow().players {
            if player.planets.iter()
                .find(|p| {
                    Planet::with(**p, |planet| {
                        planet.properties().borrow().id == self.properties().borrow().id    
                    })
                }).is_some() {
                return Some(player.clone())
            }
        }
        None
    }

    pub fn set_input_handler<F: 'static>(&mut self, input_handler: Rc<RefCell<InputHandler2D>>, input_handler_fn: F) 
    where 
        F: Fn(Box<&Planet>, PlayerAction) -> ()
    {
        self.input_handler = Some(input_handler);
        self.input_handler_fn = Some(Box::new(input_handler_fn));
    }

    pub fn set_main_loop(&mut self, main_loop: Rc<RefCell<MainLoop<Area2D>>>) {
        self.main_loop = Some(main_loop);
    }

    pub fn set_resources(&self, initial: f32, inc: f32) {
        let mut props = self.properties.borrow_mut();
        
        self.business.resources_init(&mut props, initial, inc);
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