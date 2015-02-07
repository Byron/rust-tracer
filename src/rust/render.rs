/// Implements the actual raytracer which produces the final image
use std::num::Float;
use std::old_io;
use std::old_io::stdio;
use std::ops::Drop;
use std::env;
use std::sync::TaskPool;
use std::default::Default;
use std::iter::range_step;
use std::sync::mpsc::sync_channel;
use super::vec::{Vector, RFloat};
use super::group::SphericalGroup;
use super::primitive::{Intersectable, Ray, Hit};


pub trait RGBABufferWriter {
    /// To be called before writing the first pixel
    /// x and y are the total image resolution
    fn begin(&mut self, x: u16, y: u16);

    /// Write the given RGBA buffer - it's image region might be anywhere within 
    /// our confines of total x and y resolution.
    /// This must be assured by the caller
    /// Color range is 0.0 to 1.0, everything higher is truncated
    fn write_rgba_buffer(&mut self, buffer: &RGBABuffer);
}


#[derive(Copy)]
pub struct RenderOptions {
    pub width: u16,
    pub height: u16,
    pub samples_per_pixel: u16,
}

pub struct Renderer;

#[derive(Copy, PartialEq)]
pub struct ImageRegion {
    l: u16,
    t: u16,
    r: u16,
    b: u16,
}

impl ImageRegion {
    pub fn width(&self) -> u16 {
        self.r - self.l
    }

    pub fn height(&self) -> u16 {
        self.t - self.b
    }

    pub fn area(&self) -> usize {
        self.width() as usize * self.height() as usize
    }

    pub fn contains(&self, o: &ImageRegion) -> bool {
        return  o.l >= self.l && 
                o.b >= self.b &&
                o.t <= self.t &&
                o.r <= self.r
    }

    /// x and y absolute to our rectangle ! Returns offset relative to our buffer
    #[inline]
    pub fn buffer_offset(&self, x: u16, y: u16) -> usize {
        (y - self.b) as usize * self.width() as usize + (x - self.l) as usize
    }
}

pub struct RGBABuffer {
    buf: Vec<u8>,
    reg: ImageRegion,
}

impl RGBABuffer {
    fn new(r: &ImageRegion) -> RGBABuffer {
        let mut v = Vec::with_capacity(r.area() * 4);
        let l = v.capacity();
        unsafe {v.set_len(l)};
        RGBABuffer {
            buf: v,
            reg: *r,
        }
    }

    fn components() -> usize {
        4
    }

    /// x and y must be absolute to our recangle !
    fn set_pixel_from_vector(&mut self, x: u16, y: u16, p: &Vector, alpha: RFloat) {
        let ofs = self.reg.buffer_offset(x, y) * RGBABuffer::components();
        let c = &mut self.buf[ofs .. ofs + RGBABuffer::components()];

        let scale = |v| -> u8 {

            let r = 0.5 + 255.0 * v;
            if r > 255.0 {
                return 255;
            }
            r as u8
        };

        c[0] = scale(p.x);
        c[1] = scale(p.y);
        c[2] = scale(p.z);
        c[3] = scale(alpha);
    }

    /// buffer must be contained in our rectangle
    fn set_pixels_from_buffer(&mut self, b: &RGBABuffer) {
        assert!(self.reg.contains(&b.reg));
        let w = b.reg.width() as usize * RGBABuffer::components();

        if self.reg == b.reg {
            self.buf = b.buf.clone();
            return;
        }

        for y in range(b.reg.b, b.reg.t) {
            let bl = self.reg.buffer_offset(b.reg.l, y) * RGBABuffer::components(); // bottom_left
            let their_bl = b.reg.buffer_offset(b.reg.l, y) * RGBABuffer::components();
            self.buf[bl .. bl + w].clone_from_slice(&b.buf[their_bl .. their_bl + w]);
        }
    }

    fn buffer(&self) -> &Vec<u8> {
        &self.buf
    }

