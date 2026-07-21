use serde::{Serialize, Deserialize};
use crate::characters::{Character, CharacterTemplate};
use crate::skills::{lord_normal_attack, lord_move, lord_sp_skill, lord_cd_skill, lord_energy_skill, lord_ultimate};
use crate::types::{Position, UnitType};

/// Maximum lord level.
pub const MAX_LORD_LEVEL: u32 = 5;

/// Lord progression — the lord is a special permanent unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lord {
    pub level: u32,
    pub xp_progress: f64,  // 0.0..1.0 toward next level
    pub character: Character,
    pub base_teams: u32,
}

impl Lord {
    pub fn new() -> Self {
        let tmpl = lord_template();
        let mut chr = Character::from_template(tmpl);
        chr.is_lord = true;
        chr.position = Position::Frontline;
        Self {
            level: 1,
            xp_progress: 0.0,
            character: chr,
            base_teams: 4,
        }
    }

    /// Number of teams available at current lord level.
    pub fn max_teams(&self) -> u32 {
        match self.level {
            1 => 4,
            2 => 5,
            3 => 6,
            4 => 8,
            5 => 10,
            _ => 4,
        }
    }

    pub fn max_team_size_bonus(&self) -> u32 {
        // Higher lord levels allow bigger teams
        match self.level {
            1 => 0,
            2 => 1,
            3 => 1,
            4 => 2,
            5 => 2,
            _ => 0,
        }
    }

    /// Try to add XP; returns true if leveled up.
    pub fn add_xp(&mut self, xp: f64) -> bool {
        self.xp_progress += xp;
        if self.xp_progress >= 1.0 && self.level < MAX_LORD_LEVEL {
            self.xp_progress -= 1.0;
            self.level += 1;
            // Boost stats on level up
            self.character.atk += 20;
            self.character.max_hp += 30;
            self.character.spd += 10;
            self.character.heal(self.character.max_hp);
            true
        } else {
            false
        }
    }

    pub fn level_name(level: u32) -> &'static str {
        match level {
            1 => "Squire",
            2 => "Knight",
            3 => "Captain",
            4 => "General",
            5 => "Lord Commander",
            _ => "Unknown",
        }
    }
}

pub fn lord_template() -> CharacterTemplate {
    CharacterTemplate {
        id: u32::MAX,
        name: "Lord Commander".to_string(),
        unit_type: UnitType::Infantry,
        base_atk: 120,
        base_hp: 150,
        base_spd: 100,
        preferred_position: Position::Frontline,
        normal_attack: lord_normal_attack(),
        move_skill: lord_move(),
        sp_skill: Some(lord_sp_skill()),
        cd_skill: Some(lord_cd_skill()),
        energy_skill: Some(lord_energy_skill()),
        ultimate: Some(lord_ultimate()),
    }
}
