use std::fs::File;

use crate::model::CompressionTree;
use fast_rsync::{Signature, SignatureOptions};
use memmap2::Mmap;

pub fn calculate_signature(
    file: &mut File,
    predictor: &mut CompressionTree,
) -> anyhow::Result<(Vec<u8>, u32)> {
    let mmap = unsafe { Mmap::map(&*file)? };
    let file_len = mmap.len();

    let predicted_block_size = predictor.wonderful_predict(file_len);

    //* temporary replacement for testing :)
    // let predicted_block_size = 4096;

    let options = SignatureOptions {
        block_size: predicted_block_size,
        crypto_hash_size: 8,
    };

    Ok((
        Signature::calculate(&mmap, options).into_serialized(),
        predicted_block_size,
    ))
}
