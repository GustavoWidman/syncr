mod common;
mod model;
mod server;

use anyhow::{bail, Context, Result};
use std::{
    fs::File,
    io::{Read, Seek},
};

use fast_rsync::{apply, diff, Signature, SignatureOptions};

use model::BlockSizePredictor;

fn main() {
    use std::time::Instant;

    //? RUNNING SERVER SIDE
    let server_start = Instant::now();

    let mut predictor = model::initialize!("model.json");

    let mut old_file = File::options()
        .read(true)
        .write(true)
        .open("test/data/small/old.txt")
        // .open("test/data/medium/old.txt")
        // .open("test/data/big/old.txt")
        .unwrap();
    let (signature_encoded, predicted_block_size) =
        calculate_signature(&mut old_file, &mut predictor).unwrap();
    println!("Used block size: {}", predicted_block_size);
    let signature_encoded_len = signature_encoded.len();

    let server_elapsed = server_start.elapsed();

    //? END RUNNING SERVER SIDE

    // TRANSFER SIGNATURE ENCODED
    // |
    // V

    //* RUNNING CLIENT SIDE
    let client_start = Instant::now();

    let mut new_file = File::options()
        .read(true)
        .open("test/data/small/new.txt")
        // .open("test/data/medium/new.txt")
        // .open("test/data/big/new.txt")
        .unwrap();

    let (delta, new_file_len) = calculate_delta(&mut new_file, signature_encoded).unwrap();

    let client_elapsed = client_start.elapsed();

    // println!("Delta: {:?}, len: {} bytes", delta, delta.len());

    //* END RUNNING CLIENT SIDE

    // TRANSFER DELTA AND FILE_LEN
    // |
    // V

    //? RUNNING SERVER SIDE
    // let final_start = Instant::now();

    let compression_rate: f32 =
        new_file_len as f32 / (delta.len() + 8 + signature_encoded_len) as f32;
    let delta_len = delta.len();

    // apply_delta(&mut old_file, delta, new_file_len).unwrap();

    // let final_elapsed = final_start.elapsed();

    predictor.tune(new_file_len, predicted_block_size, compression_rate);
    predictor.save();

    //? END RUNNING SERVER SIDE

    // logging time taken
    println!(
        "Server elapsed: {:?}\nClient elapsed: {:?}",
        server_elapsed, client_elapsed
    );
    println!("\nServer transfered {} bytes", signature_encoded_len);
    println!("\nClient transfered {} bytes", delta_len + 8);

    println!(
        "\nGiven that the client's file size is {} bytes, we've achieved a **lossless** compression ratio of ~{}x",
        new_file_len,
        compression_rate.round(),
    );
}

fn calculate_signature(
    file: &mut File,
    predictor: &mut BlockSizePredictor,
) -> Result<(Vec<u8>, u32)> {
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

fn calculate_delta(file: &mut File, serialized_signature: Vec<u8>) -> Result<(Vec<u8>, u64)> {
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

// ALLOW IT MATE
#[allow(dead_code)]
fn apply_delta(file: &mut File, delta: Vec<u8>, new_file_len: u64) -> Result<()> {
    let (file_contents, _) = extract_file_contents(file)?;

    file.seek(std::io::SeekFrom::Start(0))
        .context("Failed to seek to start of file")?;
    file.set_len(new_file_len)
        .context("Failed to set file length")?;
    apply(&file_contents, &delta, file).context("Failed to apply delta")?;

    Ok(())
}

fn extract_file_contents(file: &mut File) -> Result<(Vec<u8>, u64)> {
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
