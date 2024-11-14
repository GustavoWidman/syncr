use std::{
    fs::File,
    io::{Read, Seek},
};

use anyhow::{bail, Context};

pub fn extract_file_contents(file: &mut File) -> anyhow::Result<(Vec<u8>, u64)> {
    let file_len = file
        .metadata()
        .context("Failed to get file metadata")?
        .len();

    if file_len > usize::MAX as u64 {
        bail!("File is too large to process on this system!");
    }

    //? This is about the most we can do to avoid OOM errors since
    //? the fast_rsync crate works with buffers and not readers (terrible practice)
    let mut file_contents = Vec::with_capacity(file_len as usize);
    file_contents
        .try_reserve(file_len as usize)
        .context("Failed to allocate memory for file contents")?;
    file.seek(std::io::SeekFrom::Start(0))
        .context("Failed to seek to start of file")?;
    file.read_to_end(&mut file_contents)
        .context("Failed to read file contents")?;

    Ok((file_contents, file_len))
}
