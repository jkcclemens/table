#![feature(box_syntax)]
#![cfg_attr(test, feature(test))]

//! A file-based, positional collection.
//!
//! `table` keeps only an index reference in memory at all times. Data is read from the disk on
//! read, and it is written to the disk on write.
//!
//! Data can only be mutated at the end of the collection. This means the only ways to add or remove
//! data are `push` and `pop`, respectively.
//!
//! `table` creates two files for each opened collection: an index file and a data file. The index
//! file contains a vector of lengths, while the data file contains MessagePack-serialized data
//! corresponding to the lengths.

#[macro_use]
extern crate failure_derive;
extern crate failure;
extern crate rmp_serde as serde_msgpack;
extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod iter;
pub mod typed;
pub mod untyped;
#[cfg(test)]
extern crate test as std_test;
#[cfg(test)]
mod test;

pub use self::typed::TypedTable;
pub use self::untyped::Table;