    fn region(&self) -> &ImageRegion {
        &self.reg
    }
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

    #[inline]
    fn raytrace(s: &Scene, r: &Ray, c: &mut Vector) -> RFloat {
        let mut h = Hit::missed();
        s.group.intersect(&mut h, r);
        if h.has_missed() {
            return 0.0;
        }
        let g = h.pos.dot(&s.directional_light);
        if g >= 0.0 {
            return 0.0;
        }
        let p = r.pos + 
                    (r.dir.mulfed(h.distance)) + 
                    *h.pos.mulf(h.distance * <RFloat as Float>::epsilon().sqrt());

        // if there is something between us and the light, we are in shadow
        h.set_missed();
        s.group.intersect(&mut h,
                          &Ray { pos: p,
                                  dir: s.directional_light.mulfed(-1.0) });
        if h.has_missed() {
            c.x = c.x - g;
            c.y = c.y - g;
            c.z = c.z - g;
            return 1.0;
        }

        0.0
    }

    // Render region is inherently single-threaded
    pub fn render_region(samples_per_pixel: u16, width: RFloat, height: RFloat, 
                         scene: &Scene, buf: &mut RGBABuffer) {
        let ssf = samples_per_pixel as RFloat;
        let total_samples_per_pixel_recip = (ssf * ssf).recip();
        let region = *buf.region();

        let mut ray = Ray { pos: scene.eye, 
                            dir: Default::default() };

        for y in range(region.b, region.t) {
            for x in range(region.l, region.r) {
                let mut g: Vector = Default::default();
                let mut alpha: RFloat = 0.0;

                for ssx in range(0, samples_per_pixel) {
                    for ssy in range(0, samples_per_pixel) {
                        let xres = x as RFloat + ssx as RFloat / ssf;
                        let yres = y as RFloat + ssy as RFloat / ssf;
                        ray.dir.x = xres - width / 2.0;
                        ray.dir.y = yres - height / 2.0;
                        ray.dir.z = width;
                        ray.dir.normalize();
                        alpha += Renderer::raytrace(scene, &ray, &mut g);

                    }//for each ss y
                }// for each ss x

                g.mulf(total_samples_per_pixel_recip);
                alpha *= total_samples_per_pixel_recip;

                buf.set_pixel_from_vector(x, region.t - (y + 1), &g, alpha);
            }// for each x
        }// for each y
    }

    // Use runtime dispatching for the image writer to remain flexible
    // (And to test this ;))
    // sets up multi-threading accordingly
    pub fn render(o: &RenderOptions, scene: &Scene, writer: &mut RGBABufferWriter, 
                  pool: &TaskPool) {
        const CHUNK_SIZE: u16 = 64;
        assert!(o.width % CHUNK_SIZE == 0, "TODO: handle chunk sizes");
        assert!(o.height % CHUNK_SIZE == 0, "TODO: handle chunk sizes");

        writer.begin(o.width, o.height);

        // Push all tasks
        let (tx, rx) = sync_channel::<RGBABuffer>(4);
        let mut count = 0us;
        for y in range_step(0u16, o.height, CHUNK_SIZE)  {
            for x in range_step(0u16, o.width, CHUNK_SIZE) {

                let tx = tx.clone();
                let ss = o.samples_per_pixel;
                let w = o.width as RFloat;
                let h = o.height as RFloat;
                count += 1;
                pool.execute(move|| {
                    let b = RGBABuffer::new(&ImageRegion { l: x, r: x + CHUNK_SIZE, 
                                                           b: y, t: y + CHUNK_SIZE });
                    // If this is commented in, we get lifetime errors, probably due to 
                    // ... scene ?
                    Renderer::render_region(ss, w, h, scene, &mut b);
                    tx.send(b).ok();
                });
            }
        }

        // Read the results and pass them to the writer
        for b in rx.iter() {
            writer.write_rgba_buffer(&b);
            count -= 1;
            if count == 0 {
                break;
            }
        }
    }
}

