extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate clap;

pub mod bfoptimizer;
pub mod config;
pub mod parser;
pub mod stdlib;

#[cfg(test)]
mod tests;
