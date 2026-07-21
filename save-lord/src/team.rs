use serde::{Serialize, Deserialize};
use crate::characters::Character;
use crate::types::{GridPos, Side, Terrain};

/// Maximum members per team.
pub const MAX_TEAM_SIZE: usize = 6;

/// A team on the tactical map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: u32,
    pub name: String,
    pub side: Side,
    pub position: GridPos,
    pub members: Vec<Character>,
    pub move_range: i32,
    pub has_lord: bool,
    pub is_boss_team: bool,
    pub abbreviation: String,
    pub max_members: usize,
}

impl Team {
    pub fn new(id: u32, name: &str, side: Side, pos: GridPos) -> Self {
        let abbr: String = name.split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(3)
            .collect::<String>()
            .to_uppercase();
        Self {
            id,
            name: name.to_string(),
            side,
            position: pos,
            members: Vec::new(),
            move_range: 10,
            has_lord: false,
            is_boss_team: false,
            abbreviation: if abbr.is_empty() { format!("T{}", id) } else { abbr },
            max_members: 2, // Start with 2 members per team
        }
    }

    pub fn add_member(&mut self, ch: Character) -> Result<(), String> {
        if self.members.len() >= self.max_members {
            return Err(format!("Team '{}' is full (max {})", self.name, self.max_members));
        }
        self.members.push(ch);
        Ok(())
    }

    pub fn alive_members(&self) -> Vec<&Character> {
        self.members.iter().filter(|c| !c.is_dead).collect()
    }

    pub fn alive_members_mut(&mut self) -> Vec<&mut Character> {
        self.members.iter_mut().filter(|c| !c.is_dead).collect()
    }

    pub fn is_alive(&self) -> bool {
        self.members.iter().any(|c| !c.is_dead)
    }

    pub fn has_lord_alive(&self) -> bool {
        self.members.iter().any(|c| c.is_lord && !c.is_dead)
    }

    pub fn lord(&self) -> Option<&Character> {
        self.members.iter().find(|c| c.is_lord)
    }

    pub fn lord_mut(&mut self) -> Option<&mut Character> {
        self.members.iter_mut().find(|c| c.is_lord)
    }

    pub fn total_hp_percent(&self) -> f64 {
        let alive: Vec<&Character> = self.alive_members();
        if alive.is_empty() { return 0.0; }
        let total_max: i32 = alive.iter().map(|c| c.max_hp).sum();
        let total_cur: i32 = alive.iter().map(|c| c.hp).sum();
        if total_max == 0 { 0.0 } else { total_cur as f64 / total_max as f64 }
    }

    pub fn leader_name(&self) -> &str {
        if let Some(lord) = self.lord() { &lord.template.name }
        else if let Some(first) = self.alive_members().first() { &first.template.name }
        else { &self.name }
    }

    pub fn sort_by_position(&mut self) {
        self.members.sort_by_key(|c| {
            (c.position as u8, c.template.name.clone())
        });
    }

    pub fn avg_spd(&self) -> i32 {
        let alive = self.alive_members();
        if alive.is_empty() { return 0; }
        let total: f64 = alive.iter().map(|c| c.effective_spd()).sum::<f64>();
        (total / alive.len() as f64) as i32
    }

    pub fn terrain_move_cost(&self, terrain: Terrain) -> Option<i32> {
        // Team moves based on leader's unit type
        let leader = self.lord().or_else(|| self.alive_members().into_iter().next())?;
        terrain.move_cost(leader.template.unit_type)
    }
}
