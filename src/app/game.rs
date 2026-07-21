use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::app::frame::{colorize_output, format_frame};
use crate::app::terminal::detect_terminal_size;
use crate::command::parser::{Command, CommandParser};
use crate::error::{GameError, GameResult};
use crate::map::grid::{MapGrid, Terrain};
use crate::save::{MapData, SaveData, SaveManager, SquadData};

pub struct Game {
    pub map: MapGrid,
    pub messages: Vec<String>,
    pub save_dir: PathBuf,
}

impl Game {
    pub fn new(width: u16, height: u16, seed: Option<u64>, save_dir: PathBuf) -> Self {
        let map = MapGrid::new(width, height, seed);
        let mut game = Game {
            map,
            save_dir,
            messages: Vec::new(),
        };
        game.add_message("Welcome to Save/Load the Lord — a tactical roguelike.");
        game.add_message("Type 'help' for available commands.");
        game.add_message(&format!(
            "Camera at ({}, {}). Use 'camera <x> <y>' to move, or 'camera <name>' to follow.",
            game.map.camera_x, game.map.camera_y
        ));
        game
    }

    pub fn add_message(&mut self, msg: &str) {
        self.messages.push(msg.to_string());
        if self.messages.len() > 20 {
            self.messages.remove(0);
        }
    }

    // ── Commands ──

    pub fn execute_command(&mut self, cmd: Command) -> GameResult<bool> {
        match cmd {
            Command::Teleport { name, x, y } => {
                self.map.teleport_squad(&name, x, y)?;
                self.add_message(&format!("Teleported '{}' to ({}, {}).", name, x, y));
                Ok(true)
            }
            Command::Camera { x, y } => {
                self.map.set_camera(x, y);
                self.add_message(&format!(
                    "Camera moved to ({}, {}).",
                    self.map.camera_x, self.map.camera_y
                ));
                Ok(true)
            }
            Command::CameraSquad(name) => {
                self.map.camera_follow_squad(&name)?;
                self.add_message(&format!(
                    "Following '{}' at ({}, {}).",
                    name,
                    self.map.camera_x,
                    self.map.camera_y
                ));
                Ok(true)
            }
            Command::Help => {
                self.show_help();
                Ok(true)
            }
            Command::Save { name } => {
                let save_name = name.unwrap_or_else(|| "quick".to_string());
                self.save_game(&save_name)?;
                self.add_message(&format!("Saved as '{}'.", save_name));
                Ok(true)
            }
            Command::Load { name } => {
                let save_name = name.unwrap_or_else(|| "quick".to_string());
                self.load_game(&save_name)?;
                self.add_message(&format!("Loaded from '{}'.", save_name));
                Ok(true)
            }
            Command::Position => {
                self.show_positions();
                Ok(true)
            }
            Command::Quit => {
                self.add_message("Goodbye, commander.");
                Ok(false)
            }
        }
    }

