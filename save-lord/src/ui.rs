use serde::{Serialize, Deserialize};
use crate::combat::{TacticalMap, VIEWPORT_SIZE, GRID_SIZE, SPEED_BAR_MAX};
use crate::types::Side;
use crate::team::Team;
use crate::map::FloorMap;

/// Render the complete grail-layout combat screen.
pub fn render_combat(tactical: &TacticalMap, selected_team_id: Option<u32>) -> String {
    let mut out = String::new();
    out.push_str("┌──────────────────────────────────────────────────────────────┐\n");

    // Left panel: speed bar
    let speed_lines = render_speed_bar(tactical);
    // Center: battle map viewport
    let map_lines = render_battle_map(tactical);
    // Right panel: minimap + resources + portrait
    let right_lines = render_right_panel(tactical, selected_team_id);

    // Combine panels side by side
    let max_lines = speed_lines.len().max(map_lines.len()).max(right_lines.len());
    for i in 0..max_lines {
        out.push_str("│ ");
        // Speed bar (width 14)
        if i < speed_lines.len() {
            out.push_str(&pad_str(&speed_lines[i], 14));
        } else {
            out.push_str(&pad_str("", 14));
        }
        out.push_str(" │ ");
        // Battle map (width 40)
        if i < map_lines.len() {
            out.push_str(&pad_str(&map_lines[i], 44));
        } else {
            out.push_str(&pad_str("", 44));
        }
        out.push_str(" │ ");
        // Right panel (width 14)
        if i < right_lines.len() {
            out.push_str(&pad_str(&right_lines[i], 14));
        } else {
            out.push_str(&pad_str("", 14));
        }
        out.push_str(" │\n");
    }

    out.push_str("└──────────────────────────────────────────────────────────────┘\n");

    // Team details (selected team)
    if let Some(tid) = selected_team_id {
        if let Some(team) = tactical.get_team(tid) {
            out.push_str(&render_team_details(team));
        }
    }

    // Log
    out.push_str("\n--- Combat Log ---\n");
    for msg in tactical.log.iter().rev().take(8) {
        out.push_str(&format!("  {}\n", msg));
    }

    // Action prompt
    if let Some(acting) = tactical.current_acting_team {
        if let Some(team) = tactical.get_team(acting) {
            if team.side == Side::Player {
                out.push_str(&format!("\n>> Team '{}' ({}) is acting. Enter command then 'continue':\n",
                    team.name, team.abbreviation));
            } else {
                out.push_str(&format!("\n>> Enemy team '{}' is acting. Type 'continue' to resolve.\n",
                    team.name));
            }
        }
    }

    out
}

fn render_speed_bar(tactical: &TacticalMap) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("SPEED BAR".to_string());
    lines.push("────────────".to_string());

    // Sort by bar position descending
    let mut sorted: Vec<(u32, String, f64, Side)> = tactical.speed_bar.iter()
        .filter_map(|e| {
            let team = tactical.get_team(e.team_id)?;
            if !team.is_alive() { return None; }
            Some((e.team_id, format!("{} ({})", team.abbreviation, team.leader_name().chars().next().unwrap_or('?')), e.bar_position, team.side))
        })
        .collect();
    sorted.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    for (tid, label, bar, side) in &sorted {
        let is_current = Some(*tid) == tactical.current_acting_team;
        let marker = if is_current { ">>" } else { "  " };
        let side_marker = match side {
            Side::Player => "P",
            Side::Enemy => "E",
            Side::Neutral => "N",
        };
        let bar_filled = (bar / SPEED_BAR_MAX * 10.0) as usize;
        let bar_str: String = std::iter::repeat('█').take(bar_filled)
            .chain(std::iter::repeat('░').take(10 - bar_filled))
            .collect();
        lines.push(format!("{}{} [{}] {}", marker, side_marker, bar_str, label));
    }
    lines.push("".to_string());
    lines.push(format!("SP: {}", tactical.skill_points));
    lines.push(format!("Turn: {}", tactical.turn_number));
    lines
}

