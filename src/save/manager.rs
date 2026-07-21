use std::path::PathBuf;

use crate::error::{GameError, GameResult};
use crate::save::data::SaveData;

pub struct SaveManager {
    save_dir: PathBuf,
}

impl SaveManager {
    pub fn new(save_dir: PathBuf) -> GameResult<Self> {
        if !save_dir.exists() {
            std::fs::create_dir_all(&save_dir)
                .map_err(|e| GameError::SaveError(format!("cannot create save directory: {e}")))?;
        }
        Ok(SaveManager { save_dir })
    }

    pub fn save(&self, name: &str, data: &SaveData) -> GameResult<PathBuf> {
        let path = self.save_dir.join(format!("{name}.json"));
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| GameError::SaveError(format!("serialization failed: {e}")))?;
        std::fs::write(&path, &json)
            .map_err(|e| GameError::SaveError(format!("write failed: {e}")))?;
        Ok(path)
    }

    pub fn load(&self, name: &str) -> GameResult<SaveData> {
        let path = self.save_dir.join(format!("{name}.json"));
        let json = std::fs::read_to_string(&path)
            .map_err(|e| GameError::LoadError(format!("read failed: {e}")))?;
        let data: SaveData = serde_json::from_str(&json)
            .map_err(|e| GameError::LoadError(format!("parse failed: {e}")))?;
        Ok(data)
    }

    #[allow(dead_code)]
    pub fn list_saves(&self) -> GameResult<Vec<String>> {
        let mut saves = Vec::new();
        for entry in std::fs::read_dir(&self.save_dir)
            .map_err(|e| GameError::SaveError(format!("read directory failed: {e}")))? {
            let entry = entry.map_err(|e| GameError::SaveError(format!("directory entry error: {e}")))?;
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    saves.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
        saves.sort();
        Ok(saves)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::data::{MapData, SquadData};

    #[test]
    fn test_save_roundtrip() {
        let dir = std::env::temp_dir().join("roguelike_test_saves");
        let _ = std::fs::remove_dir_all(&dir);

        let mgr = SaveManager::new(dir.clone()).unwrap();

        let map_data = MapData {
            width: 100,
            height: 100,
            squads: vec![SquadData {
                name: "Mali".into(),
                x: 50,
                y: 50,
                is_enemy: false,
            }],
            tiles: vec![0u8; 10000],
        };
        let data = SaveData::new(42, 1, 100, map_data);

        mgr.save("test_save", &data).unwrap();
        let loaded = mgr.load("test_save").unwrap();

        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.turn, 1);
        assert_eq!(loaded.gold, 100);
        assert_eq!(loaded.map_data.squads.len(), 1);
        assert_eq!(loaded.map_data.squads[0].name, "Mali");
        assert_eq!(loaded.map_data.squads[0].x, 50);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
