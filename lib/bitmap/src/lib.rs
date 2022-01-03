#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

mod bitmap;

pub use crate::bitmap::Bitmap;
