/// Implements a group of intersectable items

use super::vec::{Vector, RFloat};
use std::default::Default;
use std::iter::range_step_inclusive;
use super::primitive::{DistanceMeasure, Intersectable, Ray, Sphere, Hit};

pub enum Pair<I, G> {
    Item(I),
    Group(G),
}

pub type TypedGroupPair<B, I> = Pair<I, TypedGroup<B, I>>;

/// A group with static dispatch on intersect calls, but dynamically allocated 
/// array of items.
#[derive(Default)]
pub struct TypedGroup<B, I> {
    pub bound: B,
    pub children: Vec<TypedGroupPair<B, I>>,
}

// }

/// It's interesting that 'type' is indeed a new type, and not a type-def ! At least 
/// when used in this situation !!!
/// We actually have our own type methods, but would share instance methods
impl SphericalGroup {

    fn pyramid_recursive(level: u32, p: &Vector, r: RFloat) -> TypedGroupPair<Sphere, Sphere> {
        let s = Sphere {
            center: *p,
            radius: r,
        };
        if level == 1 {
            return Pair::Item(s);
        }

        let mut g: SphericalGroup = Default::default();
        g.children.reserve(5);
        g.children.push(Pair::Item(s));
        g.bound.center = *p;
        g.bound.radius = 3.0 * r;

        let rn: RFloat = 3.0 * r / 12.0f32.sqrt();
        for dz in range_step_inclusive(-1i32, 1, 2) {
            for dx in range_step_inclusive(-1i32, 1, 2) {
                let np = *p + Vector { x: dx as RFloat * rn,
                                       y: rn, 
                                       z: dz as RFloat * rn };
                g.children.push(SphericalGroup::pyramid_recursive(level-1, &np, r * 0.5));
            }
        }
        Pair::Group(g)
    }

    pub fn pyramid(level: u32, origin: &Vector, radius: RFloat) -> SphericalGroup {
        assert!(level > 1, "Levels equal or smaller than one cause empty groups");
        match SphericalGroup::pyramid_recursive(level, origin, radius) {
            Pair::Group(g) => g,
            _ => unreachable!(),
        }
    }
}

impl<B, I> Intersectable for TypedGroup<B, I> where B: DistanceMeasure, I: Intersectable {
    fn intersect(&self, hit: &mut Hit, ray: &Ray){
        if self.bound.distance_from_ray(&ray) >= hit.distance {
            return
        }

        for item in self.children.iter() {
            match *item {
                Pair::Item(ref v) => v.intersect(hit, &ray),
                Pair::Group(ref g) => g.intersect(hit, &ray),
            };
        }
    }
}

pub type SphericalGroup = TypedGroup<Sphere, Sphere>;


#[cfg(test)]
mod tests {
    extern crate test;

    impl <B, I> TypedGroup<B, I> {

        /// Returns (num_groups, num_items), where the items are our actual payload
        fn count(&self) -> (usize, usize) {
            let mut ng = 1usize;
            let mut ni = 0usize;
            for item in self.children.iter() {
                match *item {
                    Pair::Item(_) => ni += 1,
                    Pair::Group(ref g) => {
                        let (gng, gni) = g.count();
                        ng += gng;
                        ni += gni;
                    }
                }
            }
            (ng, ni)
        }    
    }

    use super::*;
    use super::super::primitive::Intersectable;
    use super::super::vec::Vector;
    use super::super::primitive::{Sphere, Ray, Hit};
    use std::default::Default;

    fn setup_group() -> (Ray, Ray, Ray, SphericalGroup) {
        let s1 = Sphere {center: Default::default(),
                         radius: 1.0 };
        let mut s2: Sphere = Default::default();
        s2.center.z = s2.radius * 2.0;

        let mut g: SphericalGroup = Default::default();
        assert!(g.children.len() == 0);

        g.children.push(Pair::Item(s1));
        g.children.push(Pair::Item(s2));
        g.bound.radius = 3.0; // Bigger than it needs to be !
        let g = g;  // strip mut

        // ray for sphere 1
        let mut r1: Ray = Default::default();
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

        (r1, r2, r3, g)
    }

    #[test]
    fn intersect() {
        let (r1, r2, r3, g) = setup_group();

        // Intersect Rays
        for ray in [&r1, &r2].iter() {
            let mut h = Hit::missed();
            g.intersect(&mut h, &ray);
            assert!(!h.has_missed());
            assert_eq!(h.distance, 1.0);
            assert_eq!(h.pos.x, 1.0);
            assert_eq!(h.pos.z, 0.0);
        }

        let mut h = Hit::missed();
        g.intersect(&mut h, &r3);
        assert!(h.has_missed());
    }

    #[test]
    fn pyramid() {
        let g = SphericalGroup::pyramid(8, &Vector { x: 1.0, 
                                                     y: -1.0, 
                                                     z: 0.0 }, 1.0);

        assert_eq!(g.children.len(), 5);
        assert_eq!(g.count(), (5461, 21845));
    }

    const ITERATIONS: usize = 10000;

    #[bench]
    fn bench_intersect(b: &mut test::Bencher) {
        let (r1, r2, r3, g) = setup_group();
        let mut h = Hit::missed();
        b.iter(|| {
            for _ in 0 .. ITERATIONS {
                test::black_box(g.intersect(&mut h, &r1));
                h.set_missed();
                test::black_box(g.intersect(&mut h, &r2));
                h.set_missed();
                test::black_box(g.intersect(&mut h, &r3));
                h.set_missed();
            }
        });
        b.bytes = (ITERATIONS * 3usize) as u64;
    }
}
