/// Implements the actual raytracer which produces the final image
use std::num::Float;
use std::old_io;
use std::old_io::stdio;
use std::default::Default;
use super::vec::{Vector, RFloat};
use super::group::SphericalGroup;
use super::primitive::{Intersectable, Ray};

pub trait PixelWriter {
    /// To be called before writing the first pixel
    /// x and y are the max image resolution
    fn begin(&mut self, x: u16, y: u16);

    /// Write one line after another, when x is width, skip to next line
    /// This must be assured by the caller
    /// Color range is 0.0 to 1.0, everything higher leads to overflows
    fn write_next_pixel(&mut self, color: &Vector);
}


#[derive(Copy)]
pub struct Renderer {
    pub width: u16,
    pub height: u16,
    pub samples_per_pixel: u16,
}

pub struct Scene {
    pub group: SphericalGroup,
    pub directional_light: Vector,
    pub eye: Vector,
}

impl Default for Scene {
    fn default() -> Scene {
        Scene {
            group: SphericalGroup::pyramid(8, &Vector { x: 0.0, 
                                                        y: -1.0, 
                                                        z: 0.0 }, 1.0),
            directional_light: Vector { x: -1.0, y: -3.0, z: 2.0 }.normalized(),
            eye: Vector { x: 0.0, y: 0.0, z: -4.0 }
        }
    }
}

impl Renderer {

    fn raytrace(&self, s: &Scene, r: &Ray, c: &mut Vector) {
        if let Some(ref mut h) = s.group.intersect(Float::infinity(), r) {
            let g = h.pos.dot(&s.directional_light);
            if g >= 0.0 {
                return
            }
            let p = r.pos + 
                        (r.dir.mulfed(h.distance)) + 
                        *h.pos.mulf(h.distance * <RFloat as Float>::epsilon().sqrt());

            // if there is something between us and the light, we are in shadow
            if let None = s.group.intersect(
                                Float::infinity(),
                                &Ray { pos: p,
                                       dir: s.directional_light.mulfed(-1.0) }) {
                c.x = c.x - g;
                c.y = c.y - g;
                c.z = c.z - g;
            }
        }
    }

    // Use runtime dispatching for the image writer to remain flexible
    // (And to test this ;))
    pub fn render(&self, scene: &Scene, writer: &mut PixelWriter) {
        writer.begin(self.width, self.height);
        let ssf = self.samples_per_pixel as RFloat;
        let wf = self.width as RFloat;
        let hf = self.height as RFloat;

        let mut ray = Ray { pos: scene.eye, 
                            dir: Default::default() };

        for y in range(0, self.height) {
            for x in range(0, self.width) {
                let mut g: Vector = Default::default();

                for ssx in range(0, self.samples_per_pixel) {
                    for ssy in range(0, self.samples_per_pixel) {
                        let xres = x as RFloat + ssx as RFloat / ssf;
                        let yres = y as RFloat + ssy as RFloat / ssf;
                        ray.dir.x = xres - wf / 2.0;
                        ray.dir.y = yres - hf / 2.0;
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
    pub fn new(write_rgb: bool) -> PPMStdoutPixelWriter {
        PPMStdoutPixelWriter { out: old_io::stdout(), rgb: write_rgb }
    }
}

impl PixelWriter for PPMStdoutPixelWriter {
    fn begin(&mut self, x: u16, y: u16) {
        let mut ptype: &str = "P5";
        if self.rgb {
            ptype = "P6"
        }
        self.out.write_line(ptype).unwrap();
        self.out.write_line(&format!("{} {}", x, y)[]).unwrap();
        self.out.write_line("255").unwrap();
    }

    fn write_next_pixel(&mut self, c: &Vector) {
        let scale = |v| -> u8 {

            let r = 0.5 + 255.0 * v;
            if r > 255.0 {
                return 255;
            }
            r as u8
        };

        if self.rgb {
            self.out.write_u8(scale(c.x)).unwrap();
            self.out.write_u8(scale(c.y)).unwrap();
            self.out.write_u8(scale(c.z)).unwrap();
        } else {
            let avg = (c.x + c.y + c.z) / 3.0;
            self.out.write_u8(scale(avg)).unwrap();
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use super::super::vec::Vector;
    use std::default::Default;

    #[derive(Default)]
    struct DummyWriter {
        begin_called: bool,
        write_count: usize,
    }

    impl PixelWriter for DummyWriter {
        fn begin(&mut self, _: u16, _: u16) {
            self.begin_called = true;
        }
        fn write_next_pixel(&mut self, _: &Vector) {
            self.write_count += 1;
        }
    }

    const W: usize = 32;
    const H: usize = 16;

    #[test]
    fn basic_rendering() {
        let s: Scene = Default::default();
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
        let s: Scene = Default::default();
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