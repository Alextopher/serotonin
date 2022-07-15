extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod bfoptimizer;
pub mod parser;
pub mod stdlib;

#[cfg(test)]
mod tests;
