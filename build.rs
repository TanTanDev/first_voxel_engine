use anyhow::*;
use std::path::PathBuf;

use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

// copy resource folder to OUT_DIR
fn copy_res() -> Result<()> {
    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=res/*");

    let out_dir = env::var("OUT_DIR")?;
    let out_dir = PathBuf::from(out_dir);
    let out_dir = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    println!("out dir: {:?}", out_dir);
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("res/");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;
    Ok(())
}

fn main() -> Result<()> {
    copy_res()?;
    Ok(())
}
