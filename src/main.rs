#![allow(warnings)]

mod client;
mod common;
mod model;
mod server;
mod utils;

use anyhow::{Context, Result, bail};
use fast_rsync::{Signature, SignatureOptions, apply, diff};
use ignore::WalkBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
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

use common::stream::SecureStream;
use common::sync;
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
    let start_dirs = vec![dirs::home_dir().expect("Unable to get home directory")];

    let start = Instant::now();
    let paths = RwLock::new(Vec::new());

    start_dirs.into_par_iter().for_each(|dir| {
        let walker = WalkBuilder::new(dir)
            .follow_links(true)
            .hidden(false)
            .filter_entry(|entry| {
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_str().unwrap();
                (path.is_dir() && !file_name.starts_with("."))
                    || (path.is_file() && file_name == ".sync")
            })
            .build_parallel();

        walker.run(|| {
            let paths = &paths;
            Box::new(move |result| {
                if let Ok(entry) = result {
                    if entry.path().is_file()
                        && entry.path().file_name() == Some(OsStr::new(".sync"))
                    {
                        let mut paths_guard = paths.write().unwrap(); // Acquire write lock
                        paths_guard.push(entry.into_path());
                    }
                }

                ignore::WalkState::Continue
            })
        });
    });

    let elapsed = start.elapsed();
    // let paths = Arc::try_unwrap(paths)
    //     .expect("Arc unwrap failed")
    //     .into_inner()
    //     .unwrap();
    let paths = paths.into_inner().unwrap();

    println!("Found {} .sync files in {:?}", paths.len(), elapsed);
    if paths.is_empty() {
        println!("No .sync files found.");
    } else {
        for path in paths {
            println!("Found .sync file at: {:?}", path);
        }
    }
}

async fn client_main() {
    let mut client = client::Client::connect("127.0.0.1:8123").await.unwrap();

    client.run().await;
}

async fn server_main() {
    let mut server = server::Server::bind(8123).await.unwrap();
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
        .open("test/data/small/old.txt")
        // .open("test/data/medium/old.txt")
        // .open("test/data/big/old.txt")
        .unwrap();
    let (signature_encoded, predicted_block_size) =
        sync::calculate_signature(&mut old_file, &mut predictor).unwrap();
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

    let (delta, new_file_len) = sync::calculate_delta(&mut new_file, signature_encoded).unwrap();

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
