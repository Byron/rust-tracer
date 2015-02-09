#![feature(std_misc,core,io,collections,std_misc,unsafe_destructor,path,env,plugin)]
extern crate tracer;

extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;
#[plugin] #[no_link] extern crate docopt_macros;

use tracer::{Scene, Renderer, RenderOptions, PPMStdoutRGBABufferWriter, FileOrAnyWriter};
use std::default::Default;
use std::env;
use std::sync::{TaskPool, Arc};
use std::old_io;
use std::old_path::Path;


docopt!(Args derive Show, "
Usage: rtrace (<OUTPUT-FILE>|-)
       rtrace --help

<OUTPUT-FILE>|-     Either a file with .tga extension, or - to write file to stdout
");


#[allow(dead_code)]
fn main() {
    let s: Arc<Scene> = Arc::new(Default::default());
    let pool: TaskPool  = TaskPool::new(env::var_string("RTRACEMAXPROCS")
                                                            .ok()
                                                            .unwrap_or("1".to_string())
                                                            .parse::<usize>().ok()
                                                            .unwrap_or(1));

    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let mut output = if args.arg_OUTPUT_FILE != "-" {
        let p = Path::new(args.arg_OUTPUT_FILE);
        if p.extension().unwrap_or(b".UNSET") != b"tga" {
            println!("Output file '{}' must have the tga extension, e.g. {}", p.as_str().unwrap(), p.with_extension(b"tga").as_str().unwrap());
            return;
        }
        let open_file = old_io::BufferedWriter::new(old_io::File::create(&p)
                                                    .ok()
                                                    .expect("Could not open output file"));
        FileOrAnyWriter::FileWriter(open_file)
    } else {
        FileOrAnyWriter::AnyWriter(old_io::stdout())
    };

    let options = RenderOptions {   width: 1024,
                                    height: 1024,
                                    samples_per_pixel: 1 };

    Renderer::render(&options, s.clone(), 
                     &mut PPMStdoutRGBABufferWriter::new(false, &mut output), &pool);
    env::set_exit_status(0);
}