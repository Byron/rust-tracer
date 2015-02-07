#![feature(env)]
extern crate tracer;

use tracer::{Scene, Renderer, RenderOptions, PPMStdoutRGBABufferWriter};
use std::default::Default;
use std::env;
use std::sync::TaskPool;

#[allow(dead_code)]
fn main() {
    let s: Scene = Default::default();
    let pool: TaskPool  = TaskPool::new(env::var_string("RTRACE_MAX_PROCS")
                                                           .ok()
                                                           .unwrap_or("1".to_string())
                                                           .parse::<usize>().ok()
                                                           .unwrap_or(1));
    let options = RenderOptions { width: 1024,
                                  height: 1024,
                                  samples_per_pixel: 1 };

    Renderer::render(&options, &s, &mut PPMStdoutRGBABufferWriter::new(false), &pool);
    env::set_exit_status(0);
}