/// Implements the actual raytracer which produces the final image
use std::num::{Float, NumCast};
use std::old_io;
use std::old_io::stdio;
use std::default::Default;
use std::fmt::Debug;
use super::vec::Vector;
use super::group::SphericalGroup;
use super::primitive::{Intersectable, Ray};

pub trait PixelWriter<T> {
    /// To be called before writing the first pixel
    /// x and y are the max image resolution
    fn begin(&mut self, x: u16, y: u16);

    /// Write one line after another, when x is width, skip to next line
    /// This must be assured by the caller
    /// Color range is 0.0 to 1.0, everything higher leads to overflows
    fn write_next_pixel(&mut self, color: &Vector<T>);
}


pub struct Renderer<T> {
    pub width: u16,
    pub height: u16,
    pub samples_per_pixel: u16,
}

pub struct Scene<T> {
    pub group: SphericalGroup<T>,
    pub omni_light_pos: Vector<T>,
    pub eye: Vector<T>,
}

impl<T: Float + Default + Debug> Default for Scene<T> {
    fn default() -> Scene<T> {
        let one: T = Float::one();
        let zero: T = Float::zero();
        Scene {
            group: SphericalGroup::<T>::pyramid(8, &Vector { x: one, 
                                                             y: -one, 
                                                             z: zero }, one),
            omni_light_pos: Vector { x: -one, y: -(one+one+one), z: (one+one) },
            eye: Vector { x: zero, y: zero, z: -(one+one+one+one) }
        }
    }
}

impl<T: Float + Default> Renderer<T> {

    fn raytrace(&self, s: &Scene<T>, r: &Ray<T>, c: &mut Vector<T>) {
        let h = s.group.intersect(Float::infinity(), r);
        match h {
            Some(h) => {
                c.x = c.x + Float::one();
                c.y = c.y + Float::one();
                c.z = c.z + Float::one();
            }
            _ => {}
        }
    }

    // Use runtime dispatching for the image writer to remain flexible
    // (And to test this ;))
    pub fn render(&self, scene: &Scene<T>, writer: &mut PixelWriter<T>) {
        writer.begin(self.width, self.height);
        let ssf: T = NumCast::from(self.samples_per_pixel).unwrap();
        let wf: T = NumCast::from(self.width).unwrap();
        let hf: T = NumCast::from(self.height).unwrap();

        let mut ray = Ray { pos: scene.eye, 
                            dir: Default::default() };
        let two = <T as Float>::one() + Float::one();

        for y in range(0, self.height) {
            for x in range(0, self.width) {
                let mut g: Vector<T> = Default::default();

                for ssx in range(0, self.samples_per_pixel) {
                    for ssy in range(0, self.samples_per_pixel) {
                        let xres = <T as NumCast>::from(x).unwrap() + 
                                   <T as NumCast>::from(ssx).unwrap() / ssf;
                        let yres = <T as NumCast>::from(y).unwrap() + 
                                   <T as NumCast>::from(ssy).unwrap() / ssf;
                        ray.dir.x = xres - wf / two;
                        ray.dir.y = yres - hf / two;
                        ray.dir.z = wf;
                        ray.dir.normalize();
                        self.raytrace(scene, &ray, &mut g);
                    }//for each ss y
                }// for each ss x

                g.mulf((ssf * ssf).recip());
                writer.write_next_pixel(&g);
            }// for each x
        }// for each y
    }
}

pub struct PPMStdoutPixelWriter {
    out: old_io::LineBufferedWriter<stdio::StdWriter>,
    rgb: bool,
}

impl PPMStdoutPixelWriter {
    pub fn new(write_RGB: bool) -> PPMStdoutPixelWriter {
        PPMStdoutPixelWriter { out: old_io::stdout(), rgb: write_RGB }
    }
}

impl<T: Float + Debug> PixelWriter<T> for PPMStdoutPixelWriter {
    fn begin(&mut self, x: u16, y: u16) {
        let mut ptype: &str = "P5";
        if self.rgb {
            ptype = "P6"
        }
        self.out.write_line(ptype);
        self.out.write_line(&format!("{} {}", x, y)[]);
        self.out.write_line("255");
    }

    fn write_next_pixel(&mut self, c: &Vector<T>) {
        let one: T = Float::one();
        let two55 = (one+one).powi(8) - one; // 255

        if self.rgb {
            self.out.write_u8(<u8 as NumCast>::from(two55 * c.x).unwrap());
            self.out.write_u8(<u8 as NumCast>::from(two55 * c.y).unwrap());
            self.out.write_u8(<u8 as NumCast>::from(two55 * c.z).unwrap());
        } else {
            let avg = (c.x + c.y + c.z) / (one+one+one);
            self.out.write_u8(<u8 as NumCast>::from(two55 * avg).unwrap());
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use super::super::vec::Vector;
    use super::super::group::SphericalGroup;
    use std::default::Default;

    #[derive(Default)]
    struct DummyWriter {
        begin_called: bool,
        write_count: usize,
    }

    impl<T> PixelWriter<T> for DummyWriter {
        fn begin(&mut self, x: u16, y: u16) {
            self.begin_called = true;
        }
        fn write_next_pixel(&mut self, color: &Vector<T>) {
            self.write_count += 1;
        }
    }

    const W: usize = 32;
    const H: usize = 16;

    #[test]
    fn basic_rendering() {
        let s: Scene<f32> = Default::default();
        let r = Renderer { width: W as u16,
                           height: H as u16,
                           samples_per_pixel: 2 };

        let mut dw: DummyWriter = Default::default();
        r.render(&s, &mut dw);

        assert!(dw.begin_called);
        assert!(dw.write_count == W * H);
    }

    #[bench]
    fn bench_rendering(b: &mut test::Bencher) {
        const SPP: usize = 1;
        let s: Scene<f32> = Default::default();
        let r = Renderer { width: H as u16,
                           height: H as u16,
                           samples_per_pixel: SPP as u16 };

        let mut dw: DummyWriter = Default::default();
        b.iter(|| {
            r.render(&s, &mut dw);
        });
        b.bytes += (H * H * SPP * SPP) as u64;
    }
}