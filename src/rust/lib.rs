#![feature(std_misc,core,io,collections)]
#![feature(test)]

mod tests;
mod vec;
mod primitive;
mod group;
mod render;

pub use render::{Scene, Renderer, PPMStdoutRGBABufferWriter};

