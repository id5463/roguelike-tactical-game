use serde::{Serialize, Deserialize};

/// Position on the tactical grid (world coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self { Self { x, y } }
    pub fn distance(&self, other: GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
    pub fn chebyshev_distance(&self, other: GridPos) -> i32 {
        std::cmp::max((self.x - other.x).abs(), (self.y - other.y).abs())
    }
}

/// Direction for movement commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    N, S, E, W, NE, NW, SE, SW,
}

impl Direction {
    pub fn to_delta(self) -> (i32, i32) {
        match self {
            Direction::N  => (0, -1),
            Direction::S  => (0, 1),
            Direction::E  => (1, 0),
            Direction::W  => (-1, 0),
            Direction::NE => (1, -1),
            Direction::NW => (-1, -1),
            Direction::SE => (1, 1),
            Direction::SW => (-1, 1),
        }
    }
    pub fn from_str(s: &str) -> Option<Direction> {
        match s.to_uppercase().as_str() {
            "N" => Some(Direction::N),
            "S" => Some(Direction::S),
            "E" => Some(Direction::E),
            "W" => Some(Direction::W),
            "NE" => Some(Direction::NE),
            "NW" => Some(Direction::NW),
            "SE" => Some(Direction::SE),
            "SW" => Some(Direction::SW),
            _ => None,
        }
    }
}

/// Rank/Position within a team formation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Position {
    Frontline,
    Midline,
    Backline,
}

impl Position {
    pub fn as_str(&self) -> &'static str {
        match self {
            Position::Frontline => "FRONT",
            Position::Midline => "MID",
            Position::Backline => "BACK",
        }
    }
    pub fn from_str(s: &str) -> Option<Position> {
        match s.to_uppercase().as_str() {
            "FRONT" | "FRONTLINE" => Some(Position::Frontline),
            "MID" | "MIDLINE" => Some(Position::Midline),
            "BACK" | "BACKLINE" => Some(Position::Backline),
            _ => None,
        }
    }
}

/// Terrain type on the tactical map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Plain,
    Forest,
    Water,
    Obstacle,
}

impl Terrain {
    pub fn as_char(&self) -> char {
        match self {
            Terrain::Plain => '.',
            Terrain::Forest => 'F',
            Terrain::Water => '~',
            Terrain::Obstacle => '#',
        }
    }
    pub fn move_cost(&self, unit_type: UnitType) -> Option<i32> {
        match (self, unit_type) {
            (Terrain::Obstacle, UnitType::Flying) => Some(2),
            (Terrain::Obstacle, _) => None,
            (Terrain::Water, UnitType::Flying) => Some(1),
            (Terrain::Water, UnitType::Swimming) => Some(1),
            (Terrain::Water, _) => None,
            (Terrain::Forest, _) => Some(2),
            (Terrain::Plain, _) => Some(1),
        }
    }
}

/// Unit movement type for terrain interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    Infantry,
    Flying,
    Swimming,
    Cavalry,
}

/// Team/side identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Player,
    Enemy,
    Neutral,
}

/// A buff or debuff applied to a character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buff {
    pub name: String,
    pub duration: i32,
    pub atk_mod: f64,
    pub def_mod: f64,
    pub spd_mod: f64,
    pub dot: i32,
    pub is_debuff: bool,
}

/// Consumable items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumable {
    pub name: String,
    pub kind: ConsumableKind,
    pub uses: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsumableKind {
    Food,
    Potion,
}

/// Floor identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Floor(pub u32);

/// Node identifier within a floor (0..10 normal nodes + boss).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    pub floor: u32,
    pub index: u32,
}

/// Game phase tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Map,
    Campfire,
    Shop,
    Event,
    CombatTactical,
    SubBattle,
    BossIntro,
    Victory,
    Defeat,
}

/// Stats bundle for a character snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharStats {
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub spd: i32,
}