fn render_battle_map(tactical: &TacticalMap) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("BATTLE MAP".to_string());
    lines.push("────────────────────────────────────────".to_string());

    // Determine viewport center: follow first player team
    let center = tactical.teams.iter()
        .find(|t| t.side == Side::Player)
        .map(|t| t.position)
        .unwrap_or(crate::types::GridPos::new(GRID_SIZE/2, GRID_SIZE/2));

    let half = VIEWPORT_SIZE / 2;
    let start_x = (center.x - half).clamp(0, GRID_SIZE - VIEWPORT_SIZE);
    let start_y = (center.y - half).clamp(0, GRID_SIZE - VIEWPORT_SIZE);

    for dy in 0..VIEWPORT_SIZE {
        let mut line = String::new();
        for dx in 0..VIEWPORT_SIZE {
            let x = start_x + dx;
            let y = start_y + dy;
            // Check for team at this position
            let team_here = tactical.teams.iter().find(|t| t.position.x == x && t.position.y == y);
            if let Some(team) = team_here {
                let ch = match team.side {
                    Side::Player => team.abbreviation.chars().next().unwrap_or('P'),
                    Side::Enemy => team.abbreviation.chars().next().unwrap_or('e').to_ascii_lowercase(),
                    Side::Neutral => 'N',
                };
                // Mark lord team with *
                if team.has_lord {
                    line.push('★');
                } else {
                    line.push(ch);
                }
            } else {
                let terrain = tactical.grid[x as usize][y as usize];
                line.push(terrain.as_char());
            }
        }
        lines.push(line);
    }

    // Legend
    lines.push("Legend: P=Player e=Enemy ★=Lord .=Plain F=Forest ~=Water #=Obstacle".to_string());
    lines
}

fn render_right_panel(tactical: &TacticalMap, selected_id: Option<u32>) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("MINIMAP".to_string());
    lines.push("────────────".to_string());

    // Minimap: 14×14 showing whole grid
    let scale = GRID_SIZE / 14;
    for my in 0..14 {
        let mut line = String::new();
        for mx in 0..14 {
            let cx = mx * scale + scale/2;
            let cy = my * scale + scale/2;
            let team_here = tactical.teams.iter().find(|t| {
                t.position.x / scale == mx && t.position.y / scale == my
            });
            if let Some(team) = team_here {
                match team.side {
                    Side::Player => line.push(if team.has_lord { '★' } else { 'P' }),
                    Side::Enemy => line.push('E'),
                    Side::Neutral => line.push('N'),
                }
            } else {
                let terrain = tactical.grid[cx as usize][cy as usize];
                line.push(match terrain {
                    crate::types::Terrain::Plain => '·',
                    crate::types::Terrain::Forest => 'f',
                    crate::types::Terrain::Water => '~',
                    crate::types::Terrain::Obstacle => '#',
                });
            }
        }
        lines.push(line);
    }

    lines.push("────────────".to_string());
    lines.push(format!("SP: {}", tactical.skill_points));

    // Show selected team portrait
    lines.push("────────────".to_string());
    if let Some(tid) = selected_id {
        if let Some(team) = tactical.get_team(tid) {
            lines.push(format!("LEADER: {}", team.leader_name()));
            for pline in render_ascii_portrait(team) {
                lines.push(pline);
            }
        }
    } else if let Some(acting) = tactical.current_acting_team {
        if let Some(team) = tactical.get_team(acting) {
            lines.push(format!("ACTING: {}", team.abbreviation));
        }
    }

    lines
}

fn render_ascii_portrait(team: &Team) -> Vec<String> {
    let mut lines = Vec::new();
    let is_lord = team.has_lord;
    if is_lord {
        lines.push("   ╔═══╗   ".to_string());
        lines.push("   │★_★│   ".to_string());
        lines.push("   │≋≋≋│   ".to_string());
        lines.push("   ╚═══╝   ".to_string());
        lines.push("  LORD  ".to_string());
    } else {
        lines.push("   .---.   ".to_string());
        lines.push("  / o o \\ ".to_string());
        lines.push("  |  ^  | ".to_string());
        lines.push("   \\___/  ".to_string());
        lines.push(format!(" {:^8}", team.abbreviation));
    }
    lines
}

pub fn render_team_details(team: &Team) -> String {
    let mut out = String::new();
    out.push_str(&format!("\n[Team: {}] id={} side={:?} pos=({},{})\n",
        team.name, team.id, team.side, team.position.x, team.position.y));
    out.push_str(&format!(" Pos  | {:<18} | HP      | CD  | Energy | Ult   | Star\n", "Name"));
    out.push_str("------+--------------------+---------+-----+--------+-------+------\n");
    let mut members: Vec<&crate::characters::Character> = team.members.iter().collect();
    members.sort_by_key(|c| (c.position as u8, c.name().to_string()));
    for c in members {
        let dead_marker = if c.is_dead { " [DEAD]" } else { "" };
        let ult = if c.ult_used { "Used" } else { "Ready" };
        let stars: String = (0..c.star_level).map(|_| '★').collect();
        out.push_str(&format!(" {:<5}| {:<18} | {:>3}/{:<4} | {}/{} | {:>3}/100 | {:<5} | {}{}\n",
            c.position.as_str(),
            truncate(c.name(), 18),
            c.hp, c.max_hp,
            c.cd_remaining,
            c.template.cd_skill.as_ref().map(|s| s.cooldown_max).unwrap_or(0),
            c.energy,
            ult,
            stars,
            dead_marker));
    }
    out
}

