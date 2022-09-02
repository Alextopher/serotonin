#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub(crate) verbose: bool,
    pub(crate) timings: bool,
}

impl Config {
    pub fn new(verbose: bool, timings: bool) -> Self {
        Self { verbose, timings }
    }

    pub fn default() -> Self {
        Self::new(false, false)
    }
}
