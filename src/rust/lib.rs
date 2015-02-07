#![feature(std_misc,core,io,collections,std_misc)]
#![feature(test)]

mod tests;
mod vec;
mod primitive;
mod group;
mod render;

pub use render::{Scene, Renderer, RenderOptions, PPMStdoutRGBABufferWriter};

