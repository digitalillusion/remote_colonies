use gdnative::Color;

use std::cell::RefCell;

#[derive(Debug, Copy, Clone)]
pub struct ContenderProperties {
    pub id: usize,
    pub color: Color
}

pub trait Contender {
    fn properties(&self) -> &RefCell<ContenderProperties>;
}

#[derive(Debug, Copy, Clone)]
pub struct VesselProperties {
    pub id: usize,
    pub contender_id: usize,
    pub celestial_id: usize,
}

pub trait Vessel {
    fn properties(&self) -> &RefCell<VesselProperties>;
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
    fn properties(&self) -> &RefCell<CelestialProperties>;

}