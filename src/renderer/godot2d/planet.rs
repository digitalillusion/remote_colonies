use gdnative::*;

use rand::*;

use std::cell::*;
use std::rc::Rc;


use crate::local::player::*;
use crate::local::planet::PlanetBusiness;
use crate::renderer::godot2d::ship::Ship;

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

    main_loop: Option<Rc<RefCell<MainLoop<Area2D, RigidBody2D>>>>,
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
    pub unsafe fn _on_ship_arrival(&self, owner: Area2D, ship_node: Node) {
        let props = self.properties.borrow();
        let mut ship_node: RigidBody2D = ship_node.cast().unwrap();
        if ship_node.get_linear_velocity().length() == 0.0 {
            return;
        }
        ship_node.set_linear_velocity(Vector2::new(0.0, 0.0));
        
        Ship::with_mut(ship_node, |ship| {
            ship.orbit(ship_node, owner, props.radius);
        });
    }

    #[export]
    pub unsafe fn _on_resource_timer_timeout(&self, owner: Area2D) {
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
    pub unsafe fn _on_orbiters_timer_timeout(&self, owner: Area2D) {
        let mut planet_orbiters: Node2D = owner
            .find_node(GodotString::from_str("Orbiters"), false, true)
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        let rotation = planet_orbiters.get_global_rotation();
        planet_orbiters.set_global_rotation(rotation - 0.01);
    }

    pub unsafe fn add_ship(&self, resources_cost: f32, player: Option<Rc<Player<Area2D, RigidBody2D>>>) {
        let mut props = self.properties.borrow_mut();
        let owner = self.owner.borrow();

        if self.business.can_add_ship(&mut props, resources_cost) {
            let ship_node: RigidBody2D = instance_scene(&self.ship).unwrap();

            let (player_id, ships_count) = match player {
                Some(player) => {
                    player.add_ship(ship_node);
                    (player.id, player.ships.borrow().len())
                },
                None => {
                    let mut main_loop = self.get_main_loop().borrow_mut();
                    let player = Rc::new(Player::new(main_loop.players.len(), *owner, ship_node));
                    main_loop.players.push(player.clone());
                    (player.clone().id, player.clone().ships.borrow().len())
                }
            };

            Ship::with_mut(ship_node, |ship| {
                ship.set_id(player_id, ships_count);
                ship.orbit(ship_node, *owner, props.radius);
            });
        }
    }

    pub unsafe fn move_ships(&self, percent: usize, player: Option<Rc<Player<Area2D, RigidBody2D>>>, destination: &Rc<Area2D>) {
        if player.is_none() {
            return;
        }
        let player = player.unwrap();
        let mut ships = player.as_ref().ships.borrow_mut();
        let count: usize =  self.business.count_ships_to_move(ships.len(), percent);
        let selected_ships = ships.drain(0..count);
        let mut root_node = self.owner.borrow().get_parent().unwrap();

        for mut ship in selected_ships {
            let position = ship.get_global_position();
            let ship_node = Some(ship.to_node());
            ship.get_parent().unwrap().remove_child(ship_node);
            root_node.add_child(ship_node, false);
            ship.set_global_position(position);
            
            ship.look_at(destination.get_global_position());
            ship.set_linear_velocity((destination.get_global_position() - position).normalize() * 10.0);
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

    pub fn get_main_loop(&self) -> &Rc<RefCell<MainLoop<Area2D, RigidBody2D>>> {
        &self.main_loop.as_ref().unwrap()
    }

    pub unsafe fn get_player(&self) -> Option<Rc<Player<Area2D, RigidBody2D>>> {       
        for player in &self.get_main_loop().borrow().players {
            if player.planets.borrow().iter()
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

    pub fn set_main_loop(&mut self, main_loop: Rc<RefCell<MainLoop<Area2D, RigidBody2D>>>) {
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

    pub unsafe fn get_by_id(planets: &Vec<Rc<Area2D>>, id: usize) -> &Rc<Area2D> {
        planets.iter()
        .find(|p| {
            Planet::with(***p, |planet| planet.properties().borrow().id == id)
        }).unwrap()
    }
}