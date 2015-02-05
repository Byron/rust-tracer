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
    extern crate test;

    use super::*;
    use super::super::primitive::Intersectable;
    use super::super::vec::Vector;
    use super::super::primitive::{Sphere, Ray};
    use std::default::Default;
    use std::num::Float;

    fn setup_group<T: Float + Default>() -> (Ray<T>, Ray<T>, Ray<T>, SphericalGroup<T>) {
        let s1 = Sphere {center: Default::default(),
                         radius: Float::one() };
        let mut s2: Sphere<T> = Default::default();
        let one = <T as Float>::one();
        s2.center.z = s2.radius * (one + one);

        let mut g: SphericalGroup<T> = Default::default();
        assert!(g.children.len() == 0);

        g.children.push(Pair::Item(s1));
        g.children.push(Pair::Item(s2));
        g.bound.radius = one + one + one; // Bigger than it needs to be !
        let g = g;  // strip mut

        // ray for sphere 1
        let mut r1: Ray<T> = Default::default();
        r1.pos.x = one + one;
        r1.dir.x = -one;
        let r1 = r1;

        // ray for sphere 2
        let mut r2 = r1;
        r2.pos.z = s2.center.z;
        let r2 = r2;

        // missing ray
        let mut r3 = r1;
        r3.dir.x = -r3.dir.x;
        let r3 = r3;

        (r1, r2, r3, g)
    }

    #[test]
    fn intersect() {
        let (r1, r2, r3, g) = setup_group::<f32>();

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

    const ITERATIONS: usize = 10000;

    #[bench]
    fn bench_intersect(b: &mut test::Bencher) {
        let (r1, r2, r3, g) = setup_group::<f32>();
        b.iter(|| {
            for _ in range(0, ITERATIONS) {
                test::black_box(g.intersect(Float::infinity(), &r1));
                test::black_box(g.intersect(Float::infinity(), &r2));
                test::black_box(g.intersect(Float::infinity(), &r3));
            }
        });
        b.bytes += (ITERATIONS * 3us) as u64;
    }

}
