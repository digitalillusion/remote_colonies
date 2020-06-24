use super::model::*;

pub enum PlayerAction {
    AddShip,
    MoveShips(CelestialProperties, CelestialProperties),
    None
}

pub trait Player : Contender {
    type CelestialType;
    type VesselType;

    fn new(id: usize, planet: Self::CelestialType, ship: Self::VesselType) -> Self;
    fn add_ship(&self, ship: Self::VesselType);
    unsafe fn get_ships_on_planet(&self, planet_props: CelestialProperties) -> Vec<VesselProperties>;
}