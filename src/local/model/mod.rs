use gdnative::Color;

pub struct Consts {}

impl Consts {
    pub const ADD_PLAYER_RESOURCES_INIT: f32 = 100.0;
    pub const ADD_PLAYER_RESOURCES_INC: f32 = 0.002;
    pub const ADD_SHIP_RESOURCE_COST: f32 = 10.0;
    pub const MOVE_SHIP_FLEET_PERCENT: usize = 50;
}

#[derive(Debug, Copy, Clone)]
pub struct ContenderProperties {
    pub id: usize,
    pub color: Color,
    pub bot: bool
}

pub trait Contender {
    fn properties(&self) -> ContenderProperties;
}

#[derive(Debug, Copy, Clone)]
pub struct VesselProperties {
    pub id: usize,
    pub contender_id: usize,
    pub celestial_id: usize,
}

pub trait Vessel {
    fn properties(&self) -> VesselProperties;
}

#[derive(Debug, Copy, Clone)]
pub struct CelestialProperties {
    pub id: usize,
    pub contender_id: usize,
    pub radius: f32,
    pub resources: f32,
    pub resources_increase: f32,
    pub extracted: f32,
}

pub trait Celestial {
    fn properties(&self) -> CelestialProperties;

}