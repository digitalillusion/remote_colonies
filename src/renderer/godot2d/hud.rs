use gdnative::prelude::*;
use gdnative_bindings::HSlider;

pub type RefHUDNode = Ref<Node2D>;

#[derive(NativeClass)]
#[inherit(Node2D)]
#[user_data(user_data::LocalCellData<HUD>)]
pub struct HUD {
    owner: RefHUDNode,
}

#[methods]
impl HUD {
    fn new(owner: &Node2D) -> Self {
        let owner = unsafe { owner.assume_unique() }.cast::<Node2D>().unwrap();
        HUD {
            owner: owner.into_shared(),
        }
    }

    #[method]
    fn _ready(&self, #[base] owner: &Node2D) {
        let ais_slider = unsafe {
            owner
                .get_node_as::<HSlider>("AisSlider")
                .expect("Cannot resolve AisSlider")
        };
        ais_slider.set_value(1.0);

        let planets_slider = unsafe {
            owner
                .get_node_as::<HSlider>("PlanetsSlider")
                .expect("Cannot resolve PlanetsSlider")
        };
        planets_slider.set_value(10.0);
    }

    #[method]
    pub fn _on_hud_ais_slider_change(&self, #[base] owner: &Node2D, value: f64) {
        let planets_slider = unsafe {
            owner
                .get_node_as::<HSlider>("PlanetsSlider")
                .expect("Cannot resolve PlanetsSlider")
        };

        if planets_slider.value() < value + 1.0 {
            planets_slider.set_value(value + 1.0);
        }
    }

    #[method]
    pub fn _on_hud_planets_slider_change(&self, #[base] owner: &Node2D, value: f64) {
        let ais_slider = unsafe {
            owner
                .get_node_as::<HSlider>("AisSlider")
                .expect("Cannot resolve AisSlider")
        };

        if ais_slider.value() > value - 1.0 {
            ais_slider.set_value(value - 1.0);
        }
    }

    #[method]
    pub fn _on_start_button_up(&self, #[base] owner: &Node2D) {
        let ais_slider = unsafe {
            owner
                .get_node_as::<HSlider>("AisSlider")
                .expect("Cannot resolve AisSlider")
        };
        let planets_slider = unsafe {
            owner
                .get_node_as::<HSlider>("PlanetsSlider")
                .expect("Cannot resolve PlanetsSlider")
        };
        let difficulty_slider = unsafe {
            owner
                .get_node_as::<HSlider>("DifficultySlider")
                .expect("Cannot resolve DifficultySlider")
        };

        owner.hide();

        let root_node = unsafe { owner.get_parent().unwrap().assume_safe() }.as_ref();
        root_node.emit_signal(
            "start_game",
            &[
                Variant::new(ais_slider.value() as u64),
                Variant::new(planets_slider.value() as u64),
                Variant::new(difficulty_slider.value() as u64),
                Variant::new(false),
            ],
        );
    }

    pub fn game_over(&self, win: bool) {
        let owner = unsafe { self.owner.assume_safe() }.as_ref();
        owner.show();

        let title_label = unsafe {
            owner
                .get_node_as::<Label>("Title")
                .expect("Cannot resolve Title")
        };
        title_label.set_text(if win { "You win" } else { "You lose" })
    }

    pub fn with_mut<F, T>(base: &RefHUDNode, mut with_fn: F) -> T
    where
        F: FnMut(&mut HUD) -> T,
    {
        let instance = unsafe { base.assume_safe() }.cast_instance().unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap()
    }

    pub fn with<F, T>(base: &RefHUDNode, with_fn: F) -> T
    where
        F: Fn(&HUD) -> T,
    {
        let instance = unsafe { base.assume_safe() }.cast_instance().unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap()
    }
}
