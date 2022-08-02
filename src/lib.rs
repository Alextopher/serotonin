extern crate clap;
extern crate pest;
extern crate pest_derive;

pub mod bfoptimizer;
pub mod config;
pub mod parser;
pub mod semantic;
pub mod stdlib;
// pub mod typ;

#[cfg(test)]
mod tests;
