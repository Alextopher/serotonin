use include_dir::{include_dir, Dir};

pub static LIBRARIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/libraries");