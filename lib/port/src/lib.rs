#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

pub use port::{Port, PortReadOnly, PortWriteOnly};

mod port;
