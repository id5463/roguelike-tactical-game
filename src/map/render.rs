use crate::map::grid::{MapGrid, SquadEntry, CELL_SIZE};

// ─────────────────────────────────────────────────────────
// Squad block rendering
// ─────────────────────────────────────────────────────────

/// Render a single squad as a 10×10 block of characters.
fn render_squad_block(squad: &SquadEntry) -> Vec<String> {
    let interior_w: usize = 8;
    let block_w: usize = 10;
    let block_h: usize = 10;

    let name_chars: Vec<char> = squad.name.chars().collect();
    let name_len = name_chars.len();

    // Name only, centered
    let displayed: String = if name_len > interior_w {
        name_chars[..interior_w].iter().collect()
    } else {
        name_chars.iter().collect()
    };
    let displayed_len = displayed.len();
    let pad_left = (interior_w - displayed_len) / 2;
    let pad_right = interior_w - displayed_len - pad_left;

    let mut rows: Vec<String> = Vec::with_capacity(block_h);

    // Row 0: top border  "+--------+"
    let mut top = String::with_capacity(block_w);
    top.push('+');
    for _ in 0..interior_w {
        top.push('-');
    }
    top.push('+');
    rows.push(top);

    // Row 1: blank
    rows.push(format!("|{:8}|", ""));

    // Row 2: name line  "|  Mali  |"
    let mut name_line = String::with_capacity(block_w);
    name_line.push('|');
    for _ in 0..pad_left {
        name_line.push(' ');
    }
    name_line.push_str(&displayed);
    for _ in 0..pad_right {
        name_line.push(' ');
    }
    name_line.push('|');
    rows.push(name_line);

    // Rows 3..8: blank (6 rows)
    for _ in 0..6 {
        rows.push(format!("|{:8}|", ""));
    }

    // Row 9: bottom border
    let mut bottom = String::with_capacity(block_w);
    bottom.push('+');
    for _ in 0..interior_w {
        bottom.push('-');
    }
    bottom.push('+');
    rows.push(bottom);

    rows
}

/// Overlay a squad block onto the terrain grid at display coordinates.
fn overlay_block_solid(
    grid: &mut [Vec<char>],
    block: &[String],
    block_disp_x: i32,
    block_disp_y: i32,
    grid_w: usize,
    grid_h: usize,
) {
    for by in 0..CELL_SIZE {
        let row = block_disp_y + by as i32;
        if row < 0 || row >= grid_h as i32 {
            continue;
        }
        let line = &block[by];
        for bx in 0..CELL_SIZE {
            let col = block_disp_x + bx as i32;
            if col < 0 || col >= grid_w as i32 {
                continue;
            }
            grid[row as usize][col as usize] = line.as_bytes()[bx] as char;
        }
    }
}

// ─────────────────────────────────────────────────────────
// Map view rendering
// ─────────────────────────────────────────────────────────

