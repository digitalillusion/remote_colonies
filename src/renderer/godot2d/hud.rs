use gdnative::*;

use std::cell::*;

#[derive(NativeClass)]
#[inherit(Node2D)]
#[user_data(user_data::LocalCellData<HUD>)]
pub struct HUD {
    owner: RefCell<Node2D>
}

#[methods]
impl HUD {
    
    fn _init(owner: Node2D) -> Self {
        HUD {
            owner: RefCell::new(owner)
        }
    }
    
    #[export]
    unsafe fn _ready(&self, owner: Node2D) {
        let mut ais_slider: HSlider = owner
            .get_node(NodePath::from_str("AisSlider"))
            .expect("Unable to find hud/AisSlider")
            .cast()
            .expect("Unable to cast to HSlider");
        ais_slider.set_value(1.0);

        let mut planets_slider: HSlider = owner
            .get_node(NodePath::from_str("PlanetsSlider"))
            .expect("Unable to find hud/PlanetsSlider")
            .cast()
            .expect("Unable to cast to HSlider");
        planets_slider.set_value(10.0);
    }

    #[export]
    pub unsafe fn _on_hud_ais_slider_change(&self, owner: Node2D, value: f64) {
        let mut planets_slider: HSlider = owner
            .get_node(NodePath::from_str("PlanetsSlider"))
            .expect("Unable to find hud/PlanetsSlider")
            .cast()
            .expect("Unable to cast to HSlider");
        
        if planets_slider.get_value() < value + 1.0 {          
            planets_slider.set_value(value as f64);
        }
    }

    #[export]
    pub unsafe fn _on_hud_planets_slider_change(&self, owner: Node2D, value: f64) {
        let mut ais_slider: HSlider = owner
            .get_node(NodePath::from_str("AisSlider"))
            .expect("Unable to find hud/AisSlider")
            .cast()
            .expect("Unable to cast to HSlider");
        
        if ais_slider.get_value() + 1.0 > value {          
            ais_slider.set_value(value as f64);
        }
    }

    #[export]
    pub unsafe fn _on_start_button_up(&self, mut owner: Node2D) {
        let ais_slider: HSlider = owner
            .get_node(NodePath::from_str("AisSlider"))
            .expect("Unable to find hud/AisSlider")
            .cast()
            .expect("Unable to cast to HSlider");
        let planets_slider: HSlider = owner
            .get_node(NodePath::from_str("PlanetsSlider"))
            .expect("Unable to find hud/PlanetsSlider")
            .cast()
            .expect("Unable to cast to HSlider");

        owner.hide();
        owner.get_parent().unwrap().emit_signal("start_game".into(), &[
            Variant::from_u64(ais_slider.get_value() as u64),
            Variant::from_u64(planets_slider.get_value() as u64),
            Variant::from_bool(false),
        ]);
    }

    pub unsafe fn game_over(&self, win: bool) {
        let mut owner = self.owner.borrow_mut();
        owner.show();

        let mut title_label: Label = owner
            .get_node(NodePath::from_str("Title"))
            .expect("Unable to find hud/Title")
            .cast()
            .expect("Unable to cast to Label");
        title_label.set_text(if win { "You win".into() } else { "You lose".into() })
    }

    pub unsafe fn with_mut<F, T>(node: Node2D, mut with_fn: F) -> T
    where
        F: FnMut(&mut HUD) -> T
    {
        let instance = Instance::<HUD>::try_from_base(node).unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap()
    }

    pub unsafe fn with<F, T>(node: Node2D, with_fn: F) -> T
    where
        F: Fn(&HUD) -> T
    {
        let instance = Instance::<HUD>::try_from_base(node).unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap()
    }
}