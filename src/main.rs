use core::{panic, time, slice::SlicePattern};
use std::hash;

use chrono::Utc;
use libp2p::multihash::StatefulHasher;
use log::{error, warn, info};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::arch::aarch64::ST;

pub struct App {
    pub blocks: Vec,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String, //should contain all txs
    pub nonce: u64,
}

impl App {
    fn new() -> Self {
        Self { blocks: vec![] }
    }
    fn genesis(&mut self) {
        let genesis_block = Block {
            id: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: String::from("genesis"),
            data: String::from("genesis!"),
            nonce: 2836,
            hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43",
        };
        self.blocks.push(genesis_block);
    }

    fn try_add_block(&mut self, block: Block) {
        let latest_block = self.blocks.last().expect("");
        if self.is_block_valid(&block, latest_block) {
            self.blocks.push(block);
        } else {
            error!("cannot add block - invalid");
        }
    }

    fn is_block_valid(&self, block: Block, previous_block: &Block) {
        if previous_block.hash != block.previous_hash {
            warn!("block with id: {} has wrong previous hash", block.id);
            return false;
        } else if !hash_to_binary_representation(
            &hex::decode(&block.hash).expect("can decode from hex"),
        )
        .starts_with(DIFFICULTY_PREFIX)
        {
            warn!("block with id: {} has invalid", block.id);
            return false;
        } else if block.id != previous_block.id + 1 {
            warn!(
                "block with id : {} is not the next block after latest {} ",
                block.id, previous_block.id
            )
            return false; 
        }
        else if hex::encode(calculate_hash(
            block.id,
            block.timestamp,
            &block.previous_hash,
            &block.data ,
            block.nonce,
        )) != block.hash
       {
           warn!("block with id: {} has invalid hash , block.id", block.id);
        return false;
       }
       true
    }

    fn is_chain_valid(&self, chain: &[Block]) -> bool {
        for i in 0..chain.len() {
            if i == 0 {
                continue;
            }
        
            let first = chain.get(i - 1).expect("has to exist");
            let second = chain.get(i).expect("has to exist");
            if !self.is_block_valid(second,first) {
                return false;
            }
        }   
        true
    }

    fn choose_chain(&mut self, local: Vec, remote: Vec) -> Vec {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len() {
                local
            }
            else {
                remote
            }
        }
        else if is_remote_valid && !is_local_valid {
            remote
        }
        else if !is_remote_valid && is_local_valid {
            local
        }
        else {
            panic!("local and remote both chains are invalid");
        }
    }
}

impl Block {
    pub fn new(id: u64, previous_hash: String, data: String) -> Self {
        let now = Utc::now();
        let (nonce, hash) = mine_block(id, now.timestamp(), &previous_hash, &data);
        Self {
            id,
            hash,
            timestamp: now.timestamp(),
            previous_hash,
            data,
            nonce,
        }
    }

    pub fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &str) ->(u8,String) {
        info!("minning block !");

        let mut nonce = 0;
        loop {
            if nonce % 100000 == 0 {
                info!("nonce: {}",nonce);
            }
            let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
            let binary_hash = hash_to_binary_representation(&hash);
            if binary_hash.starts_with(DIFFICULTY_PREFIX) {
                info!(
                    "mined! nonce: {}, hash: {}, binary_hash:{}",
                    nonce,
                    hex::encode(&hash),
                    binary_hash
                );
                return (nonce, hex::encode(hash));
            }
            nonce += 1;
        }
    }
}

fn calculate_hash(id: u64, timestamp: i64, previous_hash: &str, data: &str, nonce: u64) -> Vec<u8> {
    let data = serde_json::json! ({
        "id": id,
        "previous_hash": previous_hash,
        "data": data,
        "timestamp": timestamp,
        "nonce": nonce
    });
    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    hasher.finalize().as_slice().to_owned()

}

const DIFFICULTY_PREFIX: &str = "00";
fn hash_to_binary_representation(hash: &[u8]) -> String {
    let mut res: String = String::default();
    for c in hash {
        res.push_str(&format!("{:b}", c))
    }
    res
}
fn main() {
    println!("")
}
