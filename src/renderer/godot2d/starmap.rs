use gdnative::prelude::*;

use super::planet::Planet;
use crate::local::model::*;
use crate::local::starmap::builder::StarmapBuilder;
use crate::local::starmap::*;
use crate::renderer::godot2d::planet::RefPlanetNode2D;

pub struct Starmap2D {
    planets: Vec<RefPlanetNode2D>,
}

impl Starmap for Starmap2D {
    type CelestialType = RefPlanetNode2D;

    fn get_planets(&self) -> Vec<RefPlanetNode2D> {
        self.planets.clone()
    }

    fn get_planet_properties(&self, planet_id: usize) -> CelestialProperties {
        let planet_node = self.planets.get(planet_id).unwrap();
        Planet::with(planet_node, |planet| planet.properties())
    }

    fn set_planets(&mut self, planets: Vec<RefPlanetNode2D>) {
        self.planets = planets;
    }

    fn new<F, G, H>(count: usize) -> StarmapBuilder<RefPlanetNode2D, Starmap2D, F, G, H>
    where
        F: FnMut(usize) -> RefPlanetNode2D,
        G: Fn(&RefPlanetNode2D, &RefPlanetNode2D) -> bool,
        H: Fn(&RefPlanetNode2D),
        Self: Sized,
    {
        StarmapBuilder::new(count, Starmap2D { planets: vec![] })
    }

    fn destroy(&self) {
        self.planets
            .iter()
            .for_each(|p| unsafe { p.assume_safe() }.queue_free());
    }

    fn get_distance_between(planet1: &RefPlanetNode2D, planet2: &RefPlanetNode2D) -> f32 {
        let planet1_obj: &Node2D = unsafe { planet1.assume_safe() }.as_ref().cast().unwrap();
        let planet2_obj: &Node2D = unsafe { planet2.assume_safe() }.as_ref().cast().unwrap();
        planet1_obj.position().distance_to(planet2_obj.position())
    }
}
