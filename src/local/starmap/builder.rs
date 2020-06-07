use gdnative::GodotObject;

use super::Starmap;

pub struct StarmapBuilder<T: GodotObject, F, G, H>
where 
    F: FnMut(usize) -> T,
    G: Fn(&T, &T) -> bool,
    H: Fn(&T) -> ()
{
    count: usize,
    generator: Option<F>,
    validator: Option<G>,
    cleaner: Option<H>
}

impl <T: GodotObject, F, G, H> StarmapBuilder<T, F, G, H>   
where 
    F: FnMut(usize) -> T,
    G: Fn(&T, &T) -> bool,
    H: Fn(&T) -> ()
{
    pub fn new(count: usize) -> StarmapBuilder<T, F, G, H> {
        StarmapBuilder {
            count,
            generator: None,
            validator: None,
            cleaner: None
        }
    }

    pub fn with_generator(self, generator: F) -> Self {
        StarmapBuilder {
            generator: Some(generator),
            ..self
        }
    }
    pub fn with_validator(self, validator: G) -> Self {
        StarmapBuilder {
            validator: Some(validator),
            ..self
        }
    }
    pub fn with_cleaner(self, cleaner: H) -> Self {
        StarmapBuilder {
            cleaner: Some(cleaner),
            ..self
        }
    }
    pub fn build(self) -> Starmap<T> {
        let mut planets: Vec<T> = vec!();
        let mut planets_invalid_indexes: Vec<usize> = (0..self.count).rev().collect();

        let mut generator = self.generator.unwrap();
        let validator = self.validator.unwrap();
        let cleaner = self.cleaner.unwrap();

        while planets.len() < self.count {
            planets_invalid_indexes.reverse();
            for i in &planets_invalid_indexes {
                let planet = generator(*i);
                planets.insert(*i, planet);
            }

            'remove_invalid : for i in planets_invalid_indexes.clone() {
                for j in 0..planets.len() {
                    if j != i && !validator(&planets[i], &planets[j]) {
                        break 'remove_invalid;
                    }
                }
                planets_invalid_indexes.remove_item(&i);
            }

            planets_invalid_indexes.sort_unstable();
            planets_invalid_indexes.reverse();
            planets_invalid_indexes.iter().for_each(|planet_invalid_index| {
                let planet_invalid = planets.remove(*planet_invalid_index);
                cleaner(&planet_invalid);
            });
        }

        Starmap {
            planets
        }
    }
}