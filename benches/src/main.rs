#![feature(test)]

extern crate test;

#[cfg(test)] mod blend;
#[cfg(test)] mod clip;
#[cfg(test)] mod fill_aa;
#[cfg(test)] mod fill_all;
#[cfg(test)] mod fill_rect;
#[cfg(test)] mod gradients;
#[cfg(test)] mod hairline;
#[cfg(test)] mod memset_fill;
#[cfg(test)] mod pattern;
#[cfg(test)] mod png_io;
#[cfg(test)] mod spiral;

fn main() {}
