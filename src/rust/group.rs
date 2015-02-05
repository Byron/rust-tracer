/// Implements a group of intersectable items

use std::num::Float;
use super::vec::Vector;
use std::default::Default;
use std::fmt::Debug;
use super::primitive::{DistanceMeasure, Intersectable, Intersection, Hit, Ray, Sphere};

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


// Separating the generator will work, but will also make everything so much more complicated !
// Can be done as an excersise later, once there is an image to render and to review
// fn generate_pyramidal_positions<T, F>(l: u32, p: &Vector<T>, r: T, f: F)
//     where T: Float, F: Fn(u32, T, Vector<T>) {
//     f(l, r, p)
//     if level == 1 {
//         return
//     }
//     let i: 0us;


// }

/// It's interesting that 'type' is indeed a new type, and not a type-def ! At least 
/// when used in this situation !!!
impl<T> SphericalGroup<T>
    where T: Float + Default + Debug {

    // fn pyramid_recursive(level: u32, origin: &Vector<T>, radius: T) -> TypedGroupPair<T, B, I> {
    //     let g: TypedGroup<T, B, I> = Default::default();
    // }

    pub fn pyramid(level: u32, origin: &Vector<T>, radius: T) -> SphericalGroup<T> {
        let g: SphericalGroup<T> = Default::default();
        g
    }
    // pub fn pyramid() {

    // }
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

    #[test]
    fn pyramid() {
        // let g = SphericalGroup::<f32> {bound: Default::default(), children: Default::default()};
        // For some reason, this doesn't resolve the name
        let g = SphericalGroup::<f32>::pyramid(8, &Vector { x: 1.0f32, 
                                                            y: -1.0f32, 
                                                            z: 0.0f32 }, 1.0);
        // let g: SphericalGroup<f32> = TypedGroup::pyramid(8, &Vector { x: 1.0f32, 
        //                                                              y: -1.0f32, 
        //                                                              z: 0.0f32 }, 1.0);
        // This also works, of course, but is less readable !
        // let g = TypedGroup::<f32, Sphere<f32>, Sphere<f32>>::pyramid(8, &Vector { x: 1.0f32, 
        //                                                                          y: -1.0f32, 
        //                                                                          z: 0.0f32 }, 
        //                                                                          1.0);

        assert!(g.children.len() > 0);

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
