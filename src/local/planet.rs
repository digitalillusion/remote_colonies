use rand::*;

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

    pub fn can_add_ship(&self, props: &mut CelestialProperties, contender_props: ContenderProperties, resources_cost: f32) -> bool {
        if props.contender_id == contender_props.id && props.extracted - resources_cost >= 0.0 {
            props.extracted -= resources_cost;
            return true
        }
        false
    }

    pub fn count_ships_to_move(&self, ships_count: usize, percent: usize) -> usize {
        (ships_count as f32 * percent as f32 / 100.0).floor() as usize
    }

    pub fn battle(&self, ships_by_player: Vec<(ContenderProperties, Vec<VesselProperties>)> ) -> (Option<ContenderProperties>, Vec<VesselProperties>) {
        let total_ship_count = ships_by_player.iter()
            .map(|(_, ships)| ships.len())
            .fold(0, |acc, count| acc + count) as f32;
        let ship_loss_probs: Vec<f32> = ships_by_player.iter()
            .map(|(_, ships)| {
                if total_ship_count == 0.0 {
                    return 0.0
                }
                (total_ship_count - ships.len() as f32) / (total_ship_count * 100.0)
            })
            .collect();

        let mut casualties = vec!();
        let mut rng = rand::thread_rng();
        ship_loss_probs.iter()
            .enumerate()
            .for_each(|(index, prob)| {
                let dice = rng.gen_range(0.0, 1.0);
                let ships = &ships_by_player.get(index).unwrap().1;
                if dice > 1.0 - *prob && ships.len() > 0 {
                    let rnd_index = rng.gen_range(0.0, ships.len() as f32).floor() as usize;
                    let ship_props = *ships.get(rnd_index).unwrap();
                    casualties.push(ship_props);
                }
            });
        
        let mut winner = None;
        let remaining_players: Vec<ContenderProperties> = ships_by_player.iter()
            .filter_map(|(player, ships)| {
                let player_casualties = casualties.iter().filter(|c| c.contender_id == player.id).count();
                if ships.len() - player_casualties > 0 {
                    return Some(*player)
                }
                None
            })
            .collect();
        if remaining_players.len() == 1 {
            winner = Some(*remaining_players.get(0).unwrap());
        }
        (winner, casualties)
    }
}