use anyhow::{Context, Result};
use memmap2::Mmap;
use std::{fs::File, io::Write, path::Path};
use tempfile::NamedTempFile;

use fast_rsync::{Signature, apply, diff};

pub fn apply_delta<P>(file_path: P, delta: Vec<u8>) -> Result<()>
where
    P: AsRef<Path>,
{
    let file = File::options()
        .read(true)
        .open(&file_path)
        .context("Failed to open the original file for reading")?;

    let mmap = unsafe { Mmap::map(&file)? };
    let mut temp_file = NamedTempFile::new()?;

    apply(&mmap, &delta, &mut temp_file).context("Failed to apply delta")?;

    temp_file
        .flush()
        .context("Failed to flush temporary file")?;

    temp_file
        .persist(&file_path)
        .context("Failed to persist temporary file")?;

    Ok(())
}

pub fn calculate_delta(file: &mut File, serialized_signature: Vec<u8>) -> Result<(Vec<u8>, usize)> {
    let deserialized = Signature::deserialize(serialized_signature.into())
        .context("Failed to deserialize signature")?;
    let signature = deserialized.index();

    let mmap = unsafe { Mmap::map(&*file)? };

    let mut delta_buf = Vec::new();
    diff(&signature, &mmap, &mut delta_buf).context("Failed to calculate delta")?;

    Ok((delta_buf, mmap.len()))
}
