use serde::{Serialize, Deserialize};
use crate::rng::SeededRng;
use crate::team::Team;
use crate::types::{GridPos, Direction, Terrain, Side};

/// Size of the tactical grid (40x40 for playable pacing).
pub const GRID_SIZE: i32 = 40;
/// Viewport size (visible area).
pub const VIEWPORT_SIZE: i32 = 21;
/// Speed bar length in UI.
pub const SPEED_BAR_MAX: f64 = 1000.0;

/// The tactical combat map with speed-based turn order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalMap {
    pub grid: Vec<Vec<Terrain>>,
    pub teams: Vec<Team>,
    pub speed_bar: Vec<SpeedEntry>,
    pub current_turn_idx: usize,
    pub skill_points: i32,
    pub turn_number: i32,
    pub next_team_id: u32,
    pub active_phase: CombatPhase,
    pub sub_battle_active: bool,
    pub log: Vec<String>,
    pub current_acting_team: Option<u32>,
    pub selected_team: Option<u32>,
    pub first_strike_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedEntry {
    pub team_id: u32,
    pub bar_position: f64,
    pub team_name: String,
    pub side: Side,
    pub avg_spd: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatPhase {
    SelectingAction,
    TeamMoving,
    TeamAttacking,
    SubBattleActive,
    BattleOver,
}

impl TacticalMap {
    pub fn new(rng: &mut SeededRng) -> Self {
        let mut grid = vec![vec![Terrain::Plain; GRID_SIZE as usize]; GRID_SIZE as usize];

        // Generate sparse terrain features (much less for open battlefield)
        for _ in 0..40 {
            let x = rng.gen_range(3, GRID_SIZE - 3);
            let y = rng.gen_range(3, GRID_SIZE - 3);
            let terrain = match rng.gen_range(0, 4) {
                0 => Terrain::Forest,
                1 => Terrain::Water,
                _ => Terrain::Obstacle,
            };
            // Small cluster
            let cluster = rng.gen_range(1, 3);
            for dx in 0..cluster {
                for dy in 0..cluster {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx < GRID_SIZE && ny < GRID_SIZE && rng.gen_bool(0.5) {
                        grid[nx as usize][ny as usize] = terrain;
                    }
                }
            }
        }

        // Clear starting zones (x < 12 for player, x > 28 for enemy)
        for x in 0..12 {
            for y in 0..GRID_SIZE {
                grid[x as usize][y as usize] = Terrain::Plain;
            }
        }
        for x in 28..GRID_SIZE {
            for y in 0..GRID_SIZE {
                grid[x as usize][y as usize] = Terrain::Plain;
            }
        }

        Self {
            grid,
            teams: Vec::new(),
            speed_bar: Vec::new(),
            current_turn_idx: 0,
            skill_points: 3,
            turn_number: 0,
            next_team_id: 1,
            active_phase: CombatPhase::SelectingAction,
            sub_battle_active: false,
            log: Vec::new(),
            current_acting_team: None,
            selected_team: None,
            first_strike_done: false,
        }
    }

    pub fn add_team(&mut self, mut team: Team) -> u32 {
        // Respect preset ID if it was explicitly set (player teams use 0..100)
        let id = if team.id < 100 {
            // Player teams with preset IDs: keep their ID
            self.next_team_id = self.next_team_id.max(team.id + 1);
            team.id
        } else {
            let nid = self.next_team_id;
            team.id = nid;
            self.next_team_id += 1;
            nid
        };
        let abbr: String = team.name.split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(3)
            .collect::<String>()
            .to_uppercase();
        team.abbreviation = if abbr.is_empty() { format!("T{}", id) } else { abbr };
        self.speed_bar.push(SpeedEntry {
            team_id: id,
            bar_position: { let mut r = rng_from_seed(id as u64); r.gen_range_f64(0.0, 200.0) },
            team_name: team.name.clone(),
            side: team.side,
            avg_spd: team.avg_spd(),
        });
        self.teams.push(team);
        id
    }

    pub fn get_team(&self, id: u32) -> Option<&Team> {
        self.teams.iter().find(|t| t.id == id)
    }

    pub fn get_team_mut(&mut self, id: u32) -> Option<&mut Team> {
        self.teams.iter_mut().find(|t| t.id == id)
    }

    /// Move a team in a direction. Returns new position if successful.
    pub fn move_team(&mut self, team_id: u32, dir: Direction, distance: i32) -> Result<GridPos, String> {
        let team = self.get_team(team_id).ok_or("Team not found")?.clone();
        let (dx, dy) = dir.to_delta();
        let old_pos = team.position;
        let max_steps = distance.min(team.move_range).max(1);
        let unit_type = team.lord()
            .map(|l| l.template.unit_type)
            .or_else(|| team.alive_members().first().map(|c| c.template.unit_type))
            .unwrap_or(crate::types::UnitType::Infantry);

        let mut new_pos = old_pos;
        let mut steps_moved = 0;

        for step in 1..=max_steps {
            let try_x = (old_pos.x + dx * step).clamp(0, GRID_SIZE - 1);
            let try_y = (old_pos.y + dy * step).clamp(0, GRID_SIZE - 1);
            let try_pos = GridPos::new(try_x, try_y);

            // Check terrain
            let terrain = self.grid[try_pos.x as usize][try_pos.y as usize];
            if terrain.move_cost(unit_type).is_none() { break; }
            // Check no other team occupies
            if self.teams.iter().any(|t| t.id != team_id && t.position == try_pos) { break; }

            new_pos = try_pos;
            steps_moved += 1;
        }

        if steps_moved == 0 {
            return Err("Cannot move in that direction".into());
        }

        if let Some(t) = self.get_team_mut(team_id) {
            t.position = new_pos;
        }
        self.log.push(format!("Team {} moved {} step(s) to ({},{})",
            team.abbreviation, steps_moved, new_pos.x, new_pos.y));
        Ok(new_pos)
    }

    /// Find adjacent enemy team to attack.
    pub fn find_adjacent_enemy(&self, team_id: u32) -> Option<u32> {
        let team = self.get_team(team_id)?;
        let my_pos = team.position;
        let my_side = team.side;
        for other in &self.teams {
            if other.id == team_id || other.side == my_side || !other.is_alive() { continue; }
            if my_pos.chebyshev_distance(other.position) <= 1 {
                return Some(other.id);
            }
        }
        None
    }

    /// Advance speed bars and determine next acting team.
    pub fn advance_speed_bar(&mut self) -> Option<u32> {
        if self.teams.is_empty() { return None; }

        // Precompute alive teams' avg spd to avoid borrow conflict
        let spd_map: Vec<(u32, i32, bool)> = self.teams.iter()
            .map(|t| (t.id, t.avg_spd().max(1), t.is_alive()))
            .collect();

        // Advance all bars based on team avg spd
        for entry in &mut self.speed_bar {
            if let Some(&(_, spd, alive)) = spd_map.iter().find(|(id, _, _)| *id == entry.team_id) {
                if !alive { continue; }
                entry.avg_spd = spd;
                entry.bar_position += spd as f64 / SPEED_BAR_MAX * 100.0;
                if entry.bar_position > SPEED_BAR_MAX {
                    entry.bar_position = SPEED_BAR_MAX;
                }
            }
        }

        // Find highest bar among alive teams
        let mut max_pos = -1.0f64;
        let mut max_id = None;
        for entry in &self.speed_bar {
            if let Some(&(_, _, alive)) = spd_map.iter().find(|(id, _, _)| *id == entry.team_id) {
                if !alive { continue; }
                if entry.bar_position > max_pos {
                    max_pos = entry.bar_position;
                    max_id = Some(entry.team_id);
                }
            }
        }

        let acting_id = max_id?;

        // Reset the acting team's bar
        for entry in &mut self.speed_bar {
            if entry.team_id == acting_id {
                entry.bar_position = 0.0;
            }
        }

        self.turn_number += 1;
        self.current_acting_team = Some(acting_id);
        Some(acting_id)
    }

    /// Enemy AI: move toward nearest player, prefer lord.
    pub fn enemy_ai_step(&mut self, _rng: &mut SeededRng) -> Option<String> {
        let player_positions: Vec<(u32, GridPos, bool)> = self.teams.iter()
            .filter(|t| t.side == Side::Player && t.is_alive())
            .map(|t| (t.id, t.position, t.has_lord))
            .collect();

        if player_positions.is_empty() { return None; }

        let current_id = self.current_acting_team?;
        let team = self.get_team(current_id)?;
        let epos = team.position;
        let target = player_positions.iter()
            .min_by(|a, b| {
                let da = (epos.distance(a.1) as i32, !a.2);
                let db = (epos.distance(b.1) as i32, !b.2);
                da.cmp(&db)
            }).copied();

        self.ai_move_toward(current_id, target)
    }

    /// Player AI: auto-move player teams toward nearest enemy when no command given.
    pub fn player_ai_step(&mut self, _rng: &mut SeededRng) -> Option<String> {
        let enemy_positions: Vec<(u32, GridPos, bool)> = self.teams.iter()
            .filter(|t| t.side == Side::Enemy && t.is_alive())
            .map(|t| (t.id, t.position, t.has_lord))
            .collect();

        if enemy_positions.is_empty() { return None; }

        let current_id = self.current_acting_team?;
        let team = self.get_team(current_id)?;
        if team.side != Side::Player { return None; }

        let ppos = team.position;
        // Prefer attacking enemy lord, then closest enemy
        let target = enemy_positions.iter()
            .min_by(|a, b| {
                let da = (ppos.distance(a.1) as i32, !a.2);
                let db = (ppos.distance(b.1) as i32, !b.2);
                da.cmp(&db)
            }).copied();

        self.ai_move_toward(current_id, target)
    }

    fn ai_move_toward(&mut self, current_id: u32, target: Option<(u32, GridPos, bool)>) -> Option<String> {
        let mut action_taken = String::new();

        if let Some((tid, tpos, is_lord)) = target {
            let epos = self.get_team(current_id).unwrap().position;
            let _ = is_lord;
            // Check if adjacent to target → attack
            if epos.chebyshev_distance(tpos) <= 1 {
                action_taken = format!("Team {} engages team {} in battle!", current_id, tid);
                return Some(action_taken);
            }

            let dx = (tpos.x - epos.x).signum();
            let dy = (tpos.y - epos.y).signum();
            let range = self.get_team(current_id).unwrap().move_range;

            // Try primary direction first, then fallbacks
            let dirs = match (dx, dy) {
                (1, -1) => [Direction::NE, Direction::E, Direction::N, Direction::SE, Direction::NW],
                (1, 1) => [Direction::SE, Direction::E, Direction::S, Direction::NE, Direction::SW],
                (-1, -1) => [Direction::NW, Direction::W, Direction::N, Direction::NE, Direction::SW],
                (-1, 1) => [Direction::SW, Direction::W, Direction::S, Direction::SE, Direction::NW],
                (1, 0) => [Direction::E, Direction::NE, Direction::SE, Direction::N, Direction::S],
                (-1, 0) => [Direction::W, Direction::NW, Direction::SW, Direction::N, Direction::S],
                (0, -1) => [Direction::N, Direction::NE, Direction::NW, Direction::E, Direction::W],
                (0, 1) => [Direction::S, Direction::SE, Direction::SW, Direction::E, Direction::W],
                _ => [Direction::E, Direction::NE, Direction::SE, Direction::N, Direction::S],
            };

            for &dir in &dirs {
                if let Ok(new_pos) = self.move_team(current_id, dir, range) {
                    action_taken = format!("Team {} moves toward ({},{})", current_id, new_pos.x, new_pos.y);
                    if new_pos.chebyshev_distance(tpos) <= 1 {
                        action_taken.push_str(&format!(" — engaging team {}!", tid));
                    }
                    break;
                }
            }
        }
        if action_taken.is_empty() { None } else { Some(action_taken) }
    }

    pub fn remove_dead_teams(&mut self) {
        let dead: Vec<u32> = self.teams.iter().filter(|t| !t.is_alive()).map(|t| t.id).collect();
        self.teams.retain(|t| t.is_alive());
        self.speed_bar.retain(|e| !dead.contains(&e.team_id));
    }

    pub fn check_victory(&self) -> Option<Side> {
        let player_alive = self.teams.iter().any(|t| t.side == Side::Player && t.is_alive());
        let enemy_alive = self.teams.iter().any(|t| t.side == Side::Enemy && t.is_alive());
        let player_lord_alive = self.teams.iter()
            .filter(|t| t.side == Side::Player)
            .any(|t| t.has_lord_alive());
        let enemy_lord_alive = self.teams.iter()
            .filter(|t| t.side == Side::Enemy)
            .any(|t| t.has_lord_alive());

        if !player_lord_alive || !player_alive {
            Some(Side::Enemy)
        } else if !enemy_lord_alive || !enemy_alive {
            Some(Side::Player)
        } else {
            None
        }
    }

    pub fn get_terrain(&self, pos: GridPos) -> Option<Terrain> {
        if pos.x < 0 || pos.x >= GRID_SIZE || pos.y < 0 || pos.y >= GRID_SIZE {
            None
        } else {
            Some(self.grid[pos.x as usize][pos.y as usize])
        }
    }

    pub fn add_log(&mut self, msg: &str) {
        self.log.push(msg.to_string());
        if self.log.len() > 50 {
            self.log.remove(0);
        }
    }
}

fn rng_from_seed(seed: u64) -> crate::rng::SeededRng {
    crate::rng::SeededRng::new(seed)
}
