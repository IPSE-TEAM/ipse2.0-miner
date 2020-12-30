use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path};
use crate::error::Result;


pub fn create_file(path: &Path, content: &str) -> Result<()> {
    if let Some(p) = path.parent() {
        create_dir_all(p)?;
    }
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

