#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate crypto;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;


//use std::ops::Deref;
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

#[derive(Deserialize, Serialize)]
struct Response {
  status: i32,
  message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Block {
  index: i32,
  timestamp: String,
  bpm: i32,
  hash: String,
  prev_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Blockchain {
  chain: Vec<Block>
}

type BlockchainMutex = Mutex<Blockchain>;

fn hash(input: String) -> String {
  let mut sha = Sha256::new();
  sha.input_str(&input);
  return sha.result_str();
}

impl Block {
  fn new() -> Block {
    let now_time = SystemTime::now();
    let datetime: DateTime<Utc> = now_time.into();

    return Block {
      index: 0,
      timestamp: datetime.format("(%d/%m/%Y %T)").to_string(),
      bpm: 0,
      hash: String::from(""),
      prev_hash:String::from(""),
    };
  }

  fn new_block(previous: &Block, bpm: i32) -> Block {
    let now_time = SystemTime::now();
    let datetime: DateTime<Utc> = now_time.into();

    let mut block = Block {
      index: previous.index + 1,
      timestamp: datetime.format("(%d/%m/%Y %T)").to_string(),
      bpm: bpm,
      hash: String::from(""),
      prev_hash: previous.hash.to_owned(),
    };
    block.set_hash(block.calculate_hash());
    return block;

  }
  fn calculate_hash(&self) -> String {
    let record = format!("{}{}{}{}", self.index, self.timestamp, self.bpm, self.prev_hash);
    return hash(record);
  }

  fn is_valid_block(&self, old_block: &Block) -> bool {
    if old_block.index+1 != self.index {
      return false;
    }

    if old_block.hash != self.prev_hash {
      return false;
    }

    if self.calculate_hash() != self.hash {
      return false;
    }

    return true;
  }

  fn set_hash(&mut self, hash: String) {
    self.hash = hash;
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
  let last_block = (&bc.chain).last().unwrap().clone();
  let new_block = Block::new_block(&last_block, message.bpm);

  if new_block.is_valid_block(&last_block) {
    let mut new_chain = bc.chain.to_vec();
    new_chain.push(new_block);

    // Also chose the longer chain.
    if new_chain.len() > bc.chain.len() {
      bc.chain = new_chain;
    }

    let my_json = Json(bc.chain.to_vec());
    return Some(my_json);
  };

  return None;
}

#[catch(404)]
fn not_found(_req: &rocket::Request) -> Json<Response> {
    Json(Response {
        status: 404,
        message: String::from("Page not found."),
    })
}

fn main() {

  rocket::ignite()
    .manage(Mutex::new(Blockchain { chain: vec![Block::new()] }))
    .catch(catchers![not_found])
    .mount("/", routes![handle_get_blockchain, handle_write_block])
    .launch();
}
