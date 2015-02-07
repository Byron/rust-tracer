//! Allows to setup a scene with scenes in pyramidal layout, along with traits 
//! to help shooting rays to check for intersections

use super::vec::Vector;
use std::num::Float;
use std::default::Default;


#[derive(Debug, Default, PartialEq, Copy)]
pub struct Ray<T: Float> {
    pub pos: Vector<T>,
    pub dir: Vector<T>
}

#[derive(Debug, Copy)]
pub struct Hit<T: Float> {
    pub distance: T,
    pub pos: Vector<T>,
}

#[derive(Debug, Copy)]
pub struct Sphere<T: Float> {
    pub center: Vector<T>,
    pub radius: T,
}

impl<T: Float + Default> Default for Sphere<T> {
    fn default() -> Sphere<T> {
        Sphere {
            center: Default::default(),
            radius: Float::one(),
        }
    }
}

impl<T: Float> DistanceMeasure<T> for Sphere<T> {
    fn distance_from_ray(&self, r: &Ray<T>) -> T {
        let v = self.center - r.pos;
        let b = v.dot(&r.dir);
        let disc = b * b - v.dot(&v) + self.radius * self.radius;

        if disc < Float::zero() {
            return Float::infinity();
        }

        let d = disc.sqrt();
        let t2 = b + d;
        if t2 < Float::zero() {
            return Float::infinity();
        }

        let t1 = b - d;
        if t1 > Float::zero() { t1 } else { t2 }
    }
}

impl<T: Float> Intersectable<T> for Sphere<T> {
    fn intersect(&self, max_distance: T, ray: &Ray<T>) -> Intersection<T> {
        let distance = self.distance_from_ray(ray);
        if distance >= max_distance {
            return None;
        }
        Some(Hit { distance: distance, 
                   pos: (ray.pos + (ray.dir.mulfed(distance) - self.center)).normalized() })
    }
}

pub type Intersection<T> = Option<Hit<T>>;

pub trait Intersectable<T> {
    /// Return intersection point of ray with item (relative to the Ray !!)
    fn intersect(&self, max_distance: T, ray: &Ray<T>) -> Intersection<T>;
}

pub trait DistanceMeasure<T> {
    fn distance_from_ray(&self, r: &Ray<T>) -> T;
}


#[cfg(test)]
mod primitive_tests {
    use super::Ray;
    use std::default::Default;

    #[test]
    fn ray_defaults() {
        let r1: Ray<f32> = Ray {pos: Default::default(),
                                dir: Default::default() };

        let r2: Ray<f32> = Default::default();
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
    

    fn setup_scene<T: Float + Default>() -> (Ray<T>, Ray<T>, Sphere<T>) {
        let s: Sphere<T> = Sphere { center: Default::default(),
                                    radius: Float::one() };

        let mut dir: Vector<T> = Default::default();
        // dir.x = -Float::one();  // Inference doesn't really work it appears
        dir.x = -<T as Float>::one();
        let r1: Ray<T> = Ray { pos: Vector { x: <T as Float>::one() + <T as Float>::one(), 
                                             y: <T as Float>::zero(), 
                                             z: <T as Float>::zero() },
                               dir: dir};
        let mut r2 = r1;
        r2.dir.x = -r2.dir.x;   // invert direction
        (r1, r2, s)
    }

    #[test]
    fn intersect() {
        let (r1, r2, s) = setup_scene::<f32>();

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
        let s: Sphere<f32> = Default::default();
        assert!(s.radius != 0.0);
    }

    const NUM_ITERATIONS: usize = 10000;

    #[bench]
    fn bench_ray_sphere(b: &mut test::Bencher) {
        let (r1, r2, s) = setup_scene::<f32>();
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
        let (r1, r2, s) = setup_scene::<f32>();
        b.iter(|| {
            for _ in range(0, NUM_ITERATIONS) {
                test::black_box(s.intersect(Float::infinity(), &r1));
                test::black_box(s.intersect(Float::infinity(), &r1));
            }
        });
        b.bytes += (NUM_ITERATIONS * 2) as u64;
    }

}