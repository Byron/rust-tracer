/// Implements a group of intersectable items

use std::num::Float;
use super::primitive::{DistanceMeasure, Intersectable, Intersection, Hit, Ray, Sphere};

#[derive(Debug)]
pub enum Pair<T, G> {
    Item(T),
    Group(G),
}


/// A group with static dispatch on intersect calls, but dynamically allocated 
/// array of items.
#[derive(Debug, Default)]
pub struct TypedGroup<T: Float, B: DistanceMeasure<T>, I: Intersectable<T>> {
    pub bound: B,
    pub children: Vec<Pair<I, TypedGroup<T, B, I>>>,
}

impl<T, B, I> Intersectable<T> for TypedGroup<T, B, I> 
where T: Float, B: DistanceMeasure<T>, I: Intersectable<T> {
    fn intersect(&self, max_distance: T, ray: &Ray<T>) -> Intersection<T> {
        if self.bound.distance_from_ray(&ray) == Float::infinity() {
            return None
        }
        for item in self.children.iter() {
            let h = match *item {
                Pair::Item(ref v) => v.intersect(max_distance, &ray),
                Pair::Group(ref g) => g.intersect(max_distance, &ray),
            };
            if h.is_some() {
                return h;
            }
        }

        None
    }
}

pub type SphericalGroup<T> = TypedGroup<T, Sphere<T>, Sphere<T>>;


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::primitive::Intersectable;
    use super::super::vec::Vector;
    use super::super::primitive::{Sphere, Ray};
    use std::default::Default;
    use std::num::Float;

    #[test]
    fn intersect() {
        let s1 = Sphere {center: Default::default(),
                         radius: 1.0f32 };
        let mut s2: Sphere<f32> = Default::default();
        s2.center.z = s2.radius * 2.0;

        let mut g: SphericalGroup<f32> = Default::default();
        assert!(g.children.len() == 0);

        g.children.push(Pair::Item(s1));
        g.children.push(Pair::Item(s2));
        g.bound.radius = 3.0; // Bigger than it needs to be !
        let g = g;  // strip mut

        // ray for sphere 1
        let mut r1: Ray<f32> = Default::default();
        r1.pos.x = 2.0;
        r1.dir.x = -1.0;
        let r1 = r1;

        // ray for sphere 2
        let mut r2 = r1;
        r2.pos.z = s2.center.z;
        let r2 = r2;

        // missing ray
        let mut r3 = r1;
        r3.dir.x = -r3.dir.x;
        let r3 = r3;


        // Intersect Rays
        for ray in [&r1, &r2].iter() {
            let ho = g.intersect(Float::infinity(), &ray);
            assert!(!ho.is_none());
            let h = ho.unwrap();
            assert_eq!(h.distance, 1.0f32);
            assert_eq!(h.pos.x, 1.0f32);
            assert_eq!(h.pos.z, 0.0f32);
        }

        assert!(g.intersect(Float::infinity(), &r3).is_none());
    }
    
}
