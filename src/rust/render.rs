/// Implements the actual raytracer which produces the final image
use std::num::Float;
use std::old_io;
use std::old_io::stdio;
use std::ops::{Drop, Deref};
use std::sync::{TaskPool, Arc};
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
        let mut v = Vec::with_capacity(r.area() * RGBABuffer::components());
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
    pub fn render_region(o: &RenderOptions, scene: &Scene, buf: &mut RGBABuffer) {
        let ssf = o.samples_per_pixel as RFloat;
        let total_samples_per_pixel_recip = (ssf * ssf).recip();
        let region = *buf.region();

        let width = o.width as RFloat;
        let height = o.height as RFloat;

        let mut ray = Ray { pos: scene.eye, 
                            dir: Default::default() };

        for y in range(region.b, region.t) {
            for x in range(region.l, region.r) {
                let mut g: Vector = Default::default();
                let mut alpha: RFloat = 0.0;

                for ssx in range(0, o.samples_per_pixel) {
                    for ssy in range(0, o.samples_per_pixel) {
                        let xres = x as RFloat + ssx as RFloat / ssf;
                        let yres = y as RFloat + ssy as RFloat / ssf;
                        ray.dir.x = xres - width / 2.0;
                        ray.dir.y = (height - yres) - height / 2.0;
                        ray.dir.z = width;
                        ray.dir.normalize();
                        alpha += Renderer::raytrace(scene, &ray, &mut g);

                    }//for each ss y
                }// for each ss x

                g.mulf(total_samples_per_pixel_recip);
                alpha *= total_samples_per_pixel_recip;

                buf.set_pixel_from_vector(x, y, &g, alpha);
            }// for each x
        }// for each y
    }

    // Use runtime dispatching for the image writer to remain flexible
    // (And to test this ;))
    // sets up multi-threading accordingly
    pub fn render(o: &RenderOptions, scene: Arc<Scene>, writer: &mut RGBABufferWriter, 
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
                let opts = *o;
                let tscene = scene.clone();

                count += 1;

                pool.execute(move|| {
                    let mut b = RGBABuffer::new(&ImageRegion { l: x, r: x + CHUNK_SIZE, 
                                                           b: y, t: y + CHUNK_SIZE });

                    Renderer::render_region(&opts, tscene.deref(), &mut b);

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

pub enum FileOrAnyWriter<'a> {
    AnyWriter(old_io::LineBufferedWriter<old_io::stdio::StdWriter>),
    FileWriter(old_io::BufferedWriter<old_io::File>),
}

pub struct PPMStdoutRGBABufferWriter<'a> {
    out: &'a mut FileOrAnyWriter<'a>,
    width: Option<u16>,
    height: Option<u16>,
    image: Option<RGBABuffer>,
    rgb: bool,
}

// It's required to mark it unsafe, as the compiler apparently can't verify 
// that our `out` reference is still valid (even though the bounds say just that)
#[unsafe_destructor]
impl<'a> Drop for PPMStdoutRGBABufferWriter<'a> {
    fn drop(&mut self) {
        // In tty mode, we flush exactly once
        if !self.output_is_file() {
            self.write_buffer_with_header();
        }
    }
}

impl<'a> PPMStdoutRGBABufferWriter<'a> {
    pub fn new(write_rgb: bool, writer: &'a mut FileOrAnyWriter) -> PPMStdoutRGBABufferWriter<'a> {
        PPMStdoutRGBABufferWriter {out: writer,
                                   image: None,
                                   width: None,
                                   height: None,
                                   rgb: write_rgb }
    }
}

impl<'a> PPMStdoutRGBABufferWriter<'a> {

    fn output_is_file(&self) -> bool {
        match *self.out {
            FileOrAnyWriter::FileWriter(ref w) => true,
            _ => false,
        }
    }

    fn write_buffer_with_header(&mut self) {
        let out: &mut old_io::Writer = match *self.out {
            FileOrAnyWriter::FileWriter(ref mut w) => {
                w.get_mut().truncate(0);
                w
            }
            FileOrAnyWriter::AnyWriter(ref mut w) => w
        };

        let mut ptype: &str = "P5";
        if self.rgb {
            ptype = "P6"
        }
        out.write_line(ptype).unwrap();
        out.write_line(&format!("{} {}", self.width.expect("begin() called"), 
                                         self.height.expect("begin() called"))[]).unwrap();
        out.write_line("255").unwrap();

        // We always write our entire buffer - it will just be zero initially
        

        let buf = self.image.as_ref().unwrap().buffer();
        // Can't write entire buffer :( thanks to alpha channel.
        for po in range_step(0, buf.len(), RGBABuffer::components()) {
            let b = &buf[po .. po + 3];

            if self.rgb {
                out.write_all(b).unwrap();
            } else {
                let avg = ((b[0] as f32 + b[1] as f32 + b[2] as f32) / 3.0f32) as u8;
                out.write_u8(avg).unwrap();
            }
        }
    }
}

impl<'a> RGBABufferWriter for PPMStdoutRGBABufferWriter<'a> {
    fn begin(&mut self, x: u16, y: u16) {
        self.width = Some(x);
        self.height= Some(y);
        self.image = Some(RGBABuffer::new(&ImageRegion { l: 0, r: x, b: 0, t: y }));
    }

    fn write_rgba_buffer(&mut self, buffer: &RGBABuffer) {
        self.image.as_mut().unwrap().set_pixels_from_buffer(buffer);

        // Flush full image right away
        if self.output_is_file() {
            self.write_buffer_with_header();
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use std::sync::{TaskPool, Arc};
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
        let s: Arc<Scene> = Arc::new(Default::default());
        let pool = TaskPool::new(1);
        let options = RenderOptions { width: W as u16, height: H as u16, samples_per_pixel: 2 };

        let mut dw: DummyWriter = Default::default();
        Renderer::render(&options, s.clone(), &mut dw, &pool);

        assert!(dw.begin_called);
        assert_eq!(dw.write_count, 2);
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
        let s: Arc<Scene> = Arc::new(Default::default());
        let options = RenderOptions { width: H as u16, height: H as u16, samples_per_pixel: SPP as u16 };

        let mut dw: DummyWriter = Default::default();
        b.iter(|| {
            Renderer::render(&options, s.clone(), &mut dw, &pool);
        });
        b.bytes += (H * H * SPP * SPP) as u64;
    }
}