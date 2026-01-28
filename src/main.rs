pub mod network;
pub mod utils;
pub mod block;
pub mod database;
mod client;

use client::Client;
fn main() {
    let _client = Client::new();
    loop {
        std::thread::park();
    }
}