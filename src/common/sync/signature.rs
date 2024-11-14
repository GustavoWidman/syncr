use std::fs::File;

use fast_rsync::{Signature, SignatureOptions};

use super::utils::extract_file_contents;
use crate::model::BlockSizePredictor;

pub fn calculate_signature(
    file: &mut File,
    predictor: &mut BlockSizePredictor,
) -> anyhow::Result<(Vec<u8>, u32)> {
    let (file_contents, file_len) = extract_file_contents(file)?;

    let predicted_block_size = predictor.predict(file_len);
    let (wondered_value, has_wondered) = predictor.wonder(file_len);

    if has_wondered {
        println!(
            "predictor wondered if the value {:?} might be better instead when faced with the question {:?}",
            wondered_value, file_len
        );
    } else {
        println!(
            "predictor has concluded and pondered upon that {:?} is the absolute best value for {:?}",
            predicted_block_size, file_len
        );
    }

    //* temporary replacement for testing :)
    // let predicted_block_size = 4096;

    let options = SignatureOptions {
        block_size: predicted_block_size,
        crypto_hash_size: 8,
    };

    Ok((
        Signature::calculate(&file_contents, options).into_serialized(),
        predicted_block_size,
    ))
}
