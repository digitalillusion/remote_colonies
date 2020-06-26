use super::model::*;

#[derive(Copy, Clone, Debug)]
pub enum PlayerAction {
    AddShip(CelestialProperties),
    MoveShips(CelestialProperties, CelestialProperties),
    Wait
}

pub trait Player : Contender {
    type CelestialType;
    type VesselType;

    fn new(id: usize, planet: Self::CelestialType, ship: Self::VesselType, is_bot: bool) -> Self;
    fn add_ship(&self, ship: Self::VesselType);
    unsafe fn get_ships_on_planet(&self, planet: CelestialProperties) -> Vec<VesselProperties>;
}