use crate::error::{GameError, GameResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Teleport { name: String, x: u16, y: u16 },
    Camera { x: u16, y: u16 },
    CameraSquad(String),
    Help,
    Save { name: Option<String> },
    Load { name: Option<String> },
    Position,
    Quit,
}

pub struct CommandParser;

impl CommandParser {
    pub fn parse(input: &str) -> GameResult<Command> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(GameError::EmptyInput);
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "teleport" => {
                if parts.len() < 4 {
                    return Err(GameError::UnknownCommand(
                        "usage: teleport <squad_name> <x> <y>".into(),
                    ));
                }
                let name = parts[1].to_string();
                let x: u16 = parts[2].parse().map_err(|_| {
                    GameError::UnknownCommand(format!("invalid x coordinate: '{}'", parts[2]))
                })?;
                let y: u16 = parts[3].parse().map_err(|_| {
                    GameError::UnknownCommand(format!("invalid y coordinate: '{}'", parts[3]))
                })?;
                Ok(Command::Teleport { name, x, y })
            }
            "camera" => {
                if parts.len() < 2 {
                    return Err(GameError::UnknownCommand(
                        "usage: camera <x> <y>  or  camera <squad_name>".into(),
                    ));
                }
                if parts.len() >= 3 {
                    let x: u16 = parts[1].parse().map_err(|_| {
                        GameError::UnknownCommand(format!("invalid x: '{}'", parts[1]))
                    })?;
                    let y: u16 = parts[2].parse().map_err(|_| {
                        GameError::UnknownCommand(format!("invalid y: '{}'", parts[2]))
                    })?;
                    Ok(Command::Camera { x, y })
                } else {
                    Ok(Command::CameraSquad(parts[1].to_string()))
                }
            }
            "help" => Ok(Command::Help),
            "save" => {
                let name = parts.get(1).map(|s| s.to_string());
                Ok(Command::Save { name })
            }
            "load" => {
                let name = parts.get(1).map(|s| s.to_string());
                Ok(Command::Load { name })
            }
            "position" | "pos" => Ok(Command::Position),
            "quit" | "exit" => Ok(Command::Quit),
            _ => Err(GameError::UnknownCommand(trimmed.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teleport() {
        assert_eq!(
            CommandParser::parse("teleport Mali 45 30").unwrap(),
            Command::Teleport { name: "Mali".into(), x: 45, y: 30 }
        );
    }

    #[test]
    fn test_camera_coords() {
        assert_eq!(
            CommandParser::parse("camera 30 40").unwrap(),
            Command::Camera { x: 30, y: 40 }
        );
    }

    #[test]
    fn test_camera_squad() {
        assert_eq!(
            CommandParser::parse("camera Mali").unwrap(),
            Command::CameraSquad("Mali".into())
        );
    }

    #[test]
    fn test_help() {
        assert_eq!(CommandParser::parse("help").unwrap(), Command::Help);
    }

    #[test]
    fn test_save() {
        assert_eq!(
            CommandParser::parse("save").unwrap(),
            Command::Save { name: None }
        );
    }

    #[test]
    fn test_save_with_name() {
        assert_eq!(
            CommandParser::parse("save quick").unwrap(),
            Command::Save { name: Some("quick".into()) }
        );
    }

    #[test]
    fn test_position() {
        assert_eq!(CommandParser::parse("position").unwrap(), Command::Position);
    }

    #[test]
    fn test_quit() {
        assert_eq!(CommandParser::parse("quit").unwrap(), Command::Quit);
    }

    #[test]
    fn test_empty_input() {
        assert!(CommandParser::parse("").is_err());
    }

    #[test]
    fn test_unknown_command() {
        assert!(CommandParser::parse("foobar").is_err());
    }
}
