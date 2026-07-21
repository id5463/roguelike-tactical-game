use serde::{Serialize, Deserialize};

/// All recognized game commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Help,
    Status,
    Select(u32),
    Continue,
    Quit,
    // Combat
    Move { team_id: u32, direction: crate::types::Direction, distance: Option<i32> },
    Attack(u32),
    Skill { team_id: u32, skill_name: String },
    CdSkill { team_id: u32, skill_name: String },
    EnergySkill { team_id: u32, skill_name: String },
    Ultimate(u32),
    Normal(u32),
    Wait(u32),
    // Save/Load
    Save(String),
    Load(String),
    ListSaves,
    ViewSave(String),
    // Map
    ShowMap,
    GotoNode(u32),
    TeamList,
    Inventory,
    Gacha,
    Merge(u32, u32),
    // Sub-battle
    SubSkill { character: String, skill_name: String },
    SubPassive { character: String, skill_name: String },
    SubRetreat,
    SubStatus,
    // Campfire
    CampfireRevive,
    CampfireUpgrade(u32),
    CampfireRest,
    CampfireLeave,
    // Shop
    ShopBuy(u32),
    ShopLeave,
    // Event
    EventChoice(u32),
    // Potions/Food
    UsePotion(String),
    UseFood(String),
    Unknown(String),
}

