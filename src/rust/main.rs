#![cfg(not(test))]

extern crate sphere_tracer;
extern crate threadpool;
extern crate clap;


use sphere_tracer::{Scene, Renderer, RenderOptions, PPMStdoutRGBABufferWriter, FileOrAnyWriter};

use std::default::Default;
use std::env;
use std::sync::Arc;
use std::ffi::OsStr;
use std::{io, fs};
use std::path::Path;
use std::process;

use threadpool::ThreadPool;
use clap::{App, Arg};

#[allow(dead_code)]
fn main() {
    let s: Arc<Scene> = Arc::new(Default::default());
    let nc_from_env = env::var("RTRACEMAXPROCS").ok()
                                                .unwrap_or("1".to_string())
                                                .parse::<usize>().ok()
                                                .unwrap_or(1);

    let args        = App::new("rtrace")
                          .author("Sebastian Thiel <byronimo@mail.com>")
                          .version("0.2.0")
                          .about("A toy-raytracer for rendering a scene with spheres")
                          .args_from_usage(
                            "--width=[X] 'The width of the output image [default: 1024]'
                            --height=[Y] 'The height of the output image [default: 1024]'
                            [ssp] --samples-per-pixel=[SAMPLES]  'Amount of samples per pixel. 4 means 16 over-samples [default: 1]'")
                          .arg(Arg::with_name("numcores")
                                   .long("num-cores")
                                   .takes_value(true)
                                   .help(
                                    "Amount of cores to do the rendering on [default: 1]
                                     If this is not set, you may also use the RTRACEMAXPROCS
                                     environment variable, e.g. RTRACEMAXPROCS=4.
                                     The commandline always overrides environment variables."))
                          .arg(Arg::with_name("output")
                                   .required(true)
                                   .empty_values(false)
                                   .help("Either a file with .tga extension, or - to write file to stdout"))
                           .get_matches();
    let num_cores: usize  = args.value_of("numcores").unwrap_or("1").parse().unwrap();
    let pool: ThreadPool  = ThreadPool::new(if num_cores > 1 { num_cores } else { nc_from_env });

    let output_file = args.value_of("output").unwrap();
    let mut output = if output_file != "-" {
        let p = Path::new(&output_file);
        if p.extension().unwrap_or(OsStr::new(".UNSET")) != "tga" {
            println!("Output file '{}' must have the tga extension, e.g. {}", 
                     p.to_str().unwrap(), 
                     p.with_extension("tga").to_str().unwrap());
            return;
        }
        let open_file = io::BufWriter::new(fs::File::create(&p).unwrap());
        FileOrAnyWriter::FileWriter(open_file)
    } else {
        FileOrAnyWriter::AnyWriter(io::stdout())
    };

    let options = RenderOptions {   width: args.value_of("width").unwrap_or("1024").parse().unwrap(),
                                    height: args.value_of("height").unwrap_or("1024").parse().unwrap(),
                                    samples_per_pixel: args.value_of("ssp").unwrap_or("1").parse().unwrap() };

    Renderer::render(&options, s.clone(), 
                     &mut PPMStdoutRGBABufferWriter::new(true, &mut output), &pool);

    process::exit(0);
}