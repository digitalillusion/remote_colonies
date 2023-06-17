pub mod builder;

use super::model::CelestialProperties;
use builder::StarmapBuilder;

pub trait Starmap {
    type CelestialType;

    fn get_planets(&self) -> Vec<Self::CelestialType>;

    fn get_planet_properties(&self, planet_id: usize) -> CelestialProperties;

    fn set_planets(&mut self, planets: Vec<Self::CelestialType>);

    fn new<F, G, H>(count: usize) -> StarmapBuilder<Self::CelestialType, Self, F, G, H>
    where
        F: FnMut(usize) -> Self::CelestialType,
        G: Fn(&Self::CelestialType, &Self::CelestialType) -> bool,
        H: Fn(&Self::CelestialType),
        Self: Sized;

    fn destroy(&self);

    fn get_distance_between(planet1: &Self::CelestialType, planet2: &Self::CelestialType) -> f32
    where
        Self: Sized;

    fn get_planets_by_max_distance(&mut self, count: usize) -> Vec<Self::CelestialType>
    where
        Self: Sized,
    {
        let mut max_distance = 0.0;
        let mut first = 0;
        let mut planets: Vec<Self::CelestialType> = self.get_planets();

        for i in 0..planets.len() {
            for j in i..planets.len() {
                let distance = Self::get_distance_between(&planets[i], &planets[j]);
                if distance > max_distance {
                    max_distance = distance;
                    first = i;
                }
            }
        }

        let mut planets_by_max_distance = vec![planets.remove(first)];

        while planets_by_max_distance.len() < count {
            max_distance = 0.0;
            let mut next = 0;
            for (i, from_planet) in planets.iter().enumerate() {
                let distance: f32 = planets_by_max_distance
                    .iter()
                    .map(|to_planet| {
                        let distance = Self::get_distance_between(from_planet, to_planet);
                        distance.sqrt()
                    })
                    .product();
                if distance > max_distance {
                    max_distance = distance;
                    next = i;
                }
            }
            planets_by_max_distance.push(planets.remove(next));
        }

        planets_by_max_distance
    }
}
