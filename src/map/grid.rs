use crate::error::{GameError, GameResult};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Size of each map cell in display characters (width and height).
pub const CELL_SIZE: usize = 10;

/// Character position in battle formation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    Front,
    Mid,
    Back,
}

impl Position {
    pub fn as_str(&self) -> &'static str {
        match self {
            Position::Front => "FRONT",
            Position::Mid => "MID  ",
            Position::Back => "BACK ",
        }
    }
}

/// A character in a team/squad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub cd_remaining: u32,
    pub cd_max: u32,
    pub energy: u32,
    pub ult_ready: bool,
    pub star_level: u32,
    pub position: Position,
    pub is_dead: bool,
}

impl Character {
    pub fn new(name: &str, position: Position, star_level: u32) -> Self {
        Character {
            name: name.to_string(),
            hp: 100,
            max_hp: 100,
            cd_remaining: 0,
            cd_max: 3,
            energy: 50,
            ult_ready: false,
            star_level,
            position,
            is_dead: false,
        }
    }
}

/// Terrain type for each cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Plain,
    Water,
    Mountain,
    Forest,
}

impl Terrain {
    /// Returns the display character for this terrain type.
    pub fn glyph(&self) -> char {
        match self {
            Terrain::Plain => '.',
            Terrain::Water => '~',
            Terrain::Mountain => '^',
            Terrain::Forest => '&',
        }
    }
}

/// A squad entry on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadEntry {
    pub name: String,
    pub abbreviation: String,
    pub x: u16,
    pub y: u16,
    pub squad_type: String, // "player" or "enemy"
    pub health: u32,
    pub max_health: u32,
    pub count: u32,
    pub speed_bar_pos: f64,
    pub is_acting: bool,
    pub has_lord: bool,
    pub members: Vec<Character>,
}

impl SquadEntry {
    pub fn alive(&self) -> bool {
        self.health > 0 && self.count > 0
    }

    pub fn leader_name(&self) -> &str {
        self.members.first().map(|c| c.name.as_str()).unwrap_or(&self.name)
    }

    pub fn with_characters(name: &str, abbr: &str, x: u16, y: u16, squad_type: &str, has_lord: bool) -> Self {
        let mut squad = SquadEntry {
            name: name.to_string(),
            abbreviation: abbr.to_string(),
            x,
            y,
            squad_type: squad_type.to_string(),
            health: 100,
            max_health: 100,
            count: 6,
            speed_bar_pos: 0.0,
            is_acting: false,
            has_lord,
            members: Vec::new(),
        };

        if squad_type == "player" && has_lord {
            // Lord team
            squad.members = vec![
                Character::new("Alexander", Position::Front, 2),
                Character::new("Beatrice", Position::Front, 1),
                Character::new("Christopher", Position::Mid, 3),
                Character::new("Diana", Position::Mid, 1),
                Character::new("Edward", Position::Back, 2),
                Character::new("Fiona", Position::Back, 1),
            ];
            // Give some variation to HP/CD/energy for demo
            squad.members[1].hp = 30;
            squad.members[1].max_hp = 40;
            squad.members[1].cd_remaining = 0;
            squad.members[1].cd_max = 2;
            squad.members[1].energy = 50;
            squad.members[3].hp = 20;
            squad.members[3].max_hp = 35;
            squad.members[3].cd_remaining = 3;
            squad.members[3].energy = 10;
            squad.members[4].cd_remaining = 0;
            squad.members[4].cd_max = 1;
            squad.members[4].energy = 70;
            squad.members[2].energy = 100;
            squad.members[2].ult_ready = true;
            squad.members[5].energy = 90;
            squad.members[5].ult_ready = true;
        } else {
            // Enemy team
            squad.members = vec![
                Character::new("GoblinBrute", Position::Front, 1),
                Character::new("GoblinScout", Position::Mid, 1),
                Character::new("GoblinShaman", Position::Back, 1),
            ];
            squad.count = 3;
            squad.health = 60;
            squad.max_health = 60;
        }

        squad
    }
}

/// The map grid: terrain + squads + camera.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapGrid {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<Vec<Terrain>>,
    pub squads: Vec<SquadEntry>,
    pub camera_x: u16,
    pub camera_y: u16,
    pub skill_points: u32,
    pub turn_number: u32,
    pub selected_squad: Option<usize>,
    pub combat_log: Vec<String>,
}

