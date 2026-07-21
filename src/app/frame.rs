use crate::map::grid::{MapGrid, SquadEntry};
use crate::map::render::format_map_view;

// ─────────────────────────────────────────────────────────
// Layout constants
// ─────────────────────────────────────────────────────────
const SPEED_BAR_W: usize = 22;
const RIGHT_PANEL_W: usize = 18;

// ─────────────────────────────────────────────────────────
// Frame builder — Holy Grail Layout
// ─────────────────────────────────────────────────────────

/// Build the full combat frame: speed bar (left), map (center), minimap/portrait (right).
pub fn format_frame(
    map: &MapGrid,
    messages: &[String],
    term_cols: u16,
    _term_rows: u16,
) -> String {
    let cols = term_cols as usize;
    let mut out = String::new();

    // ── Top border ──
    out.push_str("╔");
    out.push_str(&"═".repeat(cols.saturating_sub(2)));
    out.push_str("╗\n");

    // ── Title line ──
    let title = format!(
        " SAVE LORD — COMBAT  |  Turn {}  |  SP: {}  |  Camera: ({},{}) ",
        map.turn_number, map.skill_points, map.camera_x, map.camera_y
    );
    out.push_str("║");
    out.push_str(&pad_or_trim(&title, cols.saturating_sub(2)));
    out.push_str("║\n");

    // ── Title separator ──
    out.push_str("╠");
    out.push_str(&"═".repeat(SPEED_BAR_W + 1));
    out.push_str("╦");
    let map_area_w = cols.saturating_sub(SPEED_BAR_W + RIGHT_PANEL_W + 4);
    out.push_str(&"═".repeat(map_area_w));
    out.push_str("╦");
    out.push_str(&"═".repeat(RIGHT_PANEL_W + 1));
    out.push_str("╣\n");

    // ── Build panels ──
    let speed_lines = render_speed_bar(map);
    let map_lines = render_map_viewport(map, map_area_w);
    let right_lines = render_right_panel(map);

    let panel_h = speed_lines.len().max(map_lines.len()).max(right_lines.len());

    // ── Column headers ──
    out.push_str("║ ");
    out.push_str(&pad_str("SPEED BAR", SPEED_BAR_W));
    out.push_str(" ║ ");
    out.push_str(&pad_str("BATTLE MAP", map_area_w.saturating_sub(1)));
    out.push_str("║ ");
    out.push_str(&pad_str("STATUS", RIGHT_PANEL_W));
    out.push_str(" ║\n");

    out.push_str("║ ");
    out.push_str(&"─".repeat(SPEED_BAR_W));
    out.push_str(" ║ ");
    out.push_str(&"─".repeat(map_area_w.saturating_sub(1)));
    out.push_str("║ ");
    out.push_str(&"─".repeat(RIGHT_PANEL_W));
    out.push_str(" ║\n");

    // ── Combine panels side by side ──
    for i in 0..panel_h {
        out.push_str("║ ");
        if i < speed_lines.len() {
            out.push_str(&pad_str(&speed_lines[i], SPEED_BAR_W));
        } else {
            out.push_str(&" ".repeat(SPEED_BAR_W));
        }
        out.push_str(" ║ ");
        if i < map_lines.len() {
            out.push_str(&pad_str(&map_lines[i], map_area_w.saturating_sub(1)));
        } else {
            out.push_str(&" ".repeat(map_area_w.saturating_sub(1)));
        }
        out.push_str("║ ");
        if i < right_lines.len() {
            out.push_str(&pad_str(&right_lines[i], RIGHT_PANEL_W));
        } else {
            out.push_str(&" ".repeat(RIGHT_PANEL_W));
        }
        out.push_str(" ║\n");
    }

    // ── Separator before team details ──
    out.push_str("╠");
    out.push_str(&"═".repeat(SPEED_BAR_W + 1));
    out.push_str("╩");
    out.push_str(&"═".repeat(map_area_w));
    out.push_str("╩");
    out.push_str(&"═".repeat(RIGHT_PANEL_W + 1));
    out.push_str("╣\n");

    // ── Team details section ──
    out.push_str("║");
    out.push_str(&pad_str(" TEAM ROSTER", cols.saturating_sub(2)));
    out.push_str("║\n");

    out.push_str("║ ");
    out.push_str(&format!("{:<5}", "Pos"));
    out.push_str(" │ ");
    out.push_str(&format!("{:<18}", "Name"));
    out.push_str(" │ ");
    out.push_str(&format!("{:<9}", "HP"));
    out.push_str(" │ ");
    out.push_str(&format!("{:<5}", "CD"));
    out.push_str(" │ ");
    out.push_str(&format!("{:<7}", "Energy"));
    out.push_str(" │ ");
    out.push_str(&format!("{:<5}", "Ult"));
    out.push_str(" │ ");
    out.push_str(&format!("{:<5}", "Star"));
    // Pad to full width
    let used = 5 + 3 + 18 + 3 + 9 + 3 + 5 + 3 + 7 + 3 + 5 + 3 + 5;
    if cols > used + 2 {
        out.push_str(&" ".repeat(cols - used - 2));
    }
    out.push_str("║\n");

    out.push_str("║ ");
    out.push_str(&"─".repeat(5));
    out.push_str("─┼─");
    out.push_str(&"─".repeat(18));
    out.push_str("─┼─");
    out.push_str(&"─".repeat(9));
    out.push_str("─┼─");
    out.push_str(&"─".repeat(5));
    out.push_str("─┼─");
    out.push_str(&"─".repeat(7));
    out.push_str("─┼─");
    out.push_str(&"─".repeat(5));
    out.push_str("─┼─");
    out.push_str(&"─".repeat(5));
    if cols > used + 2 {
        out.push_str(&"─".repeat(cols - used - 2));
    }
    out.push_str("║\n");

    // Team members
    if let Some(selected_idx) = map.selected_squad {
        if let Some(squad) = map.squads.get(selected_idx) {
            let mut members: Vec<_> = squad.members.iter().collect();
            members.sort_by_key(|c| (c.position as u8, c.name.clone()));

            for c in members {
                let dead = if c.is_dead { " 💀" } else { "" };
                let ult = if c.ult_ready { "Ready" } else { "Not " };
                let stars: String = (0..c.star_level).map(|_| '★').collect();
                let hp_str = format!("{:>3}/{:<4}{}", c.hp, c.max_hp, dead);
                let cd_str = format!("{}/{}", c.cd_remaining, c.cd_max);
                let energy_str = format!("{:>3}/100", c.energy);

                out.push_str("║ ");
                out.push_str(&pad_str(c.position.as_str(), 5));
                out.push_str(" │ ");
                out.push_str(&pad_str(&truncate(&c.name, 18), 18));
                out.push_str(" │ ");
                out.push_str(&pad_str(&hp_str, 9));
                out.push_str(" │ ");
                out.push_str(&pad_str(&cd_str, 5));
                out.push_str(" │ ");
                out.push_str(&pad_str(&energy_str, 7));
                out.push_str(" │ ");
                out.push_str(&pad_str(ult, 5));
                out.push_str(" │ ");
                out.push_str(&pad_str(&stars, 5));
                if cols > used + 2 {
                    out.push_str(&" ".repeat(cols - used - 2));
                }
                out.push_str("║\n");
            }
        }
    }

    // ── Combat Log ──
    out.push_str("╠");
    out.push_str(&"═".repeat(cols.saturating_sub(2)));
    out.push_str("╣\n");

    out.push_str("║");
    out.push_str(&pad_str(" COMBAT LOG", cols.saturating_sub(2)));
    out.push_str("║\n");

    for msg in map.combat_log.iter().rev().take(4) {
        out.push_str("║ ");
        out.push_str(&pad_str(&format!("> {}", truncate(msg, cols.saturating_sub(5))), cols.saturating_sub(3)));
        out.push_str("║\n");
    }

    // Messages
    for msg in messages.iter().rev().take(2) {
        out.push_str("║ ");
        out.push_str(&pad_str(&format!("> {}", truncate(msg, cols.saturating_sub(5))), cols.saturating_sub(3)));
        out.push_str("║\n");
    }

    // ── Action prompt ──
    if let Some(acting_idx) = map.squads.iter().position(|s| s.is_acting && s.squad_type == "player") {
        let squad = &map.squads[acting_idx];
        out.push_str("╠");
        out.push_str(&"═".repeat(cols.saturating_sub(2)));
        out.push_str("╣\n");
        let prompt = format!(
            " >> Team '{}' is acting. Enter command, then type 'continue' to advance.",
            squad.name
        );
        out.push_str("║");
        out.push_str(&pad_str(&prompt, cols.saturating_sub(2)));
        out.push_str("║\n");
    }

    // ── Bottom border ──
    out.push_str("╚");
    out.push_str(&"═".repeat(cols.saturating_sub(2)));
    out.push_str("╝\n");

    out
}