/// Format the map view filling the given character area.
///
/// Each map cell is rendered as a `CELL_SIZE × CELL_SIZE` block of its
/// terrain glyph. Squad blocks (also `CELL_SIZE × CELL_SIZE`) are
/// overlaid at their cell position. The viewport is centred on the
/// camera position.
///
/// `colorize` = if true, inject ANSI color codes around squad names
///              (green for player, red for enemy).
pub fn format_map_view(
    map: &MapGrid,
    avail_cols: u16,
    avail_rows: u16,
    colorize: bool,
) -> String {
    let label_w: usize = 5;
    let label_h: usize = 3;

    let cell_area_w =
        ((avail_cols as usize).saturating_sub(label_w)) / CELL_SIZE * CELL_SIZE;
    let cell_area_h =
        ((avail_rows as usize).saturating_sub(label_h)) / CELL_SIZE * CELL_SIZE;

    let n_cells_x = cell_area_w / CELL_SIZE;
    let n_cells_y = cell_area_h / CELL_SIZE;

    if n_cells_x == 0 || n_cells_y == 0 {
        return "Map too small to render.".to_string();
    }

    let cam_disp_x =
        map.camera_x as i32 * CELL_SIZE as i32 + (CELL_SIZE as i32 / 2);
    let cam_disp_y =
        map.camera_y as i32 * CELL_SIZE as i32 + (CELL_SIZE as i32 / 2);

    let start_disp_x = cam_disp_x - (cell_area_w as i32 / 2);
    let start_disp_y = cam_disp_y - (cell_area_h as i32 / 2);

    let mut output = String::new();

    // --- Column axis labels ---
    // Hundreds row
    output.push_str(&" ".repeat(label_w));
    for cx in 0..n_cells_x {
        let map_cell = start_disp_x / CELL_SIZE as i32 + cx as i32;
        if map_cell >= 0 && map_cell < map.width as i32 && map_cell % 10 == 0 {
            let h = (map_cell / 100) as u32;
            if h > 0 {
                output.push(char::from_digit(h, 10).unwrap());
            } else {
                output.push(' ');
            }
        } else {
            output.push(' ');
        }
        for _ in 1..CELL_SIZE {
            output.push(' ');
        }
    }
    output.push('\n');

    // Tens row
    output.push_str(&" ".repeat(label_w));
    for cx in 0..n_cells_x {
        let map_cell = start_disp_x / CELL_SIZE as i32 + cx as i32;
        if map_cell >= 0 {
            let t = ((map_cell / 10) % 10) as u32;
            output.push(char::from_digit(t, 10).unwrap());
        } else {
            output.push(' ');
        }
        for _ in 1..CELL_SIZE {
            output.push(' ');
        }
    }
    output.push('\n');

    // Ones row
    output.push_str(&" ".repeat(label_w));
    for cx in 0..n_cells_x {
        let map_cell = start_disp_x / CELL_SIZE as i32 + cx as i32;
        if map_cell >= 0 {
            let o = (map_cell % 10) as u32;
            output.push(char::from_digit(o, 10).unwrap());
        } else {
            output.push(' ');
        }
        for _ in 1..CELL_SIZE {
            output.push(' ');
        }
    }
    output.push('\n');

    // --- Build the character grid ---
    let grid_w = cell_area_w;
    let grid_h = cell_area_h;
    let mut grid: Vec<Vec<char>> = vec![vec![' '; grid_w]; grid_h];

    // Fill terrain cells
    for cy in 0..n_cells_y {
        for cx in 0..n_cells_x {
            let map_cx = start_disp_x / CELL_SIZE as i32 + cx as i32;
            let map_cy = start_disp_y / CELL_SIZE as i32 + cy as i32;

            if map_cx < 0
                || map_cx >= map.width as i32
                || map_cy < 0
                || map_cy >= map.height as i32
            {
                continue;
            }

            let glyph = map.tiles[map_cy as usize][map_cx as usize].glyph();
            let ox = cx * CELL_SIZE;
            let oy = cy * CELL_SIZE;

            for dy in 0..CELL_SIZE {
                for dx in 0..CELL_SIZE {
                    grid[oy + dy][ox + dx] = glyph;
                }
            }
        }
    }

    // Overlay squad blocks and track name-row positions for coloring
    let mut sorted_squads: Vec<&SquadEntry> =
        map.squads.iter().filter(|s| s.alive()).collect();
    sorted_squads.sort_by_key(|s| {
        if s.squad_type == "player" {
            1
        } else {
            0
        }
    });

    struct NamePos {
        row: usize,
        col: usize,
        len: usize,
        is_player: bool,
    }
    let mut name_positions: Vec<NamePos> = Vec::new();

    for squad in &sorted_squads {
        let block = render_squad_block(squad);
        let sx = squad.x as i32 * CELL_SIZE as i32;
        let sy = squad.y as i32 * CELL_SIZE as i32;
        let disp_x = sx - start_disp_x;
        let disp_y = sy - start_disp_y;

        overlay_block_solid(&mut grid, &block, disp_x, disp_y, grid_w, grid_h);

        let name_row = (disp_y + 2) as usize;
        let name_col = (disp_x + 1) as usize;
        let name_len = CELL_SIZE - 2;
        if name_row < grid_h && name_col < grid_w {
            name_positions.push(NamePos {
                row: name_row,
                col: name_col,
                len: name_len,
                is_player: squad.squad_type == "player",
            });
        }
    }

    // Output rows with y-axis labels and optional color
    const ANSI_GREEN: &str = "\x1b[32m";
    const ANSI_RED: &str = "\x1b[31m";
    const ANSI_RESET: &str = "\x1b[0m";

    for row_idx in 0..grid_h {
        let map_y = start_disp_y + row_idx as i32;
        let map_cell_y = map_y / CELL_SIZE as i32;

        if row_idx % CELL_SIZE == 0 {
            output.push_str(&format!("{:>4} ", map_cell_y));
        } else {
            output.push_str("     ");
        }

        let line: String = grid[row_idx].iter().collect();

        if colorize {
            let mut colored = String::new();
            let mut last_end = 0;
            for np in &name_positions {
                if np.row != row_idx {
                    continue;
                }
                if np.col > last_end {
                    colored.push_str(&line[last_end..np.col]);
                }
                let interior = &line[np.col..np.col + np.len];
                if np.is_player {
                    colored.push_str(ANSI_GREEN);
                    colored.push_str(interior);
                    colored.push_str(ANSI_RESET);
                } else {
                    colored.push_str(ANSI_RED);
                    colored.push_str(interior);
                    colored.push_str(ANSI_RESET);
                }
                last_end = np.col + np.len;
            }
            if last_end < line.len() {
                colored.push_str(&line[last_end..]);
            }
            output.push_str(&colored);
        } else {
            output.push_str(&line);
        }
        output.push('\n');
    }

    output
}