impl MapGrid {
    pub fn new(width: u16, height: u16, seed: Option<u64>) -> Self {
        let mut rng = match seed {
            Some(s) => {
                use rand::SeedableRng;
                rand::rngs::StdRng::seed_from_u64(s.wrapping_mul(31).wrapping_add(7))
            }
            None => {
                use rand::SeedableRng;
                rand::rngs::StdRng::from_entropy()
            }
        };

        let tiles: Vec<Vec<Terrain>> = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| {
                        let roll: f64 = rng.gen::<f64>();
                        if roll < 0.55 {
                            Terrain::Plain
                        } else if roll < 0.75 {
                            Terrain::Forest
                        } else if roll < 0.90 {
                            Terrain::Water
                        } else {
                            Terrain::Mountain
                        }
                    })
                    .collect()
            })
            .collect();

        let mut squads = vec![
            SquadEntry::with_characters("Mali", "MAL", width / 2, height / 2, "player", true),
            SquadEntry::with_characters("GoblinPack", "GOB", width / 3, height / 3, "enemy", false),
            SquadEntry::with_characters("OrcWarband", "ORC", width * 2 / 3, height * 2 / 3, "enemy", false),
            SquadEntry::with_characters("WolfRiders", "WLF", width * 3 / 4, height / 4, "enemy", false),
        ];

        // Set speed bar positions for visual demo
        squads[0].speed_bar_pos = 0.85; // Heroes almost ready
        squads[0].is_acting = true;
        squads[1].speed_bar_pos = 0.65;
        squads[2].speed_bar_pos = 0.45;
        squads[3].speed_bar_pos = 0.25;

        MapGrid {
            width,
            height,
            tiles,
            squads,
            camera_x: width / 2,
            camera_y: height / 2,
            skill_points: 5,
            turn_number: 12,
            selected_squad: Some(0),
            combat_log: vec![
                "⚔️ Battle started! Turn 12".to_string(),
                "Alexander used Horizontal Slash on GoblinBrute for 24 damage!".to_string(),
                "GoblinBrute attacks! Christopher takes 8 damage.".to_string(),
                "Christopher energy full! Ultimate ready.".to_string(),
                "Beatrice is at 75% HP.".to_string(),
                "Enemy GoblinShaman is casting a heal...".to_string(),
            ],
        }
    }

    pub fn teleport_squad(&mut self, name: &str, new_x: u16, new_y: u16) -> GameResult<()> {
        let x = new_x.clamp(0, self.width.saturating_sub(1));
        let y = new_y.clamp(0, self.height.saturating_sub(1));
        let squad = self
            .squads
            .iter_mut()
            .find(|s| s.name == name && s.alive())
            .ok_or_else(|| GameError::SquadNotFound(name.to_string()))?;
        squad.x = x;
        squad.y = y;
        Ok(())
    }

    pub fn set_camera(&mut self, x: u16, y: u16) {
        self.camera_x = x.clamp(0, self.width.saturating_sub(1));
        self.camera_y = y.clamp(0, self.height.saturating_sub(1));
    }

    pub fn camera_follow_squad(&mut self, name: &str) -> GameResult<()> {
        let squad = self
            .squads
            .iter()
            .find(|s| s.name == name && s.alive())
            .ok_or_else(|| GameError::SquadNotFound(name.to_string()))?;
        self.camera_x = squad.x;
        self.camera_y = squad.y;
        Ok(())
    }

    pub fn squad_roster(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!(
            "{:<20} {:<6} {:<6} {:<8} {:<6} {:<6}",
            "Name", "Type", "X", "Y", "HP", "Count"
        ));
        lines.push(format!(
            "{:-<20} {:-<6} {:-<6} {:-<8} {:-<6} {:-<6}",
            "", "", "", "", "", ""
        ));
        for squad in &self.squads {
            let status = if squad.alive() { "" } else { " [DEAD]" };
            lines.push(format!(
                "{:<20} {:<6} {:<6} {:<8} {:<3}/{:.<3} {:<6}{}",
                squad.name,
                squad.squad_type,
                squad.x,
                squad.y,
                squad.health,
                squad.max_health,
                squad.count,
                status,
            ));
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map_dimensions() {
        let map = MapGrid::new(100, 100, Some(42));
        assert_eq!(map.width, 100);
        assert_eq!(map.height, 100);
        assert_eq!(map.tiles.len(), 100);
        assert_eq!(map.tiles[0].len(), 100);
    }

    #[test]
    fn test_teleport_squad() {
        let mut map = MapGrid::new(100, 100, Some(42));
        map.teleport_squad("Mali", 10, 20).unwrap();
        let mali = map.squads.iter().find(|s| s.name == "Mali").unwrap();
        assert_eq!(mali.x, 10);
        assert_eq!(mali.y, 20);
    }

    #[test]
    fn test_teleport_clamp() {
        let mut map = MapGrid::new(100, 100, Some(42));
        map.teleport_squad("Mali", 200, 300).unwrap();
        let mali = map.squads.iter().find(|s| s.name == "Mali").unwrap();
        assert_eq!(mali.x, 99);
        assert_eq!(mali.y, 99);
    }

    #[test]
    fn test_camera_set() {
        let mut map = MapGrid::new(100, 100, Some(42));
        map.set_camera(30, 40);
        assert_eq!(map.camera_x, 30);
        assert_eq!(map.camera_y, 40);
    }

    #[test]
    fn test_camera_clamp() {
        let mut map = MapGrid::new(100, 100, Some(42));
        map.set_camera(500, 500);
        assert_eq!(map.camera_x, 99);
        assert_eq!(map.camera_y, 99);
    }

    #[test]
    fn test_camera_follow_squad() {
        let mut map = MapGrid::new(100, 100, Some(42));
        map.teleport_squad("Mali", 40, 50).unwrap();
        map.camera_follow_squad("Mali").unwrap();
        assert_eq!(map.camera_x, 40);
        assert_eq!(map.camera_y, 50);
    }

    #[test]
    fn test_squad_roster() {
        let map = MapGrid::new(100, 100, Some(42));
        let roster = map.squad_roster();
        assert!(roster.contains("Mali"));
        assert!(roster.contains("Goblin"));
        assert!(roster.contains("Orc"));
    }
}
