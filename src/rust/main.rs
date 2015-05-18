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

use threadpool::ThreadPool;
use clap::{App}

const USAGE: &'static str = 
"Usage: rtrace [options] (<OUTPUT-FILE>|-)
       rtrace --help

Options:
--width <X>                    The width of the output image [default: 1024]
--height <Y>                   The height of the output image [default: 1024]
--samples-per-pixel <SAMPLES>  Amount of samples per pixel. 4 means 16 over-samples [default: 1]
--num-cores <NUM_CORES>        Amount of cores to do the rendering on [default: 1]
                               If this is not set, you may also use the RTRACEMAXPROCS
                               environment variable, e.g. RTRACEMAXPROCS=4.
                               The commandline always overrides environment variables.

<OUTPUT-FILE>|-     Either a file with .tga extension, or - to write file to stdout";


#[allow(dead_code)]
fn main() {
    let s: Arc<Scene> = Arc::new(Default::default());
    let nc_from_env = env::var("RTRACEMAXPROCS").ok()
                                                .unwrap_or("1".to_string())
                                                .parse::<usize>().ok()
                                                .unwrap_or(1);

    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let pool: ThreadPool  = ThreadPool::new(if args.flag_num_cores > 1
                                            { args.flag_num_cores } else { nc_from_env });

    let mut output = if args.arg_OUTPUT_FILE != "-" {
        let p = Path::new(&args.arg_OUTPUT_FILE);
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

    let options = RenderOptions {   width: args.flag_width,
                                    height: args.flag_height,
                                    samples_per_pixel: args.flag_samples_per_pixel };

    Renderer::render(&options, s.clone(), 
                     &mut PPMStdoutRGBABufferWriter::new(true, &mut output), &pool);
    env::set_exit_status(0);
}