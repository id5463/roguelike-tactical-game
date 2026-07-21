//! API for AI agents to read game state.
//!
//! The game exposes a text-based UI on stdout, but AI agents can also
//! read structured state via the `ApiState` struct, serialized as JSON.
//! To get the current state snapshot, the game calls `ApiState::from_game(game)`.

use serde::{Serialize, Deserialize};
use crate::game::GameState;
use crate::sub_battle::SubBattle;
use crate::types::{GamePhase, GridPos, Side};

/// Structured API snapshot of the current game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiState {
    pub phase: GamePhase,
    pub turn: i32,
    pub seed: u64,
    pub score: i64,
    pub floor: u32,
    pub total_floors: u32,
    pub lord_level: u32,
    pub gold: i32,
    pub skill_points: i32,
    pub acting_team_id: Option<u32>,
    pub selected_team_id: Option<u32>,
    pub teams: Vec<ApiTeam>,
    pub sub_battle: Option<ApiSubBattle>,
    pub available_commands: Vec<String>,
    pub last_log: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTeam {
    pub id: u32,
    pub name: String,
    pub abbreviation: String,
    pub side: Side,
    pub position: GridPos,
    pub is_alive: bool,
    pub has_lord: bool,
    pub avg_spd: i32,
    pub members: Vec<ApiCharacter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCharacter {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub spd: i32,
    pub star_level: u32,
    pub position: String,
    pub energy: i32,
    pub cd_remaining: i32,
    pub ult_ready: bool,
    pub is_dead: bool,
    pub is_lord: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSubBattle {
    pub turn: i32,
    pub skill_points: i32,
    pub result: Option<String>,
    pub attacker: Vec<ApiCharacter>,
    pub defender: Vec<ApiCharacter>,
    pub recent_log: Vec<String>,
}

impl ApiState {
    pub fn from_game(game: &GameState) -> Self {
        let phase = game.phase;
        let mut teams = Vec::new();
        let mut acting_team_id = None;
        let mut sp = 0;
        let mut last_log = Vec::new();

        if let Some(ref tac) = game.tactical {
            sp = tac.skill_points;
            acting_team_id = tac.current_acting_team;
            for t in &tac.teams {
                teams.push(ApiTeam::from_team(t));
            }
            for msg in tac.log.iter().rev().take(10) {
                last_log.push(msg.clone());
            }
        }

        let sub_battle = game.sub_battle.as_ref().map(ApiSubBattle::from_sb);

        // Determine available commands based on phase
        let available_commands = available_commands_for_phase(phase);

        Self {
            phase,
            turn: game.turn_number,
            seed: game.seed,
            score: game.score_damage_dealt,
            floor: game.overworld.current_floor + 1,
            total_floors: game.overworld.total_floors,
            lord_level: game.lord.level,
            gold: game.overworld.gold,
            skill_points: sp,
            acting_team_id,
            selected_team_id: game.selected_team,
            teams,
            sub_battle,
            available_commands,
            last_log,
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }
}

impl ApiTeam {
    fn from_team(t: &crate::team::Team) -> Self {
        Self {
            id: t.id,
            name: t.name.clone(),
            abbreviation: t.abbreviation.clone(),
            side: t.side,
            position: t.position,
            is_alive: t.is_alive(),
            has_lord: t.has_lord,
            avg_spd: t.avg_spd(),
            members: t.members.iter().map(ApiCharacter::from_char).collect(),
        }
    }
}

impl ApiCharacter {
    fn from_char(c: &crate::characters::Character) -> Self {
        Self {
            name: c.name().to_string(),
            hp: c.hp,
            max_hp: c.max_hp,
            atk: c.atk,
            def: c.def,
            spd: c.spd,
            star_level: c.star_level,
            position: c.position.as_str().to_string(),
            energy: c.energy,
            cd_remaining: c.cd_remaining,
            ult_ready: !c.ult_used,
            is_dead: c.is_dead,
            is_lord: c.is_lord,
        }
    }
}

impl ApiSubBattle {
    fn from_sb(sb: &SubBattle) -> Self {
        Self {
            turn: sb.turn,
            skill_points: sb.skill_points,
            result: sb.result.as_ref().map(|r| format!("{:?}", r)),
            attacker: sb.attacker_chars.iter().map(ApiCharacter::from_char).collect(),
            defender: sb.defender_chars.iter().map(ApiCharacter::from_char).collect(),
            recent_log: sb.log.iter().rev().take(10).cloned().collect(),
        }
    }
}

fn available_commands_for_phase(phase: GamePhase) -> Vec<String> {
    match phase {
        GamePhase::Map => vec!["map", "node <id>", "status", "inventory", "teamlist", "save", "quit"].into_iter().map(String::from).collect(),
        GamePhase::CombatTactical => vec!["move", "attack", "skill", "cdskill", "energyskill", "ultimate", "normal", "wait", "continue", "status", "select", "save"].into_iter().map(String::from).collect(),
        GamePhase::SubBattle => vec!["sub_skill", "sub_passive", "sub_retreat", "sub_status", "continue"].into_iter().map(String::from).collect(),
        GamePhase::Campfire => vec!["A/revive", "B/upgrade", "rest", "leave"].into_iter().map(String::from).collect(),
        GamePhase::Shop => vec!["buy <n>", "leave"].into_iter().map(String::from).collect(),
        GamePhase::Event => vec!["1", "2", "3"].into_iter().map(String::from).collect(),
        GamePhase::BossIntro => vec!["continue"].into_iter().map(String::from).collect(),
        GamePhase::Victory | GamePhase::Defeat => vec!["quit", "status"].into_iter().map(String::from).collect(),
    }
}