    fn show_help(&self) {
        println!(r#"
--- Save/Load the Lord Commands ---

Movement:
  teleport <name> <x> <y>   Move a squad

Camera:
  camera <x> <y>            Move camera to absolute coordinates
  camera <name>             Centre camera on a squad

Information:
  help                      Show this help message
  position                  Show squad positions

Save/Load:
  save [name]               Save the game (default: 'quick')
  load [name]               Load a saved game (default: 'quick')

Other:
  quit                      Exit
"#);
    }

    fn show_positions(&mut self) {
        self.add_message("--- Squad Positions ---");
        println!("{}", self.map.squad_roster());
        println!();
        println!("Camera: ({}, {})", self.map.camera_x, self.map.camera_y);
    }

    // ── Save / Load ──

    fn save_game(&self, name: &str) -> GameResult<()> {
        let manager = SaveManager::new(self.save_dir.clone())?;
        manager.save(name, &self.map_to_save_data())?;
        Ok(())
    }

    fn load_game(&mut self, name: &str) -> GameResult<()> {
        let manager = SaveManager::new(self.save_dir.clone())?;
        let data = manager.load(name)?;
        self.apply_save_data(&data);
        Ok(())
    }

    fn map_to_save_data(&self) -> SaveData {
        SaveData::new(
            0,
            0,
            0,
            MapData {
                width: self.map.width,
                height: self.map.height,
                squads: self
                    .map
                    .squads
                    .iter()
                    .map(|s| SquadData {
                        name: s.name.clone(),
                        x: s.x,
                        y: s.y,
                        is_enemy: s.squad_type != "player",
                    })
                    .collect(),
                tiles: self
                    .map
                    .tiles
                    .iter()
                    .flat_map(|row| {
                        row.iter()
                            .map(|t| match t {
                                Terrain::Plain => 0,
                                Terrain::Water => 1,
                                Terrain::Mountain => 2,
                                _ => 3,
                            })
                            .collect::<Vec<u8>>()
                    })
                    .collect(),
            },
        )
    }

    fn apply_save_data(&mut self, data: &SaveData) {
        let tiles: Vec<Vec<Terrain>> = data
            .map_data
            .tiles
            .chunks(data.map_data.width as usize)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|&v| match v {
                        0 => Terrain::Plain,
                        1 => Terrain::Water,
                        2 => Terrain::Mountain,
                        _ => Terrain::Forest,
                    })
                    .collect()
            })
            .collect();
        self.map = MapGrid {
            width: data.map_data.width,
            height: data.map_data.height,
            tiles,
            squads: data
                .map_data
                .squads
                .iter()
                .map(|s| {
                    let squad_type = if s.is_enemy { "enemy" } else { "player" };
                    let abbr = if s.is_enemy { "ENC" } else { "HER" };
                    let has_lord = !s.is_enemy;
                    crate::map::grid::SquadEntry::with_characters(
                        &s.name, abbr, s.x, s.y, squad_type, has_lord
                    )
                })
                .collect(),
            camera_x: 50,
            camera_y: 50,
            skill_points: 3,
            turn_number: data.turn,
            selected_squad: Some(0),
            combat_log: vec!["Game loaded.".to_string()],
        };
    }

    // ── Main loop ──

    pub fn run(&mut self) -> GameResult<()> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();

        loop {
            let (cols, rows) = detect_terminal_size();
            let plain = format_frame(&self.map, &self.messages, cols, rows);
            let colored = colorize_output(&plain);

            print!("{}", colored);
            io::stdout().flush().map_err(|e| GameError::IoError(e))?;

            print!("> ");
            io::stdout().flush().map_err(|e| GameError::IoError(e))?;

            let mut input = String::new();
            reader
                .read_line(&mut input)
                .map_err(|e| GameError::IoError(e))?;

            match CommandParser::parse(&input) {
                Ok(cmd) => {
                    if !self.execute_command(cmd)? {
                        break;
                    }
                }
                Err(e) => {
                    self.add_message(&format!("Error: {}", e));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_game() -> Game {
        Game::new(100, 100, Some(42), PathBuf::from("test_saves"))
    }

    #[test]
    fn test_new_game_has_squads() {
        assert!(!test_game().map.squads.is_empty());
    }

    #[test]
    fn test_execute_teleport() {
        let mut g = test_game();
        g.execute_command(Command::Teleport {
            name: "Mali".into(),
            x: 30,
            y: 40,
        })
        .unwrap();
        assert_eq!(
            g.map.squads.iter().find(|s| s.name == "Mali").unwrap().x,
            30
        );
    }

    #[test]
    fn test_execute_camera_coords() {
        let mut g = test_game();
        g.execute_command(Command::Camera { x: 25, y: 35 })
            .unwrap();
        assert_eq!(g.map.camera_x, 25);
    }

    #[test]
    fn test_execute_camera_squad() {
        let mut g = test_game();
        g.map.teleport_squad("Mali", 10, 20).unwrap();
        g.execute_command(Command::CameraSquad("Mali".into()))
            .unwrap();
        assert_eq!(g.map.camera_x, 10);
    }

    #[test]
    fn test_execute_quit() {
        assert!(!test_game()
            .execute_command(Command::Quit)
            .unwrap());
    }
}
