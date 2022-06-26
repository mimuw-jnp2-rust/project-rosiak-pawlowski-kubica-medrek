use crate::common::Position;
use crate::hitbox::Hitbox;
use crate::move_system::MoveObjectType;
use std::option::Option;

// File i/o libraries:
use std::fs;

//JSON libraries:
use serde::{Deserialize, Serialize};
use serde_json::{to_string, Result};

pub type MapId = u32;
const MAPS_PATH: &str = "maps/map";

// Structure of parsed, but not rendered objects.
#[derive(Clone, Serialize, Deserialize)]
pub struct ParsedEntity {
    pub move_type: MoveObjectType,
    pub position: Position,
    pub hitbox: Hitbox,
}

// Parses file written in JSON format, in case of invalid path or content returns None.
pub struct Parser {
    entities: Vec<ParsedEntity>,
}

impl Parser {
    pub fn new(id: MapId) -> Option<Parser> {
        let filename = get_filename(&id);
        let contents =
            fs::read_to_string(filename).expect("Something went wrong when reading the file");
        let result: Result<Vec<ParsedEntity>> = serde_json::from_str(&contents);

        return match result {
            Ok(val) => Some(Parser { entities: val }),
            Err(_) => {
                println!("Incorrect file content");
                None
            }
        };
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ParsedEntity> {
        self.entities.iter()
    }
}

// Returns unique string made from given ID.
pub fn get_string_from_id(id: &MapId) -> String {
    to_string(id).expect("Couldn't convert ID to String")
}

// Returns full path to a map with given ID. Saving location is set to MAPS_PATH.
pub fn get_filename(id: &MapId) -> String {
    let mut filename_beginning: String = String::from(MAPS_PATH);
    let map_name: String = get_string_from_id(id);
    filename_beginning.push_str(&map_name);
    filename_beginning
}
