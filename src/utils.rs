use std::fs::{copy, create_dir_all, metadata, read_dir, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use crate::error::Result;


pub fn create_file(path: &Path, content: &str) -> Result<()> {
    if let Some(p) = path.parent() {
        create_dir_all(p)?;
    }
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

