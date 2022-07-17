use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub optimize: bool,
    pub golf: bool,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Config {{ optimize: {}, golf: {} }}",
            self.optimize, self.golf
        )
    }
}
