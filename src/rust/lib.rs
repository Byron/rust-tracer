#![feature(std_misc,core)]
#![feature(test)]

mod tests;
mod vec;
mod primitive;
mod group;
mod render;

pub use render::{Scene, Renderer, PPMStdoutPixelWriter};

