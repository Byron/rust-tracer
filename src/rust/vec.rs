//! A module implementing a Vector type which can be parametized to support different
//! floating point precision.

extern crate test;

use std::num::Float;
use std::ops::{Add, Sub, Mul};

#[derive(Debug, PartialEq, Eq, Copy)]
pub struct Vector<T: Float> {
    x: T,
    y: T,
    z: T,
}

impl<T: Float> Add for Vector<T> {
    type Output = Vector<T>;

    // Probably it will be optimized to not actually copy self and rhs for each call !
    #[inline(always)]
    fn add(self, rhs: Vector<T>) -> Vector<T> {
      Vector {  x: self.x + rhs.x,
                y: self.y + rhs.y, 
                z: self.z + rhs.z }
    }
}

impl<T: Float> Sub for Vector<T> {
    type Output = Vector<T>;

    #[inline(always)]
    fn sub(self, rhs: Vector<T>) -> Vector<T> {
      Vector {  x: self.x - rhs.x,
                y: self.y - rhs.y, 
                z: self.z - rhs.z }
    }   
}

impl<T: Float> Mul for Vector<T> {
    type Output = Vector<T>;

    #[inline(always)]
    fn mul(self, rhs: Vector<T>) -> Vector<T> {
      Vector {  x: self.x * rhs.x,
                y: self.y * rhs.y, 
                z: self.z * rhs.z }
    }   
}

impl<'a, T: Float> Vector<T> {
    #[inline(always)]
    fn mulfed(&self, m: T) -> Vector<T> {
        Vector {
            x: self.x * m,
            y: self.y * m,
            z: self.z * m,
        }
    }

    // in ruby, you can use ! to signal it's in-place - here we have to find another way
    #[inline(always)]
    fn mulf(&'a mut self, m: T) -> &'a mut Vector<T> {
        self.x = self.x * m;
        self.y = self.y * m;
        self.z = self.z * m;
        self
    }

    // The dot product - should we keep going and use  &ref type as self ?
    // Or just keep copying self around as in sub, add, mul ?
    #[inline(always)]
    fn dot(&self, r: &Vector<T>) -> T {
        self.x * r.x + self.y * r.y + self.z * r.z
    }

    // #[inline(always)]
    // fn normalize(&'a mut self) -> &'a mut Vector<T> {
    //     self.mulf(Float::one() / self.dot(self).sqrt())
    // }
    // fn normalized(&self) -> Vector<T> {
    //     self.mulfed(Float::one() as T / self.dot(self).sqrt())
    // }
}


#[cfg(test)]
#[test]
fn basics() {
    let v32 = Vector { x: 5.0f32, y: 4.0f32, z: 0.0f32 };
    assert_eq!(v32.x, 5.0f32);
    assert_eq!(v32, v32);
    assert!(!(v32 != v32));
    {
        // This is a copyable type - if not, this assignment would fail
        let mut copy = v32;
        copy.x = 10.0f32;
    }

    let v64 = Vector { x: 1.0f64, y: 2.0f64, z: 3.0f64, };
    assert_eq!(v64.x, 1.0f64);

    println!("{:?}", v32);
    println!("{:?}", v64);

    // Addition
    let v = v32 + v32;
    assert_eq!(v.x, v32.x + v32.x);
    assert_eq!(v.y, v32.y + v32.y);
    assert_eq!(v.z, v32.z + v32.z);

    let v = v32 - v32;
    assert_eq!(v.x, 0.0f32);

    let v = v32 * v32;
    assert_eq!(v.x , v32.x * v32.x);

    let mut v = v32.mulfed(3.0);
    assert_eq!(v.x, v32.x * 3.0);

    v.mulf(2.0);
    assert_eq!(v.x, v32.x * 3.0 * 2.0);
}

// #[cfg(test)]
// #[bench]
// fn perf(b: &mut test::Bencher) {
//     let v = Vector { x: 5.0f32, y: 4.0f32, z: 0.0f32 };
//     let mut x = v;
//     b.iter(|| {
//         for _ in range(1, 1000000) {
//             x = x + v;
//         }
//     })
// }