use bevy::prelude::*;

use crate::common::Position;
use crate::hitbox::Hitbox;
use crate::move_system::MoveObjectType;
use std::option::Option;

// File i/o libraries:
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;

//JSON libraries:
use serde::{Deserialize, Serialize};
use serde_json::{to_string, Result};

pub type MapId = u32;
const SAVES_PATH: &str = "saves/map";

// Structure of objects not rendered yet.
#[derive(Clone, Serialize, Deserialize)]
pub struct ParsedEntity {
    pub move_type: MoveObjectType,
    pub position: Position,
    pub hitbox: Hitbox,
}

pub struct Parser {
    entities: Vec<ParsedEntity>,
}

// Returns unique string made from given ID (may be made more complicated if needed).
pub fn get_string_from_id(id: &MapId) -> String {
    to_string(id).expect("Couldn't convert ID to String")
}

// Returns full save path from given ID, place of saving can be changed with constant SAVES_PATH.

pub fn get_filename(id: &MapId) -> String {
    let mut filename_beginning: String = String::from(SAVES_PATH);
    let save_name: String = get_string_from_id(id);
    filename_beginning.push_str(&save_name);
    filename_beginning
}
// May be used to make JSON files from existing state of game, currently unused.

pub fn save_map(filename: &str, map: &[ParsedEntity]) {
    let mut output = File::create(filename);
    let json = serde_json::to_string(map);
    match output {
        Ok(mut val) => {
            write!(val, "{}", json.unwrap()).expect("Error, couldn't write to file");
        }
        Err(err) => {
            println!("Unable to open demanded file");
        }
    }
}

impl Parser {
    // Parses file written in JSON format, return None if path or content is invalid.
    pub fn new(id: MapId) -> Option<Parser> {
        let filename = get_filename(&id);

        let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
        let result: Result<Vec<ParsedEntity>> = serde_json::from_str(&contents);

        return match result {
            Ok(val) => {
                Some(Parser { entities: val })
            }
            Err(err) => {
                println!("Incorrect file content");
                None
            }
        };
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ParsedEntity> {
        self.entities.iter()
    }
}