/// Render the overworld map view.
pub fn render_overworld(floor: &FloorMap) -> String {
    let mut out = String::new();
    out.push_str(&floor.render_tree());
    out.push('\n');
    out.push_str("Available next nodes:\n");
    for node in floor.available_nodes() {
        out.push_str(&format!("  node {} — {} [use: node {}]\n",
            node.id.index, node.node_type.as_str(), node.id.index));
    }
    out
}

/// Render the campfire screen.
pub fn render_campfire() -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════╗\n");
    out.push_str("║        🔥 CAMPFIRE 🔥        ║\n");
    out.push_str("╠══════════════════════════════╣\n");
    out.push_str("║  A: Revive all dead chars    ║\n");
    out.push_str("║  B: Upgrade a team (+1 cap)  ║\n");
    out.push_str("║  C: Rest (heal 30% all)      ║\n");
    out.push_str("║  D: Leave campfire           ║\n");
    out.push_str("╚══════════════════════════════╝\n");
    out
}

/// Render the shop screen.
pub fn render_shop(gold: i32, items: &[ShopItem]) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════╗\n");
    out.push_str(&format!("║        🏪 SHOP ({}g)       ║\n", gold));
    out.push_str("╠══════════════════════════════╣\n");
    for (i, item) in items.iter().enumerate() {
        out.push_str(&format!("║  {}: {} ({}g)            ║\n", i+1, item.name, item.price));
    }
    out.push_str("║  L: Leave shop               ║\n");
    out.push_str("╚══════════════════════════════╝\n");
    out.push_str("Buy with: buy <number>\n");
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    pub name: String,
    pub price: i32,
    pub kind: ShopItemKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShopItemKind {
    Equipment(usize),
    Relic(usize),
    Potion,
    Food,
    Character(u32),
}

/// Render sub-battle status.
pub fn render_sub_battle(sb: &crate::sub_battle::SubBattle) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║           ⚔️  SUB-BATTLE ⚔️              ║\n");
    out.push_str("╠══════════════════════════════════════════╣\n");

    out.push_str("║  ATTACKER SIDE:                          ║\n");
    for c in &sb.attacker_chars {
        let dead = if c.is_dead { " 💀" } else { "" };
        out.push_str(&format!("║   {:<18} HP {}/{} [{}]{}  ║\n",
            truncate(c.name(), 18), c.hp, c.max_hp, c.position.as_str(), dead));
    }
    out.push_str("╠══════════════════════════════════════════╣\n");
    out.push_str("║  DEFENDER SIDE:                          ║\n");
    for c in &sb.defender_chars {
        let dead = if c.is_dead { " 💀" } else { "" };
        out.push_str(&format!("║   {:<18} HP {}/{} [{}]{}  ║\n",
            truncate(c.name(), 18), c.hp, c.max_hp, c.position.as_str(), dead));
    }
    out.push_str("╠══════════════════════════════════════════╣\n");
    out.push_str(&format!("║  Skill Points: {}                        ║\n", sb.skill_points));
    out.push_str(&format!("║  Turn: {}                                 ║\n", sb.turn));

    out.push_str("╠══════════════════════════════════════════╣\n");
    out.push_str("║  Recent events:                          ║\n");
    for msg in sb.log.iter().rev().take(6) {
        out.push_str(&format!("║   {}  ║\n", truncate(msg, 38)));
    }

    if let Some(result) = &sb.result {
        out.push_str("╠══════════════════════════════════════════╣\n");
        out.push_str(&format!("║  RESULT: {:<32}║\n", format!("{:?}", result)));
    }

    out.push_str("╚══════════════════════════════════════════╝\n");
    out
}

/// Render status screen.
pub fn render_status(game: &crate::game::GameState) -> String {
    let mut out = String::new();
    out.push_str("=== SAVE LORD STATUS ===\n");
    out.push_str(&format!("Phase: {:?}\n", game.phase));
    out.push_str(&format!("Seed: {}\n", game.seed));
    out.push_str(&format!("Score (damage dealt): {}\n", game.score_damage_dealt));
    out.push_str(&format!("Lord Level: {} ({})\n", game.lord.level, crate::lord::Lord::level_name(game.lord.level)));
    out.push_str(&format!("Max Teams: {}\n", game.lord.max_teams()));
    out.push_str(&format!("Floor: {}/{}\n", game.overworld.current_floor + 1, game.overworld.total_floors));
    out.push_str(&format!("Gold: {}\n", game.overworld.gold));
    out.push_str(&format!("Unlocked Characters: {}\n", game.pool.owned_characters.len()));
    out.push_str(&format!("Owned Equipment: {}\n", game.pool.owned_equipment.len()));
    out.push_str(&format!("Owned Relics: {}\n", game.pool.owned_relics.len()));
    out.push_str(&format!("Potions: {}\n", game.pool.owned_potions.len()));
    out.push_str(&format!("Food: {}\n", game.pool.owned_food.len()));
    out.push_str(&format!("Global XP (meta): {}\n", game.global_xp));
    if !game.pool.owned_relics.is_empty() {
        out.push_str("\nRelics owned:\n");
        for r in &game.pool.owned_relics {
            out.push_str(&format!("  - {}: {}\n", r.name, r.description));
        }
    }
    out
}

