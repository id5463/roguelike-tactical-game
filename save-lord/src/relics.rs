use serde::{Serialize, Deserialize};

/// Global relic effects (similar to Slay the Spire relics).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelicEffect {
    GlobalAtkPercent(f64),
    GlobalDefPercent(f64),
    GlobalHpPercent(f64),
    GlobalSpdPercent(f64),
    StartingSkillPoints(i32),
    CampfireEffectDouble,
    ShopDiscount(f64),
    FloorStartHealPercent(f64),
    SpeedBarBoostPercent(f64),
    ExtraActivePoints(i32),
    LordShield(i32),
    GoldPerBattle(i32),
    CritBonus(f64),
    DamageToBoss(f64),
    ReviveOnDeath,
    ExtraGachaPulls(i32),
    EnergyStartPercent(f64),
    MoveBonus(i32),
}

impl RelicEffect {
    pub fn description(&self) -> String {
        match self {
            RelicEffect::GlobalAtkPercent(p) => format!("All teams ATK +{:.0}%", p * 100.0),
            RelicEffect::GlobalDefPercent(p) => format!("All teams DEF +{:.0}%", p * 100.0),
            RelicEffect::GlobalHpPercent(p) => format!("All teams HP +{:.0}%", p * 100.0),
            RelicEffect::GlobalSpdPercent(p) => format!("All teams SPD +{:.0}%", p * 100.0),
            RelicEffect::StartingSkillPoints(n) => format!("Start each battle with {} Skill Points", n),
            RelicEffect::CampfireEffectDouble => "Campfire effects doubled".to_string(),
            RelicEffect::ShopDiscount(d) => format!("Shop items {:.0}% off", (1.0 - d) * 100.0),
            RelicEffect::FloorStartHealPercent(p) => format!("Heal {:.0}% HP at floor start", p * 100.0),
            RelicEffect::SpeedBarBoostPercent(p) => format!("Speed bar +{:.0}%", p * 100.0),
            RelicEffect::ExtraActivePoints(n) => format!("Extra {} active points in sub-battles", n),
            RelicEffect::LordShield(hp) => format!("Lord gains {} shield HP", hp),
            RelicEffect::GoldPerBattle(n) => format!("Gain {} gold per battle", n),
            RelicEffect::CritBonus(p) => format!("Crit rate +{:.0}%", p * 100.0),
            RelicEffect::DamageToBoss(p) => format!("Damage to Boss +{:.0}%", p * 100.0),
            RelicEffect::ReviveOnDeath => "Revive once per battle with 50% HP".to_string(),
            RelicEffect::ExtraGachaPulls(n) => format!("{} extra gacha pulls per battle", n),
            RelicEffect::EnergyStartPercent(p) => format!("Start battles with {:.0}% Energy", p * 100.0),
            RelicEffect::MoveBonus(n) => format!("All units move +{} tiles", n),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relic {
    pub name: String,
    pub effect: RelicEffect,
    pub description: String,
}

pub fn all_relic_templates() -> Vec<Relic> {
    vec![
        Relic { name: "Burning Blood".into(), effect: RelicEffect::FloorStartHealPercent(0.20), description: "Heal 20% HP at start of each floor".into() },
        Relic { name: "War Horn".into(), effect: RelicEffect::GlobalAtkPercent(0.15), description: "All teams gain 15% ATK".into() },
        Relic { name: "Iron Bulwark".into(), effect: RelicEffect::GlobalDefPercent(0.15), description: "All teams gain 15% DEF".into() },
        Relic { name: "Ancient Codex".into(), effect: RelicEffect::StartingSkillPoints(3), description: "Start each battle with 3 Skill Points".into() },
        Relic { name: "Eternal Flame".into(), effect: RelicEffect::CampfireEffectDouble, description: "Campfire effects are doubled".into() },
        Relic { name: "Merchant Seal".into(), effect: RelicEffect::ShopDiscount(0.80), description: "Shop items are 20% off".into() },
        Relic { name: "Wind Chime".into(), effect: RelicEffect::SpeedBarBoostPercent(0.10), description: "Speed bar fills 10% faster".into() },
        Relic { name: "Battle Standard".into(), effect: RelicEffect::ExtraActivePoints(2), description: "Gain 2 extra active points in sub-battles".into() },
        Relic { name: "Lord's Crest".into(), effect: RelicEffect::LordShield(50), description: "Lord starts each battle with 50 shield HP".into() },
        Relic { name: "Golden Purse".into(), effect: RelicEffect::GoldPerBattle(25), description: "Gain 25 gold after every battle".into() },
        Relic { name: "Critical Eye".into(), effect: RelicEffect::CritBonus(0.08), description: "Critical hit rate +8%".into() },
        Relic { name: "Dragon Bane".into(), effect: RelicEffect::DamageToBoss(0.25), description: "Deal 25% more damage to Bosses".into() },
        Relic { name: "Phoenix Feather".into(), effect: RelicEffect::ReviveOnDeath, description: "Revive once per battle at 50% HP".into() },
        Relic { name: "Lucky Star".into(), effect: RelicEffect::ExtraGachaPulls(20), description: "20 extra gacha pulls per battle".into() },
        Relic { name: "Energy Core".into(), effect: RelicEffect::EnergyStartPercent(0.30), description: "Start battles with 30% Energy charged".into() },
        Relic { name: "Swift Boots".into(), effect: RelicEffect::MoveBonus(1), description: "All units gain +1 movement range".into() },
        Relic { name: "Vitality Stone".into(), effect: RelicEffect::GlobalHpPercent(0.20), description: "All teams gain 20% max HP".into() },
        Relic { name: "Haste Rune".into(), effect: RelicEffect::GlobalSpdPercent(0.10), description: "All teams gain 10% SPD".into() },
        Relic { name: "Sage Stone".into(), effect: RelicEffect::EnergyStartPercent(0.50), description: "Start battles with 50% Energy charged".into() },
        Relic { name: "Warlord Banner".into(), effect: RelicEffect::GlobalAtkPercent(0.10), description: "All teams gain 10% ATK".into() },
    ]
}
