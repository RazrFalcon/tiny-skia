#![feature(test)]

extern crate test;

#[rustfmt::skip]
#[cfg(test)]
mod blend;
#[rustfmt::skip]
#[cfg(test)]
mod clip;
#[rustfmt::skip]
#[cfg(test)]
mod fill;
#[rustfmt::skip]
#[cfg(test)]
mod gradients;
#[rustfmt::skip]
#[cfg(test)]
mod hairline;
#[rustfmt::skip]
#[cfg(test)]
mod patterns;
#[rustfmt::skip]
#[cfg(test)]
mod png_io;
#[rustfmt::skip]
#[cfg(test)]
mod spiral;

fn main() {}
