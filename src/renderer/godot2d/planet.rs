use gdnative::*;

use rand::*;

#[derive(NativeClass)]
#[inherit(Area2D)]
pub struct Planet;

#[methods]
impl Planet {
    
    fn _init(_owner: Area2D) -> Planet {
        Planet
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: Area2D) {
    }

    pub unsafe fn set_random_features(&self, mut owner: Area2D) {
        let planet_sprite: Sprite = owner
            .find_node(GodotString::from_str("Sprite"), false, true)
            .expect("Unable to find planet/Sprite")
            .cast()
            .expect("Unable to cast to Sprite");

        let viewport_rect: Rect2 = owner.get_viewport_rect();
        let viewport_width = viewport_rect.width();
        let viewport_height = viewport_rect.height();

        let mut rng = rand::thread_rng();
        let size = planet_sprite.get_texture()
            .expect("Unable to get Texture")
            .get_width() as f32;
        let scale = rng.gen_range(0.2, 1.0);
        let border = scale * size * 0.5;
        let x_offset = (rng.gen_range(0.0, 1.0) * viewport_width).clamp(border, viewport_width - border);
        let y_offset = (rng.gen_range(0.0, 1.0) * viewport_height).clamp(border, viewport_height - border);
        
        owner.set_scale(Vector2::new(scale, scale));
        owner.set_position(Vector2::new(x_offset, y_offset));
    }

    pub unsafe fn set_label(&self, text: &str, owner: Area2D) {
        let mut planet_label: Label = owner
            .find_node(GodotString::from_str("Label"), false, true)
            .expect("Unable to find planet/Label")
            .cast()
            .expect("Unable to cast to Label");

        planet_label.set_text(GodotString::from_str(text));
    }
}