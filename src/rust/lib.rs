#![feature(step_by)]
#![cfg_attr(test, feature(test))]

mod vec;
mod primitive;
mod group;
mod render;

pub use render::{Scene, Renderer, RenderOptions, PPMStdoutRGBABufferWriter, FileOrAnyWriter};