// ─────────────────────────────────────────────────────────
// Minimap rendering
// ─────────────────────────────────────────────────────────

/// Render a minimap of the entire map at reduced resolution.
///
/// Returns `height` strings, each `width` characters wide.
/// Each character represents a block of map cells. Terrain is sampled
/// from the center of each block. Squad positions are marked with
/// their initial letter (uppercase = player, lowercase = enemy).
/// If `colorize` is true, squad markers are wrapped in ANSI color codes.
pub fn render_minimap(
    map: &MapGrid,
    width: usize,
    height: usize,
    colorize: bool,
) -> Vec<String> {
    if width == 0 || height == 0 || map.width == 0 || map.height == 0 {
        return vec![" ".repeat(width); height];
    }

    let cell_w = (map.width as usize / width).max(1);
    let cell_h = (map.height as usize / height).max(1);

    struct SquadMarker {
        mmx: usize,
        mmy: usize,
        ch: char,
        is_player: bool,
    }
    let markers: Vec<SquadMarker> = map
        .squads
        .iter()
        .filter(|s| s.alive())
        .map(|s| {
            let first = s.name.chars().next().unwrap_or('?');
            let ch = if s.squad_type == "player" {
                first.to_ascii_uppercase()
            } else {
                first.to_ascii_lowercase()
            };
            SquadMarker {
                mmx: (s.x as usize).saturating_sub(0) / cell_w,
                mmy: (s.y as usize).saturating_sub(0) / cell_h,
                ch,
                is_player: s.squad_type == "player",
            }
        })
        .collect();

    const ANSI_GREEN: &str = "\x1b[32m";
    const ANSI_RED: &str = "\x1b[31m";
    const ANSI_RESET: &str = "\x1b[0m";

    let mut lines: Vec<String> = Vec::with_capacity(height);
    for my in 0..height {
        let mut line = String::with_capacity(width);
        for mx in 0..width {
            let marker = markers.iter().find(|m| m.mmx == mx && m.mmy == my);

            if let Some(m) = marker {
                if colorize {
                    if m.is_player {
                        line.push_str(ANSI_GREEN);
                        line.push(m.ch);
                        line.push_str(ANSI_RESET);
                    } else {
                        line.push_str(ANSI_RED);
                        line.push(m.ch);
                        line.push_str(ANSI_RESET);
                    }
                } else {
                    line.push(m.ch);
                }
            } else {
                let map_x = (mx * cell_w + cell_w / 2).min(map.width as usize - 1);
                let map_y = (my * cell_h + cell_h / 2).min(map.height as usize - 1);
                line.push(map.tiles[map_y][map_x].glyph());
            }
        }
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::grid::MapGrid;

    #[test]
    fn test_squad_block_size() {
        let squad = crate::map::grid::SquadEntry::with_characters("Mali", "MAL", 50, 50, "player", false);
        let block = render_squad_block(&squad);
        assert_eq!(block.len(), 10);
        for row in &block {
            assert_eq!(row.len(), 10);
        }
        assert_eq!(block[0], "+--------+");
        assert_eq!(block[9], "+--------+");
        assert!(block[2].contains("Mali"));
    }

    #[test]
    fn test_format_map_view_contains_squad_blocks() {
        let map = MapGrid::new(100, 100, Some(42));
        let view = format_map_view(&map, 85, 43, false);
        assert!(
            view.contains('+'),
            "Terrain view should contain squad block borders"
        );
        assert!(
            view.contains('-'),
            "Terrain view should contain block horizontal lines"
        );
    }

    #[test]
    fn test_format_map_view_10x10_terrain() {
        let map = MapGrid::new(100, 100, Some(42));
        let view = format_map_view(&map, 85, 43, false);
        assert!(
            view.contains(".........."),
            "Terrain should show blocks of 10 identical chars"
        );
    }
}
