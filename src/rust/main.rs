#![feature(env)]
extern crate tracer;

use tracer::{Scene, Renderer, PPMStdoutRGBABufferWriter};
use std::default::Default;
use std::env;

#[allow(dead_code)]
fn main() {
    let s: Scene = Default::default();
    let r = Renderer { width: 1024,
                       height: 1024,
                       samples_per_pixel: 1 };

    r.render(&s, &mut PPMStdoutRGBABufferWriter::new(false));
    env::set_exit_status(0);
}