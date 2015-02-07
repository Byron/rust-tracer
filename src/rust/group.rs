/// Implements a group of intersectable items

use std::num::{Float, NumCast};
use super::vec::Vector;
use std::default::Default;
use std::fmt::Debug;
use std::iter::range_step_inclusive;
use super::primitive::{DistanceMeasure, Intersectable, Intersection, Ray, Sphere, Hit};

#[derive(Debug)]
pub enum Pair<I, G> {
    Item(I),
    Group(G),
}

type TypedGroupPair<T, B, I> = Pair<I, TypedGroup<T, B, I>>;

/// A group with static dispatch on intersect calls, but dynamically allocated 
/// array of items.
#[derive(Debug, Default)]
pub struct TypedGroup<T, B, I> {
    pub bound: B,
    pub children: Vec<TypedGroupPair<T, B, I>>,
}

// }

/// It's interesting that 'type' is indeed a new type, and not a type-def ! At least 
/// when used in this situation !!!
impl<T> SphericalGroup<T>
    where T: Float + Default + Debug + NumCast {

    fn pyramid_recursive(level: u32, p: &Vector<T>, r: f64)
        -> TypedGroupPair<T, Sphere<T>, Sphere<T>> {
        let s = Sphere {
            center: *p,
            radius: <T as NumCast>::from(r).unwrap(),
        };
        if level == 1 {
            return Pair::Item(s);
        }

        let mut g: SphericalGroup<T> = Default::default();
        g.children.reserve(5);
        g.children.push(Pair::Item(s));
        g.bound.center = *p;
        g.bound.radius = <T as NumCast>::from(3.0 * r).unwrap();

        let rn = 3.0 * r / 12.0.sqrt();
        for dz in range_step_inclusive(-1i32, 1, 2) {
            for dx in range_step_inclusive(-1i32, 1, 2) {
                let rnt = <T as NumCast>::from(rn).unwrap();
                let np = *p + Vector { x: <T as NumCast>::from(dx).unwrap() * rnt,
                                       y: rnt, 
                                       z: <T as NumCast>::from(dz).unwrap() * rnt };
                g.children.push(SphericalGroup::pyramid_recursive(level-1, &np, r * 0.5));
            }
        }
        Pair::Group(g)
    }

    pub fn pyramid(level: u32, origin: &Vector<T>, radius: T) -> SphericalGroup<T> {
        assert!(level > 1, "Levels equal or smaller than one cause empty groups");
        let radius_f64 = <f64 as NumCast>::from(radius).unwrap();
        match SphericalGroup::pyramid_recursive(level, origin, radius_f64) {
            Pair::Group(g) => g,
            _ => unreachable!(),
        }
    }
}

impl<T, B, I> Intersectable<T> for TypedGroup<T, B, I> 
where T: Float + Default, B: DistanceMeasure<T>, I: Intersectable<T> {
    fn intersect(&self, max_distance: T, ray: &Ray<T>) -> Intersection<T> {
        if self.bound.distance_from_ray(&ray) >= max_distance {
            return None
        }

        let mut closest_hit: Hit<T> = Hit { distance: max_distance, 
                                            pos: Default::default() };
        for item in self.children.iter() {
            let ho = match *item {
                Pair::Item(ref v) => v.intersect(closest_hit.distance, &ray),
                Pair::Group(ref g) => g.intersect(closest_hit.distance, &ray),
            };

            // TODO: Can this be done more ideomatically, using option helpers ?
            if let Some(ref hit) = ho {
                if hit.distance < closest_hit.distance {
                    closest_hit = *hit;
                }
            }
        }
        if closest_hit.distance < max_distance { Some(closest_hit) } else { None }
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

    #[test]
    fn pyramid() {
        let g = SphericalGroup::<f32>::pyramid(8, &Vector { x: 1.0f32, 
                                                            y: -1.0f32, 
                                                            z: 0.0f32 }, 1.0);

        assert_eq!(g.children.len(), 5);
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
