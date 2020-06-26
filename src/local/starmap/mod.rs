pub mod builder;

use std::rc::Rc;

use builder::StarmapBuilder;
use super::model::CelestialProperties;

pub trait Starmap {
    type CelestialType;

    fn get_planets(&self) -> &Vec<Rc<Self::CelestialType>>;

    unsafe fn get_planet_properties(&self, planet_id: usize) -> CelestialProperties;

    fn set_planets(&mut self, planets: Vec<Rc<Self::CelestialType>>);

    fn new<F, G, H>(count: usize) -> StarmapBuilder<Self::CelestialType, Self, F, G, H> 
    where 
        F: FnMut(usize) -> Self::CelestialType,
        G: Fn(&Self::CelestialType, &Self::CelestialType) -> bool,
        H: Fn(&Self::CelestialType) -> (),
        Self: Sized;  
    
    unsafe fn get_distance_between(planet1: &Self::CelestialType, planet2: &Self::CelestialType) -> f32 
    where 
        Self: Sized;

    unsafe fn get_planets_by_max_distance(&mut self, count: usize) -> Vec<Rc<Self::CelestialType>>
    where 
        Self: Sized
    {
        let mut max_distance = 0.0;
        let mut first = 0;
        let mut planets:Vec<Rc<Self::CelestialType>> = vec!();
        self.get_planets().iter().for_each(|p| planets.push(p.clone()));

        for i in 0..planets.len() {
            for j in i..planets.len() {
                let distance = Self::get_distance_between(planets[i].as_ref(), planets[j].as_ref());
                if distance > max_distance {
                    max_distance = distance;
                    first = i;
                }
            }
        }

        let mut planets_by_max_distance = vec!(planets.remove(first));
        
        while planets_by_max_distance.len() < count {
            max_distance = 0.0;
            let mut next = 0;
            for i in 0..planets.len() {
                let distance: f32 = planets_by_max_distance.iter()
                    .map(|planet| {
                        let distance = Self::get_distance_between(planets[i].as_ref(), planet.as_ref());
                        distance.sqrt()
                    }).product();
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