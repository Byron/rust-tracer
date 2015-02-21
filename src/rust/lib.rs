#![feature(std_misc,core,old_io,collections,std_misc,unsafe_destructor)]
#![feature(test)]

mod vec;
mod primitive;
mod group;
mod render;

pub use render::{Scene, Renderer, RenderOptions, PPMStdoutRGBABufferWriter, FileOrAnyWriter};