/// Render inventory.
pub fn render_inventory(game: &crate::game::GameState) -> String {
    let mut out = String::new();
    out.push_str("=== INVENTORY ===\n");

    out.push_str("\n-- Characters (owned) --\n");
    for c in &game.pool.owned_characters {
        let dups = game.pool.duplicate_char_counts.get(&c.id).copied().unwrap_or(0);
        out.push_str(&format!("  #{} {} | ATK {} HP {} SPD {} | dups: {}\n",
            c.id, truncate(&c.name, 24), c.base_atk, c.base_hp, c.base_spd, dups));
    }

    out.push_str("\n-- Equipment --\n");
    for (i, eq) in game.pool.owned_equipment.iter().enumerate() {
        let abilities: Vec<String> = eq.abilities.iter().map(|a| a.description()).collect();
        out.push_str(&format!("  {}. {} [{}]\n", i+1, eq.name, abilities.join(", ")));
    }

    out.push_str("\n-- Relics --\n");
    for r in &game.pool.owned_relics {
        out.push_str(&format!("  - {}: {}\n", r.name, r.description));
    }

    out.push_str("\n-- Potions --\n");
    for p in &game.pool.owned_potions {
        out.push_str(&format!("  - {} x{}\n", p.name, p.uses));
    }

    out.push_str("\n-- Food --\n");
    for f in &game.pool.owned_food {
        out.push_str(&format!("  - {} ({} uses)\n", f.name, f.uses));
    }
    out
}

/// Render help.
pub fn render_help() -> String {
    let mut out = String::new();
    out.push_str("=== SAVE LORD — Command Reference ===\n\n");
    out.push_str("Basic:\n");
    out.push_str("  help                Show this help\n");
    out.push_str("  status              Global status overview\n");
    out.push_str("  select <team_id>    Select a team\n");
    out.push_str("  continue            Advance to next UI refresh\n");
    out.push_str("  quit                Quit the game\n\n");
    out.push_str("Combat:\n");
    out.push_str("  move <id> <dir> [n] Move team (N/S/E/W/NE/NW/SE/SW)\n");
    out.push_str("  attack <id>         Attack adjacent enemy (enter sub-battle)\n");
    out.push_str("  skill <id> <name>   Use SP skill\n");
    out.push_str("  cdskill <id> <name> Use cooldown skill (interrupt)\n");
    out.push_str("  energyskill <id> <name>  Use energy skill (interrupt)\n");
    out.push_str("  ultimate <id>       Use ultimate (once per battle, interrupt)\n");
    out.push_str("  normal <id>         Normal attack\n");
    out.push_str("  wait <id>           Skip unit's turn\n\n");
    out.push_str("Save/Load:\n");
    out.push_str("  save <name>         Save to named slot\n");
    out.push_str("  load <name>         Load from named slot\n");
    out.push_str("  listsaves           List all saves\n");
    out.push_str("  viewsave <name>     View save contents\n\n");
    out.push_str("Map:\n");
    out.push_str("  map                 Show overworld map\n");
    out.push_str("  node <id>           Travel to node\n");
    out.push_str("  teamlist            List teams\n");
    out.push_str("  inventory           Show inventory\n");
    out.push_str("  gacha               Show gacha pool\n");
    out.push_str("  merge <n1> <n2>     Merge duplicate characters\n\n");
    out.push_str("Sub-battle:\n");
    out.push_str("  sub_skill <c> <s>   Use active skill\n");
    out.push_str("  sub_passive <c> <s> Trigger passive\n");
    out.push_str("  sub_retreat         Retreat\n");
    out.push_str("  sub_status          Sub-battle status\n\n");
    out.push_str("Tip: Game is turn-based. After any action, type 'continue'.\n");
    out
}

fn pad_str(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count >= width {
        s.chars().take(width).collect()
    } else {
        let mut padded = s.to_string();
        padded.extend(std::iter::repeat(' ').take(width - count));
        padded
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}
