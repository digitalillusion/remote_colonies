use super::model::*;
pub struct PlanetBusiness {
}

impl PlanetBusiness {

    pub fn new()-> Self {
        PlanetBusiness{}
    }

    pub fn resources_update(&self, props: &mut CelestialProperties, orbiters_count: i32) {
        props.resources += props.resources_increase;
        let extracted = orbiters_count as f32;
        let extracted = props.resources.min(extracted);
        props.extracted += extracted;
        props.resources -= extracted;
    }

    pub fn resources_init(&self, props: &mut CelestialProperties, initial: f32, inc: f32) {
        props.resources = initial;
        props.resources_increase = initial * inc;
    }

    pub fn can_add_ship(&self, props: &mut CelestialProperties, contender_props: &ContenderProperties, resources_cost: f32) -> bool {
        if props.contender_id == contender_props.id && props.extracted - resources_cost >= 0.0 {
            props.extracted -= resources_cost;
            return true
        }
        false
    }

    pub fn count_ships_to_move(&self, ships_count: usize, percent: usize) -> usize {
        (ships_count as f32 * percent as f32 / 100.0).floor() as usize
    }
}