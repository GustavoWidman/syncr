use blake3;
use std::io::Result;

pub fn hash_file(path: impl AsRef<std::path::Path>) -> Result<blake3::Hash> {
    Ok(blake3::Hasher::new().update_mmap_rayon(&path)?.finalize())
}