/// Add ANSI color codes.
pub fn colorize_output(output: &str) -> String {
    const GREEN: &str = "\x1b[32m";
    const YELLOW: &str = "\x1b[33m";
    const BOLD: &str = "\x1b[1m";
    const RESET: &str = "\x1b[0m";

    let mut result = output.to_string();

    // Highlight acting team in speed bar with >>
    result = result.replace(">> ", &format!("{BOLD}{YELLOW}>> {RESET}"));

    // Color player/enemy markers in roster
    result = result.replace(
        &format!("{GREEN}HER{RESET}"),
        &format!("{BOLD}{GREEN}HER{RESET}"),
    );

    // Color stars
    result = result.replace("★", &format!("{YELLOW}★{RESET}"));

    result
}

// ─────────────────────────────────────────────────────────
// Speed Bar Panel (Left)
// ─────────────────────────────────────────────────────────

fn render_speed_bar(map: &MapGrid) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::new());

    // Sort by speed bar position descending
    let mut sorted: Vec<(usize, &SquadEntry)> = map.squads.iter().enumerate().filter(|(_, s)| s.alive()).collect();
    sorted.sort_by(|a, b| b.1.speed_bar_pos.partial_cmp(&a.1.speed_bar_pos).unwrap_or(std::cmp::Ordering::Equal));

    for (_idx, squad) in &sorted {
        let is_current = squad.is_acting;
        let marker = if is_current { ">>" } else { "  " };
        let side_marker = if squad.squad_type == "player" { "P" } else { "E" };

        let bar_filled = (squad.speed_bar_pos * 10.0) as usize;
        let bar_filled = bar_filled.min(10);
        let bar_str: String = std::iter::repeat('█').take(bar_filled)
            .chain(std::iter::repeat('░').take(10 - bar_filled))
            .collect();

        let lord_marker = if squad.has_lord { " ★" } else { "" };
        let name = truncate(&squad.abbreviation, 3);
        lines.push(format!(
            "{}{} [{}] {}{}",
            marker, side_marker, bar_str, name, lord_marker
        ));

        // HP bar on second line
        let hp_ratio = squad.health as f64 / squad.max_health as f64;
        let hp_filled = (hp_ratio * 10.0) as usize;
        let hp_filled = hp_filled.min(10);
        let hp_bar: String = std::iter::repeat('♥').take(hp_filled)
            .chain(std::iter::repeat('·').take(10 - hp_filled))
            .collect();
        lines.push(format!("   HP [{}] {}/{}", hp_bar, squad.health, squad.max_health));
    }

    lines.push(String::new());
    lines.push(format!("Turn: {}", map.turn_number));
    lines.push(format!("SP: {}", map.skill_points));

    lines
}

