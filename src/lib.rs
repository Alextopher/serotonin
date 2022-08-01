extern crate clap;
extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod bfoptimizer;
pub mod config;
pub mod parser;
pub mod stdlib;
pub mod typ;

#[cfg(test)]
mod tests;
