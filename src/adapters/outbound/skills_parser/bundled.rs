use include_dir::{Dir, include_dir};

pub static BUNDLED_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills");
