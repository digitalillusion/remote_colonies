use gdnative::GodotObject;
use gdnative::godot_print;

pub struct Starmap<T: GodotObject> {
    planets : Vec<T>
}

impl <T: GodotObject> Starmap<T> {
    pub fn new<F, G, H>(count: usize, mut generator: F, validator: G, destroy: H) -> Self
    where 
        F: FnMut(&str) -> T,
        G: Fn(&T, &T) -> bool,
        H: Fn(&T) -> ()
    {
        let mut planets: Vec<T> = vec!();
        let mut planets_invalid_indexes: Vec<usize> = vec!();
        let mut planets_invalid: Vec<T> = vec!();
        while planets.len() < count {
            if planets_invalid_indexes.is_empty() {
                for i in 0..count - planets.len() {
                    let planet = generator(&format!("{}", i).to_string());
                    planets.push(planet);
                }
            } else {
                planets_invalid_indexes.reverse();
                for i in &planets_invalid_indexes {
                    let planet = generator(&format!("{}", i).to_string());
                    planets.insert(*i, planet);
                }
            }
            

            for i in 0..planets.len() {
                if planets_invalid_indexes.is_empty() || planets_invalid_indexes.contains(&i) {
                    let mut is_valid = true;
                    for j in 0..planets.len() {
                        if (i != j) {
                            is_valid &= validator(&planets[i], &planets[j]);
                        }
                    }
    
                    if !is_valid && !planets_invalid_indexes.contains(&i) {
                        planets_invalid_indexes.push(i);
                    } else if is_valid && planets_invalid_indexes.contains(&i) {
                        planets_invalid_indexes.remove_item(&i);
                    }
                }
            }

            planets_invalid_indexes.sort();
            planets_invalid_indexes.reverse();
            planets_invalid_indexes.iter().for_each(|planet_invalid_index| {
                planets_invalid.push(planets.remove(*planet_invalid_index));
            });
        }

        planets_invalid.iter().for_each(|planet_invalid| destroy(&planet_invalid));

        Starmap {
            planets
        }
    }
}
