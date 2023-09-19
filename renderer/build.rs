use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=assets/*");

    let base_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = base_dir.join("../public");

    let paths_to_copy = vec![base_dir.join("assets")];

    let copy_options = CopyOptions {
        overwrite: true,
        ..Default::default()
    };

    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}
