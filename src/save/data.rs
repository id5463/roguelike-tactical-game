use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadData {
    pub name: String,
    pub x: u16,
    pub y: u16,
    pub is_enemy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub timestamp: String,
    pub seed: u64,
    pub turn: u32,
    pub gold: u64,
    pub map_data: MapData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    pub width: u16,
    pub height: u16,
    pub squads: Vec<SquadData>,
    /// terrain encoded row by row: 0=Plain, 1=Water, 2=Mountain, 3=Forest
    pub tiles: Vec<u8>,
}

impl SaveData {
    pub fn new(seed: u64, turn: u32, gold: u64, map_data: MapData) -> Self {
        SaveData {
            version: 1,
            timestamp: chrono::Utc::now().to_rfc3339(),
            seed,
            turn,
            gold,
            map_data,
        }
    }
}
