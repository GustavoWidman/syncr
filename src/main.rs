#![allow(warnings)]

mod client;
mod common;
mod data;
mod model;
mod server;

use anyhow::{Context, Result, bail};
use fast_rsync::{Signature, SignatureOptions, apply, diff};
use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    future,
    io::{Read, Seek},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    thread::panicking,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task,
    time::Instant,
};

use common::sync;
use common::{quick_config, sync::apply_delta};
use model::BlockSizePredictor;

#[tokio::main]
async fn main() {
    // read environment variable MODE
    match env::var("MODE") {
        Ok(mode) => match mode.to_lowercase().as_str() {
            "server" => server_main().await,
            "client" => client_main().await,
            "watch" => watch_main().await,
            "sync" => sync_main().await,
            _ => panic!("Invalid mode specified"),
        },
        Err(_) => sync_main().await,
    }
}

async fn watch_main() {
    let mut config = quick_config!("./syncr.toml").unwrap();

    config.secret = "password".into();

    config.save().unwrap();

    println!("{:?}", config);
    return;
}

async fn client_main() {
    let mut client_cfg = quick_config!("./client.toml").unwrap();
    let mut client = client::Client::connect(Some(client_cfg)).await.unwrap();

    client.run().await;
}

async fn server_main() {
    let mut server_cfg = quick_config!("./server.toml").unwrap();
    let mut server = server::Server::bind(Some(server_cfg)).await.unwrap();
    server.run().await;
}

async fn sync_main() {
    use std::time::Instant;

    //? RUNNING SERVER SIDE
    let server_start = Instant::now();

    let mut predictor = model::initialize!("model.json").expect("Failed to initialize model");

    let mut old_file = File::options()
        .read(true)
        .write(true)
        // .open("test/data/small/old.txt")
        // .open("test/data/medium/old.txt")
        .open("test/data/big/old.txt")
        .unwrap();
    let (signature_encoded, predicted_block_size) =
        sync::calculate_signature(&mut old_file, &mut predictor).unwrap();
    drop(old_file);
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
        // .open("test/data/small/new.txt")
        // .open("test/data/medium/new.txt")
        .open("test/data/big/new.txt")
        .unwrap();

    let (delta, new_file_len) = sync::calculate_delta(&mut new_file, signature_encoded).unwrap();

    let client_elapsed = client_start.elapsed();

    // println!("Delta: {:?}, len: {} bytes", delta, delta.len());

    //* END RUNNING CLIENT SIDE

    // TRANSFER DELTA AND FILE_LEN
    // |
    // V

    //? RUNNING SERVER SIDE
    let final_start = Instant::now();

    let compression_rate: f32 =
        new_file_len as f32 / (delta.len() + 8 + signature_encoded_len) as f32;
    let delta_len = delta.len();

    // let mut old_file = File::options()
    //     .read(true)
    //     .write(true)
    //     // .open("test/data/small/old.txt")
    //     // .open("test/data/medium/old.txt")
    //     .open("test/data/big/old.txt")
    //     .unwrap();

    // let old_file = "test/data/small/old.txt";
    // let old_file = "test/data/medium/old.txt";
    let old_file = "test/data/big/old.txt";

    apply_delta(old_file, delta, new_file_len).unwrap();

    let final_elapsed = final_start.elapsed();

    predictor.tune(new_file_len, predicted_block_size, compression_rate);
    predictor.save();

    //? END RUNNING SERVER SIDE

    // logging time taken
    println!(
        "\nServer elapsed: {:?}\nClient elapsed: {:?}",
        server_elapsed, client_elapsed
    );
    println!("Apply delta elapsed: {:?}", final_elapsed);
    println!("\nServer transfered {} bytes", signature_encoded_len);
    println!("\nClient transfered {} bytes", delta_len + 8);

    println!(
        "\nGiven that the client's file size is {} bytes, we've achieved a **lossless** compression ratio of ~{}x",
        new_file_len,
        compression_rate.round(),
    );
}