pub struct PPMStdoutRGBABufferWriter {
    out: old_io::LineBufferedWriter<stdio::StdWriter>,
    image: Option<RGBABuffer>,
    rgb: bool,
}

impl Drop for PPMStdoutRGBABufferWriter {
    fn drop(&mut self) {
        // We always write our entire buffer - it will just be zero initially
        // That way, we can see the buckets being written in preview :) ... .
        // Can't write entire buffer :( thanks to alpha channel
        let buf = self.image.as_ref().unwrap().buffer();
        for po in range_step(0, buf.len(), RGBABuffer::components()) {
            let b = &buf[po .. po + 3];

            if self.rgb {
                self.out.write(b).unwrap();
            } else {
                let avg = ((b[0] as f32 + b[1] as f32 + b[2] as f32) / 3.0f32) as u8;
                self.out.write_u8(avg).unwrap();
            }
        }
    }
}

impl PPMStdoutRGBABufferWriter {
    pub fn new(write_rgb: bool) -> PPMStdoutRGBABufferWriter {
        PPMStdoutRGBABufferWriter { out: old_io::stdout(),
                               image: None,
                               rgb: write_rgb }
    }
}

impl RGBABufferWriter for PPMStdoutRGBABufferWriter {
    fn begin(&mut self, x: u16, y: u16) {
        let mut ptype: &str = "P5";
        if self.rgb {
            ptype = "P6"
        }
        self.out.write_line(ptype).unwrap();
        self.out.write_line(&format!("{} {}", x, y)[]).unwrap();
        self.out.write_line("255").unwrap();

        self.image = Some(RGBABuffer::new(&ImageRegion { l: 0, r: x, b: 0, t: y }));
    }

    fn write_rgba_buffer(&mut self, buffer: &RGBABuffer) {
        self.image.as_mut().unwrap().set_pixels_from_buffer(buffer);
    }
}


#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use super::super::vec::Vector;
    use std::sync::TaskPool;
    use std::default::Default;

    #[derive(Default)]
    struct DummyWriter {
        begin_called: bool,
        write_count: usize,
    }

    impl RGBABufferWriter for DummyWriter {
        fn begin(&mut self, _: u16, _: u16) {
            self.begin_called = true;
        }
        fn write_rgba_buffer(&mut self, _: &RGBABuffer) {
            self.write_count += 1;
        }
    }

    const W: usize = 64;
    const H: usize = 128;

    #[test]
    fn basic_rendering() {
        let s: Scene = Default::default();
        let pool = TaskPool::new(1);
        let options = RenderOptions { width: W as u16, height: H as u16, samples_per_pixel: 2 };

        let mut dw: DummyWriter = Default::default();
        Renderer::render(&options, &s, &mut dw, &pool);

        assert!(dw.begin_called);
        assert!(dw.write_count == 1);
    }

    #[test]
    fn image_region() {
        let r = ImageRegion { l: 2, t: 18, r: 34, b: 2 };
        assert_eq!(r.width(), 32);
        assert_eq!(r.height(), 16);
        assert_eq!(r.area(), 16*32);
        assert!(r.contains(&r));
        let mut l = r;
        l.l = 1;
        assert!(l.contains(&r));
        assert!(!r.contains(&l));
    }

    #[bench]
    fn bench_rendering(b: &mut test::Bencher) {
        const SPP: usize = 1;
        let pool = TaskPool::new(4);
        let s: Scene = Default::default();
        let options = RenderOptions { width: H as u16, height: H as u16, samples_per_pixel: SPP as u16 };

        let mut dw: DummyWriter = Default::default();
        b.iter(|| {
            Renderer::render(&options, &s, &mut dw, &pool);
        });
        b.bytes += (H * H * SPP * SPP) as u64;
    }
}