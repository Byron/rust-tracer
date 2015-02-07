//! Allows to setup a scene with scenes in pyramidal layout, along with traits 
//! to help shooting rays to check for intersections

use super::vec::{Vector, RFloat};
use std::num::Float;
use std::default::Default;


#[derive(Default, PartialEq, Copy, Debug)]
pub struct Ray {
    pub pos: Vector,
    pub dir: Vector
}

#[derive(Copy)]
pub struct Hit {
    pub distance: RFloat,
    pub pos: Vector,
}

#[derive(Copy)]
pub struct Sphere {
    pub center: Vector,
    pub radius: RFloat,
}

impl Default for Sphere {
    fn default() -> Sphere {
        Sphere {
            center: Default::default(),
            radius: 1.0,
        }
    }
}

impl DistanceMeasure for Sphere {
    fn distance_from_ray(&self, r: &Ray) -> RFloat {
        let v = self.center - r.pos;
        let b = v.dot(&r.dir);
        let disc = b * b - v.dot(&v) + self.radius * self.radius;

        if disc < 0.0 {
            return Float::infinity();
        }

        let d = disc.sqrt();
        let t2 = b + d;
        if t2 < 0.0 {
            return Float::infinity();
        }

        let t1 = b - d;
        if t1 > 0.0 { t1 } else { t2 }
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, max_distance: RFloat, ray: &Ray) -> Intersection {
        let distance = self.distance_from_ray(ray);
        if distance >= max_distance {
            return None;
        }
        Some(Hit { distance: distance, 
                   pos: (ray.pos + (ray.dir.mulfed(distance) - self.center)).normalized() })
    }
}

pub type Intersection = Option<Hit>;

pub trait Intersectable {
    /// Return intersection point of ray with item (relative to the Ray !!)
    fn intersect(&self, max_distance: RFloat, ray: &Ray) -> Intersection;
}

pub trait DistanceMeasure {
    fn distance_from_ray(&self, r: &Ray) -> RFloat;
}


#[cfg(test)]
mod primitive_tests {
    use super::Ray;
    use std::default::Default;

    #[test]
    fn ray_defaults() {
        let r1: Ray = Ray {pos: Default::default(),
                                dir: Default::default() };

        let r2: Ray = Default::default();
        assert_eq!(r1, r2);
    }
}

#[cfg(test)]
mod sphere {
    extern crate test;

    use super::*;
    use std::num::Float;
    use std::default::Default;
    use super::super::vec::Vector;
    

    fn setup_scene() -> (Ray, Ray, Sphere) {
        let s = Sphere { center: Default::default(),
                                 radius: 1.0 };

        let mut dir: Vector = Default::default();
        dir.x = -1.0;
        let r1 = Ray { pos: Vector { x: 2.0, 
                                     y: 0.0, 
                                     z: 0.0 },
                               dir: dir};
        let mut r2 = r1;
        r2.dir.x = -r2.dir.x;   // invert direction
        (r1, r2, s)
    }

    #[test]
    fn intersect() {
        let (r1, r2, s) = setup_scene();

        {
            let dfr = s.distance_from_ray(&r1);
            assert_eq!(dfr, 1.0);
            let dfr = s.distance_from_ray(&r2);
            assert_eq!(dfr, Float::infinity());
        }

        { 
            match s.intersect(2.0, &r1) {
                Some(Hit { distance: d, pos: p }) => {
                    assert_eq!(d, 1.0);
                    assert_eq!(p.x, 1.0);
                }
                None => unreachable!(),
            }

            assert!(s.intersect(0.5, &r1).is_none(), "Max Distance too short");
            assert!(s.intersect(10.0, &r2).is_none(), "r2 is shot the wrong way");
        }
    }

    #[test]
    fn defaultdefault() {
        let s: Sphere = Default::default();
        assert!(s.radius != 0.0);
    }

    const NUM_ITERATIONS: usize = 10000;

    #[bench]
    fn bench_ray_sphere(b: &mut test::Bencher) {
        let (r1, r2, s) = setup_scene();
        b.iter(|| {
            for _ in range(0, NUM_ITERATIONS) {
                test::black_box(s.distance_from_ray(&r1));
                test::black_box(s.distance_from_ray(&r2));
            }
        });
        b.bytes += (NUM_ITERATIONS * 2) as u64;
    }

    #[bench]
    fn bench_intersect(b: &mut test::Bencher) {
        let (r1, _, s) = setup_scene();
        b.iter(|| {
            for _ in range(0, NUM_ITERATIONS) {
                test::black_box(s.intersect(Float::infinity(), &r1));
                test::black_box(s.intersect(Float::infinity(), &r1));
            }
        });
        b.bytes += (NUM_ITERATIONS * 2) as u64;
    }

}