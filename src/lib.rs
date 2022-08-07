#![feature(pointer_byte_offsets)]

extern crate clap;
extern crate pest;
extern crate pest_derive;

pub(crate) mod bfoptimizer;
pub mod config;
pub(crate) mod definition;
pub(crate) mod gen;
pub mod parser;
pub(crate) mod semantic;
// pub mod typ;

#[cfg(test)]
mod tests;
