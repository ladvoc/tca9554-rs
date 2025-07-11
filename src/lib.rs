#![doc = include_str!("../README.md")]

#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(unused_must_use)]

mod driver;
mod address;

pub use driver::Tca9554;
pub use address::Address;