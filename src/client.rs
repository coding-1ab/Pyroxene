use crate::block::block::Block;
use crate::network::UdpBroadcast;
use crate::utils::{generate_keys, spawn_miner};

use crate::block::transaction::Transaction;
use rand::random;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{io, thread};
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::{Local, TimeZone};
use rsa::RsaPrivateKey;
use crate::block::Address;

pub struct Client{
    pub id: usize,
    private_key: RsaPrivateKey,
    pub network: UdpBroadcast,
    pub data_sender: Sender<Vec<Transaction>>,
    data_receiver: Receiver<Vec<Transaction>>,
    block_sender: Sender<Block>,
    block_receiver: Receiver<Block>,
    cancel_sender: Sender<()>,
    cancel_receiver: Receiver<()>,
    stdin_sender: Sender<String>,
    stdin_receiver: Receiver<String>,
    chain: Arc<Mutex<Vec<Block>>>
}

impl Client{
    pub fn new() -> Self {
        let (data_sender, data_receiver) = channel();
        let (block_sender, block_receiver) = channel::<Block>();
        let (cancel_sender, cancel_receiver) = channel::<()>();
        let (stdin_sender, stdin_receiver) = channel();
        let id: usize = random();
        let chain = Arc::new(Mutex::new(Vec::new()));

        println!("Client ID: {}", id);

        let network = UdpBroadcast::new(chain.clone()).unwrap();
        let private_key = generate_keys();

        Self {
            id,
            private_key,
            network,
            data_sender,
            data_receiver,
            block_sender,
            block_receiver,
            cancel_sender,
            cancel_receiver,
            stdin_sender,
            stdin_receiver,
            chain,
        }
    }

    pub fn start(self) {
        let zero_length = 8;

        let mut my_address: Address = [0u8; 32];
        let id_bytes = self.id.to_be_bytes();
        my_address[..id_bytes.len()].copy_from_slice(&id_bytes);

        spawn_stdin(self.stdin_sender);
        spawn_chain_watcher(self.chain.clone());
        // miner
        spawn_miner(
            self.block_sender,
            self.chain.clone(),
            self.cancel_receiver,
            self.data_receiver,
            zero_length,
        );

        // network
        self.network.spawn(
            self.block_receiver,
            self.cancel_sender,
        );

        let mut pending_transactions: Vec<Transaction> = Vec::new();
        println!("Welcome to Pyroxene!");
        println!("\n=== Blockchain Client ID: {} ===", self.id);
        println!("내 주소(Hex): {}", hex::encode(my_address));
        println!("명령어:");
        println!("  chain               - 현재 체인 상태 보기");
        println!("  add <address> <data>    - 트랜잭션 추가");
        println!("  mine                - 모인 트랜잭션으로 채굴 시작");
        println!("  status              - 현재 대기 중인 트랜잭션 확인");
        println!("  exit                - 종료");
        println!("================================\n");

        while let Ok(line) = self.stdin_receiver.recv() {
            let parts: Vec<&str> = line.trim().splitn(3, ' ').collect();
            let command = parts[0];

            match command {
                "chain" => {
                    let chain = self.chain.lock().unwrap();
                    println!("Current chain(Count: {})", chain.len());
                    for block in chain.iter(){
                        let timestamp = Local.timestamp_opt(block.block_header.timestamp, 0).unwrap();
                        println!("Block #{} - Txs: {} (Timestamp: {})", block.block_header.height, block.txs.len(), timestamp);
                    }
                }

                "add" => {
                    if parts.len() < 3 {
                        println!("잘못된 사용법입니다.\n사용법: add <address> <data>");
                        continue;
                    }

                    let to_res = parse_address(parts[1]);
                    let value_res = parts[2].parse::<u128>();

                    match (to_res, value_res) {
                        (Ok(to), Ok(value)) => {
                            let tx = Transaction::new(to, my_address, value);
                            pending_transactions.push(tx);
                            println!("추가됨: [To: {}..., Value: {}] (대기열: {}개)", hex::encode(&to[..4]), value, pending_transactions.len());
                        }
                        (Err(e), _) => println!("주소 오류: {}", e),
                        (_, Err(_)) => println!("금액은 숫자여야 합니다."),
                    }
                }

                "mine" => {
                    if pending_transactions.is_empty(){
                        println!("채굴할 트랜잭션이 없습니다. `add` 를 비롯한 타 명령어를 먼저 써주세요.");
                    }
                    else{
                        let count = pending_transactions.len();
                        let txs_to_send = std::mem::take(&mut pending_transactions);

                        if self.data_sender.send(txs_to_send).is_ok() {
                            println!("{}개의 트랜잭션을 마이너에게 전달했습니다. 채굴을 시작합니다.", count);
                        }
                        else {
                            println!("마이너에게 전달을 실패했습니다. 무슨 오류인지는 모르겠습니다? 아하하..");
                        }
                    }
                }

                "status" => {
                    println!("현재 대기열: {}개", pending_transactions.len());
                    for (i, tx) in pending_transactions.iter().enumerate() {
                        println!("  {}. To: {}... | Value: {}", i + 1, hex::encode(&tx.to[..4]) ,tx.value);
                    }
                }

                "exit" => break,
                _ => println!("알 수 없는 명령어: [chain, add, mine, status, exit]만을 써주세요."),
            }
        }

    }
}

fn parse_address(s: &str) -> Result<[u8; 32], String> {
    let s = s.strip_prefix("0x").unwrap_or(s);

    let mut bytes = [0u8; 32];
    let decoded = hex::decode(s).map_err(|_| "Invalid Hex string")?;

    if decoded.len() > 32 {
        return Err("Address too long (max 32 bytes)".to_string());
    }
    bytes[..decoded.len()].copy_from_slice(&decoded);
    Ok(bytes)
}

pub fn spawn_stdin(tx: Sender<String>) {
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(cmd) => {
                    if tx.send(cmd).is_err() {
                        break; // receiver 죽으면 종료
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn spawn_chain_watcher(chain: Arc<Mutex<Vec<Block>>>) {
    thread::spawn(move || {
        let mut last_len = 0usize;

        loop {
            {
                let chain_guard = chain.lock().unwrap();
                let current_len = chain_guard.len();

                if current_len > last_len {
                    let new_blocks = current_len - last_len;

                    println!(
                        "\n🧱 Chain updated! {} new block(s). total = {}\n",
                        new_blocks,
                        current_len
                    );

                    // 마지막 블록 정보도 보고 싶으면
                    if let Some(block) = chain_guard.last() {
                        println!(
                            "  height: {}\n  txs: {}\n",
                            block.block_header.height,
                            block.txs.len()
                        );
                    }

                    last_len = current_len;
                }
            }

            // 너무 자주 lock 안 걸리게
            thread::sleep(Duration::from_millis(300));
        }
    });
}