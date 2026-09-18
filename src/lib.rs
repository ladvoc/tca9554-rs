#![doc = include_str!("../README.md")]
#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(unused_must_use)]

mod address;
mod driver;
#[cfg(feature = "interrupt")]
mod interrupt;
mod pin;

pub use address::Address;
pub use driver::Tca9554;

