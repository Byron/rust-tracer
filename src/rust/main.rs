#![feature(env,std_misc)]
extern crate tracer;

use tracer::{Scene, Renderer, RenderOptions, PPMStdoutRGBABufferWriter};
use std::default::Default;
use std::env;
use std::sync::{TaskPool, Arc};
use std::old_io;

#[allow(dead_code)]
fn main() {
    let s: Arc<Scene> = Arc::new(Default::default());
    let pool: TaskPool  = TaskPool::new(env::var_string("RTRACEMAXPROCS")
                                                           .ok()
                                                           .unwrap_or("1".to_string())
                                                           .parse::<usize>().ok()
                                                           .unwrap_or(1));
    let options = RenderOptions { width: 1024,
                                  height: 1024,
                                  samples_per_pixel: 1 };

    let mut stdout = old_io::stdout();
    Renderer::render(&options, s.clone(), &mut PPMStdoutRGBABufferWriter::new(false, &mut stdout), &pool);
    env::set_exit_status(0);
}