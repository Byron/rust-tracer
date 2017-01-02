//! Allows to setup a scene with scenes in pyramidal layout, along with traits
//! to help shooting rays to check for intersections

use super::vec::{Vector, RFloat};
use std::default::Default;

use std::f32;

#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct Ray {
    pub pos: Vector,
    pub dir: Vector,
}

#[derive(Clone, Copy)]
pub struct Hit {
    pub distance: RFloat,
    pub pos: Vector,
}

impl Hit {
    pub fn missed() -> Hit {
        Hit {
            distance: f32::INFINITY,
            pos: Default::default(),
        }
    }

    pub fn has_missed(&self) -> bool {
        self.distance == f32::INFINITY
    }

    pub fn set_missed(&mut self) {
        self.distance = f32::INFINITY;
    }
}

#[derive(Clone, Copy)]
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
    #[inline(always)]
    fn distance_from_ray(&self, r: &Ray) -> RFloat {
        let v = self.center - r.pos;
        let b = v.dot(&r.dir);
        let disc = b * b - v.dot(&v) + self.radius * self.radius;

        if disc < 0.0 {
            return f32::INFINITY;
        }

        let d = disc.sqrt();
        let t2 = b + d;
        if t2 < 0.0 {
            return f32::INFINITY;
        }

        let t1 = b - d;
        if t1 > 0.0 { t1 } else { t2 }
    }
}

impl Intersectable for Sphere {
    #[inline(always)]
    fn intersect(&self, hit: &mut Hit, ray: &Ray) {
        let distance = self.distance_from_ray(ray);
        if distance >= hit.distance {
            return;
        }
        hit.distance = distance;
        hit.pos = (ray.pos + (ray.dir.mulfed(distance) - self.center)).normalized();
    }
}

pub trait Intersectable {
    /// Return intersection point of ray with item (relative to the Ray !!)
    fn intersect(&self, &mut Hit, ray: &Ray);
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
        let r1: Ray = Ray {
            pos: Default::default(),
            dir: Default::default(),
        };

        let r2: Ray = Default::default();
        assert_eq!(r1, r2);
    }
}

#[cfg(test)]
mod sphere {
    extern crate test;

    use super::*;
    use std::default::Default;
    use super::super::vec::Vector;

    use std::f32;


    fn setup_scene() -> (Ray, Ray, Sphere) {
        let s = Sphere {
            center: Default::default(),
            radius: 1.0,
        };

        let mut dir: Vector = Default::default();
        dir.x = -1.0;
        let r1 = Ray {
            pos: Vector {
                x: 2.0,
                y: 0.0,
                z: 0.0,
            },
            dir: dir,
        };
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
            assert_eq!(dfr, f32::INFINITY);
        }

        {
            let mut h = Hit {
                distance: 2.0,
                pos: Default::default(),
            };
            s.intersect(&mut h, &r1);
            assert_eq!(h.distance, 1.0);
            assert_eq!(h.pos.x, 1.0);

            h.distance = 0.5;
            s.intersect(&mut h, &r1);
            assert!(h.distance == 0.5, "Max Distance too short");
            h.distance = 10.0;
            s.intersect(&mut h, &r2);
            assert!(h.distance == 10.0, "r2 is shot the wrong way");
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
            for _ in 0..NUM_ITERATIONS {
                test::black_box(s.distance_from_ray(&r1));
                test::black_box(s.distance_from_ray(&r2));
            }
        });
        b.bytes = (NUM_ITERATIONS * 2) as u64;
    }

    #[bench]
    fn bench_intersect(b: &mut test::Bencher) {
        let (r1, r2, s) = setup_scene();
        let mut h = Hit::missed();
        b.iter(|| {
            for _ in 0..NUM_ITERATIONS {
                h.set_missed();
                test::black_box(s.intersect(&mut h, &r1));
                h.set_missed();
                test::black_box(s.intersect(&mut h, &r2));
            }
        });
        b.bytes = (NUM_ITERATIONS * 2) as u64;
    }

}