// ─────────────────────────────────────────────────────────
// Map Viewport (Center) — 10×10 character cells per grid cell
// ─────────────────────────────────────────────────────────

fn render_map_viewport(map: &MapGrid, width: usize) -> Vec<String> {
    // Use the proper 10x10 cell block renderer from map/render.rs
    // Each grid cell = 10x10 characters (100 chars per cell) as specified
    // We request enough rows for the viewport
    let available_cols = width as u16;
    let available_rows = 50; // Generous size for display
    
    let mut lines: Vec<String> = format_map_view(map, available_cols, available_rows, true)
        .lines()
        .map(|l| l.to_string())
        .collect();
    
    // Add legend at the bottom
    lines.push(String::new());
    lines.push("Legend: +--+ = Squad block  Green=Player  Red=Enemy".to_string());
    lines.push("        . = Plain  & = Forest  ~ = Water  ^ = Mountain".to_string());
    
    lines
}

// ─────────────────────────────────────────────────────────
// Right Panel: Minimap + Resources + Portrait
// ─────────────────────────────────────────────────────────

fn render_right_panel(map: &MapGrid) -> Vec<String> {
    let mut lines = Vec::new();

    // ── Minimap ──
    lines.push("── MINIMAP ──".to_string());

    let mm_size = 14;
    let cell_w = (map.width as usize / mm_size).max(1);
    let cell_h = (map.height as usize / mm_size).max(1);

    for my in 0..mm_size {
        let mut line = String::new();
        for mx in 0..mm_size {
            let squad_here = map.squads.iter().find(|s| {
                s.alive() && (s.x as usize / cell_w == mx) && (s.y as usize / cell_h == my)
            });
            if let Some(squad) = squad_here {
                if squad.has_lord {
                    line.push('★');
                } else if squad.squad_type == "player" {
                    line.push('P');
                } else {
                    line.push('E');
                }
            } else {
                let map_x = (mx * cell_w + cell_w / 2).min(map.width as usize - 1);
                let map_y = (my * cell_h + cell_h / 2).min(map.height as usize - 1);
                line.push(match map.tiles[map_y][map_x] {
                    crate::map::grid::Terrain::Plain => '·',
                    crate::map::grid::Terrain::Forest => '♣',
                    crate::map::grid::Terrain::Water => '~',
                    crate::map::grid::Terrain::Mountain => '^',
                });
            }
        }
        lines.push(line);
    }

    lines.push(String::new());

    // ── Resources ──
    lines.push("── RESOURCES ──".to_string());
    lines.push(format!("SP: {}", map.skill_points));
    lines.push("Potions: 3".to_string());
    lines.push("Food: 5".to_string());
    lines.push("Gold: 120".to_string());
    lines.push(String::new());

    // ── Leader Portrait (ASCII art) ──
    lines.push("── LEADER ──".to_string());
    if let Some(idx) = map.selected_squad {
        if let Some(squad) = map.squads.get(idx) {
            lines.push(format!("{}", squad.leader_name()));
            lines.push(String::new());
            for pline in render_ascii_portrait(squad) {
                lines.push(pline);
            }
        }
    }

    lines
}

