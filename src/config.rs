#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub(crate) verbose: bool,
    pub(crate) timings: bool,
    pub(crate) optimize: bool,
}

impl Config {
    pub fn new(verbose: bool, timings: bool, optimize: bool) -> Self {
        Self {
            verbose,
            timings,
            optimize,
        }
    }

    pub fn default() -> Self {
        Self::new(false, false, false)
    }
}
