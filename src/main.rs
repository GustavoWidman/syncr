mod client;
mod common;
mod data;
mod model;
mod schema;
mod server;
mod utils;

use client::tray::TrayMenu;
use client::watcher;
use common::config::SyncConfig;
use data::DatabaseDriver;
use log::{LevelFilter, info};
use server::database::ServerDatabase;
use std::{env, fs::File};
use utils::hash::hash_file;
use utils::log::Logger;

use common::sync::{self};
use common::{quick_config, sync::apply_delta};

#[tokio::main]
async fn main() {
    Logger::init(Some(LevelFilter::Info));

    info!("Intializing syncr...");

    // read environment variable MODE
    match env::var("MODE") {
        Ok(mode) => match mode.to_lowercase().as_str() {
            "server" => server_main().await,
            "client" => client_main().await,
            "watch" => watch_main().await,
            "sync" => sync_main().await,
            "tray" => tray_main().await,
            _ => panic!("Invalid mode specified"),
        },
        Err(_) => sync_main().await,
    }
}

async fn tray_main() {
    // todo!();

    let handle = tokio::spawn(async move {
        let tray = TrayMenu::new("./icons/icon.png".into()).unwrap();

        tray.run();
    });

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("Tray menu tick");
        if handle.is_finished() {
            break;
        }
    }

    println!("Tray menu shutdown");
}

async fn watch_main() {
    let sync_config = SyncConfig::read("./syncr.toml".into()).unwrap();

    let watcher = watcher::Watcher::new(sync_config).await.unwrap();

    watcher
        .run(move |event| {
            info!("{:?}: {:?}", event.paths[0], event.kind);
        })
        .await
        .unwrap();

    return;
}

async fn client_main() {
    let client_cfg = quick_config!("./client.toml").unwrap();
    let mut client = client::Client::connect(Some(client_cfg)).await.unwrap();

    client.run().await;
}

async fn server_main() {
    let server_cfg = quick_config!("./server.toml").unwrap();
    let mut server = server::Server::bind(Some(server_cfg)).await.unwrap();
    server.run().await;
}

async fn sync_main() {
    use std::time::Instant;

    // let test_size = "small";
    // let test_size = "medium";
    let test_size = "big";

    let old_test_path = format!("test/data/{}/old.txt", test_size);
    let new_test_path = format!("test/data/{}/new.txt", test_size);

    //? COMMON SIDES

    let common_start = Instant::now();

    let mut database = ServerDatabase::new(None).await.unwrap();
    let mut predictor = model::initialize!(&mut database).unwrap();

    let common_elapsed = common_start.elapsed();

    //? RUNNING SERVER SIDE

    let server_start = Instant::now();

    let mut old_file = File::options()
        .read(true)
        .write(true)
        .open(&old_test_path)
        .unwrap();
    let (signature_encoded, predicted_block_size) =
        sync::calculate_signature(&mut old_file, &mut predictor).unwrap();
    drop(old_file);
    info!("Used block size: {}", predicted_block_size);
    let signature_encoded_len = signature_encoded.len();

    let server_elapsed = server_start.elapsed();

    //? END RUNNING SERVER SIDE

    // TRANSFER SIGNATURE ENCODED
    // |
    // V

    //* RUNNING CLIENT SIDE
    let client_start = Instant::now();

    let mut new_file = File::options().read(true).open(&new_test_path).unwrap();

    let (delta, new_file_len) = sync::calculate_delta(&mut new_file, signature_encoded).unwrap();

    let client_elapsed = client_start.elapsed();

    // info!("Delta: {:?}, len: {} bytes", delta, delta.len());

    //* END RUNNING CLIENT SIDE

    // TRANSFER DELTA AND FILE_LEN
    // |
    // V

    //? RUNNING SERVER SIDE
    let final_start = Instant::now();

    let compression_rate: f32 =
        new_file_len as f32 / (delta.len() + 8 + signature_encoded_len) as f32;
    let delta_len = delta.len();

    apply_delta(&old_test_path, delta).unwrap();

    let final_elapsed = final_start.elapsed();

    // hash both files
    let old_hash = hash_file(&old_test_path).unwrap();
    let new_hash = hash_file(&new_test_path).unwrap();

    // compare hashes
    if old_hash == new_hash {
        info!("Files are identical");
    }

    // undo the apply, copy old.txt.bak to old.txt
    let old_bak_path = format!("test/data/{}/old.txt.bak", test_size);
    std::fs::copy(old_bak_path, old_test_path).unwrap();

    predictor
        .tune(new_file_len, predicted_block_size, compression_rate)
        .unwrap();
    predictor.save(&mut database).await.unwrap();

    //? END RUNNING SERVER SIDE

    // logging time taken
    info!(
        "\nServer elapsed: {:?}\nClient elapsed: {:?}",
        server_elapsed, client_elapsed
    );
    info!("Apply delta elapsed: {:?}", final_elapsed);
    info!("Common initialization time: {:?}", common_elapsed);
    info!("\nServer transfered {} bytes", signature_encoded_len);
    info!("\nClient transfered {} bytes", delta_len + 8);

    info!(
        "\nGiven that the client's file size is {} bytes, we've achieved a **lossless** compression ratio of ~{}x",
        new_file_len,
        compression_rate.round(),
    );
}
