extern crate tracer;

use tracer::{Scene, Renderer, PPMStdoutPixelWriter};
use std::default::Default;
use std::os;

fn main() {
    let s: Scene = Default::default();
    let r = Renderer { width: 1024,
                       height: 1024,
                       samples_per_pixel: 1 };

    r.render(&s, &mut PPMStdoutPixelWriter::new(false));
    os::set_exit_status(0);
}