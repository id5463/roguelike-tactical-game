use serde::{Serialize, Deserialize};
use crate::types::Position;

/// Priority order of skill types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SkillPriority {
    NormalAttack = 1,
    Move = 2,
    SkillPoint = 3,
    Cooldown = 4,
    Energy = 5,
    Ultimate = 6,
}

/// Target for a skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTarget {
    Self_,
    AllyFront,
    AllyMid,
    AllyBack,
    AllyAll,
    AllyAdjacent,
    AllyLowestHp,
    EnemyFront,
    EnemyMid,
    EnemyBack,
    EnemyAll,
    EnemyLowestHp,
    EnemyHighestAtk,
    AllAllies,
    AllEnemies,
    PositionTarget(Position),
}

/// Type of effect a skill applies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffect {
    Damage { multiplier: f64, target: SkillTarget },
    Heal { multiplier: f64, target: SkillTarget },
    BuffAtk { percent: f64, duration: i32, target: SkillTarget },
    BuffDef { percent: f64, duration: i32, target: SkillTarget },
    BuffSpd { percent: f64, duration: i32, target: SkillTarget },
    DebuffAtk { percent: f64, duration: i32, target: SkillTarget },
    DebuffDef { percent: f64, duration: i32, target: SkillTarget },
    DebuffSpd { percent: f64, duration: i32, target: SkillTarget },
    Shield { amount: i32, target: SkillTarget },
    Cleanse { target: SkillTarget },
    Revive { hp_percent: f64, target: SkillTarget },
    EnergyGain { amount: i32, target: SkillTarget },
    MoveSkill { range: i32 },
    SpeedBarBoost { percent: f64, target: SkillTarget },
    AoEDamage { multiplier: f64 },
    Dot { damage: i32, duration: i32, target: SkillTarget },
    Dispel { target: SkillTarget },
    Taunt { duration: i32 },
    Dodge { chance: f64, duration: i32 },
    Stun { duration: i32, target: SkillTarget },
}

/// A skill definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub priority: SkillPriority,
    pub description: String,
    pub effects: Vec<SkillEffect>,
    pub cooldown_max: i32,
    pub energy_cost: i32,
    pub sp_cost: i32,
}

impl Skill {
    pub fn normal_attack(name: &str, multiplier: f64, target: SkillTarget) -> Self {
        Self {
            name: name.to_string(),
            priority: SkillPriority::NormalAttack,
            description: format!("Normal attack dealing {:.1}x damage", multiplier),
            effects: vec![SkillEffect::Damage { multiplier, target }],
            cooldown_max: 0,
            energy_cost: 0,
            sp_cost: 0,
        }
    }
    pub fn move_skill(name: &str, range: i32) -> Self {
        Self {
            name: name.to_string(),
            priority: SkillPriority::Move,
            description: format!("Move up to {} tiles", range),
            effects: vec![SkillEffect::MoveSkill { range }],
            cooldown_max: 0,
            energy_cost: 0,
            sp_cost: 0,
        }
    }
    pub fn sp_skill(name: &str, desc: &str, effects: Vec<SkillEffect>) -> Self {
        Self {
            name: name.to_string(),
            priority: SkillPriority::SkillPoint,
            description: desc.to_string(),
            effects,
            cooldown_max: 0,
            energy_cost: 0,
            sp_cost: 1,
        }
    }
    pub fn cd_skill(name: &str, desc: &str, cd: i32, effects: Vec<SkillEffect>) -> Self {
        Self {
            name: name.to_string(),
            priority: SkillPriority::Cooldown,
            description: desc.to_string(),
            effects,
            cooldown_max: cd,
            energy_cost: 0,
            sp_cost: 0,
        }
    }
    pub fn energy_skill(name: &str, desc: &str, effects: Vec<SkillEffect>) -> Self {
        Self {
            name: name.to_string(),
            priority: SkillPriority::Energy,
            description: desc.to_string(),
            effects,
            cooldown_max: 0,
            energy_cost: 100,
            sp_cost: 0,
        }
    }
    pub fn ultimate(name: &str, desc: &str, effects: Vec<SkillEffect>) -> Self {
        Self {
            name: name.to_string(),
            priority: SkillPriority::Ultimate,
            description: desc.to_string(),
            effects,
            cooldown_max: 0,
            energy_cost: 0,
            sp_cost: 0,
        }
    }
}

// --- Lord skills (reference template) ---
pub fn lord_normal_attack() -> Skill {
    Skill::normal_attack("Horizontal Slash", 1.0, SkillTarget::EnemyFront)
}

pub fn lord_move() -> Skill {
    Skill::move_skill("Infantry Move", 4)
}

pub fn lord_sp_skill() -> Skill {
    Skill::sp_skill("Speed Boost", "Increase own speed bar by 30%",
        vec![SkillEffect::SpeedBarBoost { percent: 0.3, target: SkillTarget::Self_ }])
}

pub fn lord_cd_skill() -> Skill {
    Skill::cd_skill("Inspire", "Increase speed of all nearby allies by 25% (3 CD)", 3,
        vec![SkillEffect::BuffSpd { percent: 0.25, duration: 3, target: SkillTarget::AllAllies }])
}

pub fn lord_energy_skill() -> Skill {
    Skill::energy_skill("All-Out Charge", "Increase speed of all allies on the map by 20%",
        vec![SkillEffect::BuffSpd { percent: 0.2, duration: 2, target: SkillTarget::AllAllies }])
}

pub fn lord_ultimate() -> Skill {
    Skill::ultimate("Double Time", "Double own speed for this battle",
        vec![SkillEffect::BuffSpd { percent: 1.0, duration: 99, target: SkillTarget::Self_ }])
}