fn render_ascii_portrait(squad: &SquadEntry) -> Vec<String> {
    let mut lines = Vec::new();
    if squad.has_lord {
        lines.push("   ╔═════╗   ".to_string());
        lines.push("   │ ♥_♥ │   ".to_string());
        lines.push("   │ ≋≋≋ │   ".to_string());
        lines.push("   ╚═════╝   ".to_string());
        lines.push("   [ LORD ]  ".to_string());
    } else if squad.squad_type == "enemy" {
        lines.push("    .---.    ".to_string());
        lines.push("   / x x \\  ".to_string());
        lines.push("   |  >  |  ".to_string());
        lines.push("    \\___/   ".to_string());
        lines.push(format!("   [{:^5}]", squad.abbreviation));
    } else {
        lines.push("    .---.    ".to_string());
        lines.push("   / o o \\  ".to_string());
        lines.push("   |  ^  |  ".to_string());
        lines.push("    \\___/   ".to_string());
        lines.push(format!("   [{:^5}]", squad.abbreviation));
    }
    lines
}

// ─────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────

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

fn pad_or_trim(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count >= width {
        s.chars().take(width).collect()
    } else {
        let mut r = s.to_string();
        r.push_str(&" ".repeat(width - count));
        r
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else if max <= 1 {
        s.chars().take(max).collect()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::grid::MapGrid;

    fn test_map() -> MapGrid {
        MapGrid::new(100, 100, Some(42))
    }

    #[test]
    fn test_format_frame_basic() {
        let map = test_map();
        let msgs = vec!["Welcome".into(), "Type help".into()];
        let frame = format_frame(&map, &msgs, 100, 50);
        assert!(frame.contains("SAVE LORD"));
        assert!(frame.contains("SPEED BAR"));
        assert!(frame.contains("BATTLE MAP"));
        assert!(frame.contains("MINIMAP"));
        assert!(frame.contains("TEAM ROSTER"));
        assert!(frame.contains("COMBAT LOG"));
    }

    #[test]
    fn test_render_speed_bar() {
        let map = test_map();
        let speed = render_speed_bar(&map);
        assert!(!speed.is_empty());
        assert!(speed.iter().any(|l| l.contains("█")));
    }

    #[test]
    fn test_render_map_viewport() {
        let map = test_map();
        let map_lines = render_map_viewport(&map, 50);
        assert!(!map_lines.is_empty());
    }

    #[test]
    fn test_render_right_panel() {
        let map = test_map();
        let right = render_right_panel(&map);
        assert!(!right.is_empty());
        assert!(right.iter().any(|l| l.contains("MINIMAP")));
        assert!(right.iter().any(|l| l.contains("LORD")));
    }

    #[test]
    fn test_pad_str() {
        assert_eq!(pad_str("hi", 4), "hi  ");
        assert_eq!(pad_str("hello world", 5), "hello");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 8), "hello");
        assert_eq!(truncate("hello world", 6), "hello…");
    }
}
