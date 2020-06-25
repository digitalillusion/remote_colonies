use std::rc::Rc;

use super::Starmap;

pub struct StarmapBuilder<T, U, F, G, H>
where 
    U: Starmap<CelestialType = T>,
    F: FnMut(usize) -> T,
    G: Fn(&T, &T) -> bool,
    H: Fn(&T) -> ()
{
    count: usize,
    generator: Option<F>,
    validator: Option<G>,
    cleaner: Option<H>,
    target: U,
}

impl <T, U, F, G, H> StarmapBuilder<T, U, F, G, H>   
where 
    U: Starmap<CelestialType = T>,
    F: FnMut(usize) -> T,
    G: Fn(&T, &T) -> bool,
    H: Fn(&T) -> ()
{
    pub fn new(count: usize, target: U) -> StarmapBuilder<T, U, F, G, H> {
        StarmapBuilder {
            target,
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
    pub fn build(mut self) -> U {
        let mut planets: Vec<Rc<T>> = vec!();
        let mut planets_invalid_indexes: Vec<usize> = (0..self.count).rev().collect();

        let mut generator = self.generator.unwrap();
        let validator = self.validator.unwrap();
        let cleaner = self.cleaner.unwrap();

        while planets.len() < self.count {
            planets_invalid_indexes.reverse();
            for i in &planets_invalid_indexes {
                let planet = Rc::new(generator(*i));
                planets.insert(*i, planet);
            }

            'remove_invalid : for i in planets_invalid_indexes.clone() {
                for j in 0..planets.len() {
                    if j != i && !validator(&planets[i], &planets[j]) {
                        break 'remove_invalid;
                    }
                }
                let remove_index = planets_invalid_indexes.iter().position(|pii|  pii == &i).unwrap();
                planets_invalid_indexes.remove(remove_index);
            }

            planets_invalid_indexes.sort_unstable();
            planets_invalid_indexes.reverse();
            planets_invalid_indexes.iter().for_each(|planet_invalid_index| {
                let planet_invalid = planets.remove(*planet_invalid_index);
                cleaner(&planet_invalid);
            });
        }

        self.target.set_planets(planets);

        self.target
    }
}