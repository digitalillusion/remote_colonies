use gdnative::*;

use rand::*;

#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct Planet {
    id: usize,
    resources: f32,
    resources_increase_ratio: f32,
}

#[methods]
impl Planet {
    
    fn _init(_owner: Node2D) -> Planet {
        let mut rng = rand::thread_rng();

        Planet {
            id: 0,
            resources: rng.gen_range(10.0, 250.0),
            resources_increase_ratio: 1.0 + rng.gen_range(0.0002, 0.005)
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: Node2D) {
    }

    pub unsafe fn set_random_features(&self, mut owner: Node2D) {
        let mut planet_shape: Area2D = owner
            .find_node(GodotString::from_str("Shape"), false, true)
            .expect("Unable to find planet/Shape")
            .cast()
            .expect("Unable to cast to Shape");

        let viewport_rect: Rect2 = owner.get_viewport_rect();
        let viewport_width = viewport_rect.width();
        let viewport_height = viewport_rect.height();

        let mut rng = rand::thread_rng();
        let planet_sprite: Sprite = planet_shape
            .find_node(GodotString::from_str("Sprite"), false, true)
            .expect("Unable to find planet/Shape")
            .cast()
            .expect("Unable to cast to Sprite");
        let size = planet_sprite.get_texture()
            .expect("Unable to get Texture")
            .get_width() as f32;
        let scale = rng.gen_range(0.2, 1.0);
        planet_shape.set_scale(Vector2::new(scale, scale));

        let border = scale * size * 0.5;
        let x_offset = (rng.gen_range(0.0, 1.0) * viewport_width).clamp(border, viewport_width - border);
        let y_offset = (rng.gen_range(0.0, 1.0) * viewport_height).clamp(border, viewport_height - border);
        owner.set_position(Vector2::new(x_offset, y_offset));
    }

    #[export]
    pub unsafe fn on_resource_timer_timeout(&mut self, owner: Node2D) {
        self.resources *= self.resources_increase_ratio;
        
        let label = &format!("{} - {}", self.id.to_string(), (self.resources as usize).to_string());
        let mut planet_label: Label = owner
            .find_node(GodotString::from_str("Label"), false, true)
            .expect("Unable to find planet/Label")
            .cast()
            .expect("Unable to cast to Label");
        planet_label.set_text(GodotString::from_str(label));
    }

    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    pub fn set_resources(&mut self, initial: f32, inc: f32) {
        self.resources = initial;
        self.resources_increase_ratio = inc;
    }
}