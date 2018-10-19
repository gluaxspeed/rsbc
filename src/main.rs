#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate crypto;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;


use std::ops::Deref;
use std::sync::Mutex;
use chrono::offset::Utc;
use chrono::{DateTime};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::time::{SystemTime};
use rocket::State;
use rocket_contrib::json::{Json};

#[derive(Deserialize, Serialize)]
struct Message {
  bpm: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Block {
  index: i32,
  timestamp: String,
  bpm: i32,
  hash: String,
  prev_hash: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Blockchain {
  chain: Vec<Block>
}

type BlockchainMutex = Mutex<Blockchain>;

impl Block {
  fn new(previous: Option<Block>, bpm: i32) -> Block {
    let now_time = SystemTime::now();
    let datetime: DateTime<Utc> = now_time.into();

    match previous {
      Some(previous_block) => {
        return Block {
          index: previous_block.index + 1,
          timestamp: datetime.format("(%d/%m/%Y %T)").to_string(),
          bpm: bpm,
          hash: previous_block.calculate_hash(),
          prev_hash: Some(previous_block.hash),
        };
      },
      None => {
        return Block {
          index: 0,
          timestamp: datetime.format("(%d/%m/%Y %T)").to_string(),
          bpm: bpm,
          hash: "".to_string(),
          prev_hash: None,
        }
      }
    }

  }
  fn calculate_hash(&self) -> String {
    let record: String;
    match self.prev_hash {
      Some(ref previous_hash) => record = format!("{}{}{}{}", self.index, self.timestamp, self.bpm, previous_hash),
      None => record = format!("{}{}{}", self.index, self.timestamp, self.bpm),
    }
    println!("{}", record);
    let mut sha = Sha256::new();
    sha.input_str(&record);
    return sha.result_str();
  }

  fn is_valid_block(&self, old_block: Block) -> bool {
    if old_block.index+1 != self.index {
      return false;
    }

    let prev_hash: String;
    match self.prev_hash {
      Some(ref previous_hash) => prev_hash = previous_hash.to_string(),
      None => prev_hash = String::from(""),
    }
    if old_block.hash != prev_hash {
      return false;
    }

    if &self.calculate_hash() != &self.hash {
      return false;
    }

    return true;
  }
}

fn replace_chain(new_blocks: Vec<Block>, mut blocks: Vec<Block>) {
  if new_blocks.len() > blocks.len() {
    blocks = new_blocks;
  }
}

#[get("/", format = "text/html")]
fn handle_get_blockchain(chain: State<BlockchainMutex>) -> Option<Json<Vec<Block>>> {
  let bc = chain.lock().unwrap();
  let blockchain = bc.chain.to_vec();
  let my_json = Json(blockchain);
  return Some(my_json);
}

#[post("/", format = "application/json", data="<message>")]
fn handle_write_block(message: Json<Message>, chain: State<BlockchainMutex>) -> Option<Json<Vec<Block>>> {
  let mut bc = chain.lock().unwrap();
  let new_block = Block::new(None, message.bpm);
  //if new_block.is_valid_block(old_block: Block)
  bc.chain.push(new_block);
  let my_json = Json(bc.chain.to_vec());
  return Some(my_json);
}

// #[catch(404)]
// fn not_found() -> JsonValue {
//     json!({
//         "status": "error",
//         "reason": "Resource was not found."
//     })
// }

fn main() {

  rocket::ignite()
    .manage(Mutex::new(Blockchain { chain: Vec::new() }))
    //.register(catchers![])
    .mount("/", routes![handle_get_blockchain, handle_write_block])
    .launch();
}