pub fn parse_command(input: &str) -> Command {
    let input = input.trim();
    if input.is_empty() {
        return Command::Unknown(String::new());
    }
    let parts: Vec<&str> = input.split_whitespace().collect();
    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        "help" | "h" | "?" => Command::Help,
        "status" | "st" => Command::Status,
        "select" | "sel" => {
            if parts.len() < 2 { return Command::Unknown("select requires team_id".into()); }
            match parts[1].parse::<u32>() {
                Ok(id) => Command::Select(id),
                Err(_) => Command::Unknown(format!("Invalid team_id: {}", parts[1])),
            }
        }
        "continue" | "c" | "cont" => Command::Continue,
        "quit" | "exit" | "q" => Command::Quit,
        "move" | "mv" => {
            if parts.len() < 3 { return Command::Unknown("move requires team_id and direction".into()); }
            let team_id = match parts[1].parse::<u32>() { Ok(v) => v, Err(_) => return Command::Unknown("Invalid team_id".into()) };
            let direction = match crate::types::Direction::from_str(parts[2]) {
                Some(d) => d,
                None => return Command::Unknown(format!("Invalid direction: {}. Use N/S/E/W/NE/NW/SE/SW", parts[2])),
            };
            let distance = if parts.len() >= 4 { parts[3].parse::<i32>().ok() } else { None };
            Command::Move { team_id, direction, distance }
        }
        "attack" | "atk" => {
            if parts.len() < 2 { return Command::Unknown("attack requires team_id".into()); }
            match parts[1].parse::<u32>() {
                Ok(id) => Command::Attack(id),
                Err(_) => Command::Unknown("Invalid team_id".into()),
            }
        }
        "skill" | "sp" => {
            if parts.len() < 3 { return Command::Unknown("skill requires team_id and skill name".into()); }
            let team_id = match parts[1].parse::<u32>() { Ok(v) => v, Err(_) => return Command::Unknown("Invalid team_id".into()) };
            let skill_name = parts[2..].join(" ");
            Command::Skill { team_id, skill_name }
        }
        "cdskill" | "cd" => {
            if parts.len() < 3 { return Command::Unknown("cdskill requires team_id and skill name".into()); }
            let team_id = match parts[1].parse::<u32>() { Ok(v) => v, Err(_) => return Command::Unknown("Invalid team_id".into()) };
            let skill_name = parts[2..].join(" ");
            Command::CdSkill { team_id, skill_name }
        }
        "energyskill" | "energy" | "es" => {
            if parts.len() < 3 { return Command::Unknown("energyskill requires team_id and skill name".into()); }
            let team_id = match parts[1].parse::<u32>() { Ok(v) => v, Err(_) => return Command::Unknown("Invalid team_id".into()) };
            let skill_name = parts[2..].join(" ");
            Command::EnergySkill { team_id, skill_name }
        }
        "ultimate" | "ult" | "u" => {
            if parts.len() < 2 { return Command::Unknown("ultimate requires team_id".into()); }
            match parts[1].parse::<u32>() {
                Ok(id) => Command::Ultimate(id),
                Err(_) => Command::Unknown("Invalid team_id".into()),
            }
        }
        "normal" | "n" => {
            if parts.len() < 2 { return Command::Unknown("normal requires team_id".into()); }
            match parts[1].parse::<u32>() {
                Ok(id) => Command::Normal(id),
                Err(_) => Command::Unknown("Invalid team_id".into()),
            }
        }
        "wait" | "w" => {
            if parts.len() < 2 { return Command::Unknown("wait requires team_id".into()); }
            match parts[1].parse::<u32>() {
                Ok(id) => Command::Wait(id),
                Err(_) => Command::Unknown("Invalid team_id".into()),
            }
        }
        "save" => {
            if parts.len() < 2 { return Command::Unknown("save requires slot name".into()); }
            Command::Save(parts[1].to_string())
        }
        "load" => {
            if parts.len() < 2 { return Command::Unknown("load requires slot name".into()); }
            Command::Load(parts[1].to_string())
        }
        "listsaves" | "ls" => Command::ListSaves,
        "viewsave" => {
            if parts.len() < 2 { return Command::Unknown("viewsave requires slot name".into()); }
            Command::ViewSave(parts[1].to_string())
        }
        "map" => Command::ShowMap,
        "node" => {
            if parts.len() < 2 { return Command::Unknown("node requires node_id".into()); }
            match parts[1].parse::<u32>() {
                Ok(id) => Command::GotoNode(id),
                Err(_) => Command::Unknown("Invalid node_id".into()),
            }
        }
        "teamlist" | "teams" | "tl" => Command::TeamList,
        "inventory" | "inv" | "i" => Command::Inventory,
        "gacha" => Command::Gacha,
        "merge" => {
            if parts.len() < 3 { return Command::Unknown("merge requires two character IDs".into()); }
            let a = match parts[1].parse::<u32>() { Ok(v) => v, Err(_) => return Command::Unknown("Invalid char id".into()) };
            let b = match parts[2].parse::<u32>() { Ok(v) => v, Err(_) => return Command::Unknown("Invalid char id".into()) };
            Command::Merge(a, b)
        }
        "sub_skill" => {
            if parts.len() < 3 { return Command::Unknown("sub_skill requires character and skill name".into()); }
            Command::SubSkill { character: parts[1].to_string(), skill_name: parts[2..].join(" ") }
        }
        "sub_passive" => {
            if parts.len() < 3 { return Command::Unknown("sub_passive requires character and skill name".into()); }
            Command::SubPassive { character: parts[1].to_string(), skill_name: parts[2..].join(" ") }
        }
        "sub_retreat" | "retreat" => Command::SubRetreat,
        "sub_status" => Command::SubStatus,
        // Campfire shortcuts A/B/C/D
        "a" | "revive" => Command::CampfireRevive,
        "b" | "upgrade" => {
            let tid = if parts.len() >= 2 { parts[1].parse::<u32>().ok() } else { None };
            Command::CampfireUpgrade(tid.unwrap_or(1))
        }
        "rest" => Command::CampfireRest,
        "d" | "leave" => Command::CampfireLeave,
        "buy" => {
            if parts.len() < 2 { return Command::Unknown("buy requires item number".into()); }
            match parts[1].parse::<u32>() {
                Ok(n) => Command::ShopBuy(n),
                Err(_) => Command::Unknown("Invalid item number".into()),
            }
        }
        "usepotion" | "potion" => {
            if parts.len() < 2 { return Command::Unknown("usepotion requires name".into()); }
            Command::UsePotion(parts[1..].join(" "))
        }
        "usefood" | "food" => {
            if parts.len() < 2 { return Command::Unknown("usefood requires name".into()); }
            Command::UseFood(parts[1..].join(" "))
        }
        _ => {
            // Check for numeric event choice: "1", "2", "3"
            if let Ok(n) = cmd.parse::<u32>() {
                return Command::EventChoice(n);
            }
            Command::Unknown(format!("Unknown command: {}", input))
        }
    }
}
