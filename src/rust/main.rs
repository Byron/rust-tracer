#![feature(env,std_misc)]
extern crate tracer;

use tracer::{Scene, Renderer, RenderOptions, PPMStdoutRGBABufferWriter, FileOrAnyWriter};
use std::default::Default;
use std::env;
use std::sync::{TaskPool, Arc};
use std::old_io;
use std::old_path::Path;

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


    // let mut output = old_io::stdout();
    let mut output = if true {
        let p = Path::new("myout.tga");
        let open_file = old_io::BufferedWriter::new(old_io::File::create(&p).ok().expect("Could not open output file"));
        FileOrAnyWriter::FileWriter(open_file)
    } else {
        FileOrAnyWriter::AnyWriter(old_io::stdout())
    };

    Renderer::render(&options, s.clone(), 
                     &mut PPMStdoutRGBABufferWriter::new(false, &mut output), &pool);
    env::set_exit_status(0);
}