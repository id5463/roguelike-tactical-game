use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use crate::types::GamePhase;
use crate::map::Overworld;
use crate::combat::TacticalMap;
use crate::lord::Lord;
use crate::gacha::GachaPool;

/// Save slot — a named snapshot of the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlot {
    pub name: String,
    pub seed: u64,
    pub turn: i32,
    pub score: i64,
    pub phase: GamePhase,
    pub overworld: Overworld,
    pub tactical: Option<TacticalMap>,
    pub lord: Lord,
    pub pool: GachaPool,
    pub global_xp: i64,
    pub global_unlocks: Vec<String>,
}

pub const SAVE_DIR: &str = "saves";
pub const META_FILE: &str = "meta.json";

pub fn save_dir() -> PathBuf {
    let mut p = PathBuf::from(SAVE_DIR);
    p.push("save-lord");
    p
}

fn ensure_save_dir() -> std::io::Result<()> {
    fs::create_dir_all(save_dir())
}

/// Save to a named slot.
pub fn save_game(slot_name: &str, slot: &SaveSlot) -> std::io::Result<()> {
    ensure_save_dir()?;
    let mut path = save_dir();
    path.push(format!("{}.json", slot_name));
    let json = serde_json::to_string_pretty(slot)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(&path, json)?;
    Ok(())
}

/// Load from a named slot.
pub fn load_game(slot_name: &str) -> std::io::Result<SaveSlot> {
    let mut path = save_dir();
    path.push(format!("{}.json", slot_name));
    let data = fs::read_to_string(&path)?;
    let slot: SaveSlot = serde_json::from_str(&data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(slot)
}

/// List all saved slots.
pub fn list_saves() -> Vec<String> {
    let dir = match save_dir().read_dir() {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let mut names = Vec::new();
    for entry in dir.flatten() {
        if let Some(ext) = entry.path().extension() {
            if ext == "json" {
                if let Some(stem) = entry.path().file_stem() {
                    names.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    names.sort();
    names
}

/// View a save file's contents as human-readable text.
pub fn view_save(slot_name: &str) -> std::io::Result<String> {
    let slot = load_game(slot_name)?;
    let mut out = String::new();
    out.push_str(&format!("Save Slot: {}\n", slot.name));
    out.push_str(&format!("Seed: {}\n", slot.seed));
    out.push_str(&format!("Turn: {}\n", slot.turn));
    out.push_str(&format!("Score: {}\n", slot.score));
    out.push_str(&format!("Lord Level: {} ({})\n", slot.lord.level, crate::lord::Lord::level_name(slot.lord.level)));
    out.push_str(&format!("Phase: {:?}\n", slot.phase));
    out.push_str(&format!("Max Teams: {}\n", slot.lord.max_teams()));
    out.push_str(&format!("Floor: {}/{}\n", slot.overworld.current_floor + 1, slot.overworld.total_floors));
    out.push_str(&format!("Gold: {}\n", slot.overworld.gold));
    out.push_str(&format!("Unlocked Characters: {}\n", slot.pool.owned_characters.len()));
    out.push_str(&format!("Owned Equipment: {}\n", slot.pool.owned_equipment.len()));
    out.push_str(&format!("Owned Relics: {}\n", slot.pool.owned_relics.len()));
    out.push_str(&format!("Global XP: {}\n", slot.global_xp));
    Ok(out)
}

/// Auto-save after every action.
pub fn auto_save(slot: &SaveSlot) {
    let _ = save_game("autosave", slot);
}
