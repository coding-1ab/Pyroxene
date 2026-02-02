pub mod network;
pub mod utils;
pub mod block;
pub mod database;
mod client;

use client::Client;
fn main() {
    let client = Client::new();
    client.start();
    loop {
        std::thread::park();
    }
}