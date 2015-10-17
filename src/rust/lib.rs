#![feature(step_by, clone_from_slice)]
#![cfg_attr(test, feature(test))]

mod vec;
mod primitive;
mod group;
mod render;

pub use render::{Scene, Renderer, RenderOptions, PPMStdoutRGBABufferWriter, FileOrAnyWriter};
