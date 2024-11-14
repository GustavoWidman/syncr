use anyhow::{Context, Result};
use std::{fs::File, io::Seek};

use fast_rsync::{apply, diff, Signature, SignatureOptions};

use super::utils::extract_file_contents;

pub fn apply_delta(file: &mut File, delta: Vec<u8>, new_file_len: u64) -> Result<()> {
    let (file_contents, _) = extract_file_contents(file)?;

    file.seek(std::io::SeekFrom::Start(0))
        .context("Failed to seek to start of file")?;
    file.set_len(new_file_len)
        .context("Failed to set file length")?;
    apply(&file_contents, &delta, file).context("Failed to apply delta")?;

    Ok(())
}

pub fn calculate_delta(file: &mut File, serialized_signature: Vec<u8>) -> Result<(Vec<u8>, u64)> {
    let deserialized = Signature::deserialize(serialized_signature.into())
        .context("Failed to deserialize signature")?;
    let signature = deserialized.index();

    let (file_contents, file_len) = extract_file_contents(file)?;

    let mut delta_buf = Vec::new(); // dynamically allocated
    delta_buf
        .try_reserve(file_len as usize)
        .context("Failed to allocate memory for delta buffer")?;

    diff(&signature, &file_contents, &mut delta_buf).context("Failed to calculate delta")?;

    Ok((delta_buf, file_len))
}
