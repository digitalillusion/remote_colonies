pub mod builder;

use gdnative::GodotObject;

use builder::StarmapBuilder;

pub struct Starmap<T: GodotObject> {
    pub planets: Vec<T>
}

impl <T: GodotObject> Starmap<T> {
    pub fn new<F, G, H>(count: usize) -> StarmapBuilder<T, F, G, H> 
    where 
        F: FnMut(usize) -> T,
        G: Fn(&T, &T) -> bool,
        H: Fn(&T) -> () 
    {
        StarmapBuilder::new(count)
    }   

    pub fn get_planets_by_max_distance<F>(&self, count: usize, distance_fn: F) -> Vec<&T>
    where
        F: Fn(&T, &T) -> f32,
    {
        let mut max_distance = 0.0;
        let mut first = 0;
        let mut planets:Vec<&T> = self.planets.iter().collect();
        for i in 0..planets.len() {
            for j in i..planets.len() {
                let distance = distance_fn(&planets[i], &planets[j]);
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
                let distance = planets_by_max_distance.iter()
                    .map(|planet| {
                        let distance = distance_fn(&planets[i], &planet);
                        distance.powf(2.0)
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