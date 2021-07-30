#![no_std]
#![feature(asm)]

#[cfg(test)]
#[macro_use]
extern crate std;

pub use port::Port;

mod port;
