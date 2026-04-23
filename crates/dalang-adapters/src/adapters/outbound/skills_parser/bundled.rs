use include_dir::{Dir, include_dir};

// Bundled skills live at the repository root (`skills/`).
pub static BUNDLED_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../skills");
