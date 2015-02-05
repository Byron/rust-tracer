/// Implements the actual raytracer which produces the final image
use std::num::Float;
use super::vec::Vector;

pub trait ImageWriter<T>: Drop {
    /// To be called before writing the first pixel
    /// x and y are the max image resolution
    fn begin(&mut self, x: u16, y: u16);
    fn write_pixel_at(&mut self, x: u16, y: u16, color: &Vector<T>);
}

pub struct Renderer<T> {
    pub width: u16,
    pub height: u16,
    pub samples_per_pixel: u16,
    pub eye: Vector<T>,
}

impl<T> Renderer<T> {

    // Use runtime dispatching for the image writer to remain flexible
    // (And to test this ;))
    fn render(&self, writer: &mut ImageWriter<T>) {

    }
}

struct PPMImageWriter<'a> {
    out: &'a mut Writer,
    rgb: bool,
}

impl<'a> PPMImageWriter<'a> {
    pub fn new(out: &mut Writer, write_RGB: bool) {
        PPMImageWriter { out: out, rgb: write_RGB };
    }
}

impl<'a, T: Float> ImageWriter<T> for PPMImageWriter<'a> {
    fn begin(&mut self, x: u16, y: u16) {
        let mut ptype: &str = "P5";
        if self.write_RGB {
            ptype = "P6"
        }
        self.out.write_line(ptype);
        self.out.write_line(format!("{} {}", x, y).as_str());
        self.out.write_line("255");
    }

    fn write_pixel_at(&mut self, x: u16, y: u16, color: &Vector<T>) {

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::vec::Vector;
    use super::super::group::SphericalGroup;

    #[test]
    fn basic_rendering() {
        let r = Renderer { width: 1024u16,
                           height: 1024u16,
                           samples_per_pixel: 1,
                           eye: Vector { x:0.0f32, y: 0.0, z: -4.0 } };

    }
}