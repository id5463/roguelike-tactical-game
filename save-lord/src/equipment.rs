use serde::{Serialize, Deserialize};

/// Passive ability granted by equipment (stat modifiers / passive effects).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassiveAbility {
    AtkPercent(f64),
    DefPercent(f64),
    HpPercent(f64),
    SpdFlat(i32),
    CritPercent(f64),
    HealPercent(f64),
    MoveBonus(i32),
    HpRegenPercent(f64),
    Lifesteal(f64),
    DamageToBoss(f64),
    DamageReduction(f64),
    EnergyGainPercent(f64),
    CounterChance(f64),
    DodgeChance(f64),
}

impl PassiveAbility {
    pub fn description(&self) -> String {
        match self {
            PassiveAbility::AtkPercent(p) => format!("ATK +{:.0}%", p * 100.0),
            PassiveAbility::DefPercent(p) => format!("DEF +{:.0}%", p * 100.0),
            PassiveAbility::HpPercent(p) => format!("HP +{:.0}%", p * 100.0),
            PassiveAbility::SpdFlat(s) => format!("SPD +{}", s),
            PassiveAbility::CritPercent(p) => format!("Crit +{:.0}%", p * 100.0),
            PassiveAbility::HealPercent(p) => format!("Healing +{:.0}%", p * 100.0),
            PassiveAbility::MoveBonus(m) => format!("Move +{}", m),
            PassiveAbility::HpRegenPercent(p) => format!("Regen {:.1}% HP/turn", p * 100.0),
            PassiveAbility::Lifesteal(p) => format!("Lifesteal {:.0}%", p * 100.0),
            PassiveAbility::DamageToBoss(p) => format!("Damage vs Boss +{:.0}%", p * 100.0),
            PassiveAbility::DamageReduction(p) => format!("Damage Taken -{:.0}%", p * 100.0),
            PassiveAbility::EnergyGainPercent(p) => format!("Energy Gain +{:.0}%", p * 100.0),
            PassiveAbility::CounterChance(p) => format!("Counter {:.0}%", p * 100.0),
            PassiveAbility::DodgeChance(p) => format!("Dodge {:.0}%", p * 100.0),
        }
    }
}

/// Equipment item — provides passive abilities only, no active skills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    pub name: String,
    pub abilities: Vec<PassiveAbility>,
}

/// Generate all possible equipment templates.
pub fn all_equipment_templates() -> Vec<Equipment> {
    vec![
        Equipment { name: "Iron Sword".into(), abilities: vec![PassiveAbility::AtkPercent(0.10)] },
        Equipment { name: "Steel Blade".into(), abilities: vec![PassiveAbility::AtkPercent(0.15)] },
        Equipment { name: "Flame Sword".into(), abilities: vec![PassiveAbility::AtkPercent(0.20), PassiveAbility::CritPercent(0.05)] },
        Equipment { name: "Frost Axe".into(), abilities: vec![PassiveAbility::AtkPercent(0.18), PassiveAbility::SpdFlat(-5)] },
        Equipment { name: "Thunder Spear".into(), abilities: vec![PassiveAbility::AtkPercent(0.12), PassiveAbility::SpdFlat(5)] },
        Equipment { name: "Shadow Dagger".into(), abilities: vec![PassiveAbility::AtkPercent(0.08), PassiveAbility::CritPercent(0.10)] },
        Equipment { name: "Holy Mace".into(), abilities: vec![PassiveAbility::AtkPercent(0.10), PassiveAbility::HealPercent(0.15)] },
        Equipment { name: "Demon Blade".into(), abilities: vec![PassiveAbility::AtkPercent(0.25), PassiveAbility::DefPercent(-0.10)] },
        Equipment { name: "Dragon Slayer".into(), abilities: vec![PassiveAbility::AtkPercent(0.30), PassiveAbility::DamageToBoss(0.20)] },
        Equipment { name: "Leather Armor".into(), abilities: vec![PassiveAbility::DefPercent(0.10)] },
        Equipment { name: "Chain Mail".into(), abilities: vec![PassiveAbility::DefPercent(0.15)] },
        Equipment { name: "Plate Armor".into(), abilities: vec![PassiveAbility::DefPercent(0.25), PassiveAbility::SpdFlat(-10)] },
        Equipment { name: "Mage Robe".into(), abilities: vec![PassiveAbility::DefPercent(0.05), PassiveAbility::HpPercent(0.10)] },
        Equipment { name: "Dragon Scale".into(), abilities: vec![PassiveAbility::DefPercent(0.20), PassiveAbility::HpPercent(0.15)] },
        Equipment { name: "Shadow Cloak".into(), abilities: vec![PassiveAbility::DodgeChance(0.10)] },
        Equipment { name: "Guardian Shield".into(), abilities: vec![PassiveAbility::DefPercent(0.20), PassiveAbility::DamageReduction(0.05)] },
        Equipment { name: "Health Amulet".into(), abilities: vec![PassiveAbility::HpPercent(0.20)] },
        Equipment { name: "Speed Boots".into(), abilities: vec![PassiveAbility::SpdFlat(10)] },
        Equipment { name: "Winged Sandals".into(), abilities: vec![PassiveAbility::SpdFlat(8), PassiveAbility::MoveBonus(1)] },
        Equipment { name: "Ring of Vitality".into(), abilities: vec![PassiveAbility::HpPercent(0.15), PassiveAbility::HpRegenPercent(0.03)] },
        Equipment { name: "Ring of Power".into(), abilities: vec![PassiveAbility::AtkPercent(0.10), PassiveAbility::CritPercent(0.05)] },
        Equipment { name: "Vampire Fangs".into(), abilities: vec![PassiveAbility::AtkPercent(0.08), PassiveAbility::Lifesteal(0.15)] },
        Equipment { name: "Bloodstone".into(), abilities: vec![PassiveAbility::HpPercent(0.10), PassiveAbility::Lifesteal(0.10)] },
        Equipment { name: "Energy Crystal".into(), abilities: vec![PassiveAbility::SpdFlat(5), PassiveAbility::EnergyGainPercent(0.20)] },
        Equipment { name: "Counter Gauntlets".into(), abilities: vec![PassiveAbility::AtkPercent(0.10), PassiveAbility::CounterChance(0.15)] },
        Equipment { name: "Healer's Staff".into(), abilities: vec![PassiveAbility::HealPercent(0.25)] },
        Equipment { name: "Berserker Helm".into(), abilities: vec![PassiveAbility::AtkPercent(0.15), PassiveAbility::DefPercent(-0.05)] },
        Equipment { name: "Titan Belt".into(), abilities: vec![PassiveAbility::HpPercent(0.25), PassiveAbility::DefPercent(0.05)] },
        Equipment { name: "Lucky Coin".into(), abilities: vec![PassiveAbility::CritPercent(0.08), PassiveAbility::DodgeChance(0.05)] },
        Equipment { name: "Phoenix Feather".into(), abilities: vec![PassiveAbility::HpRegenPercent(0.05)] },
        Equipment { name: "Ancient Tome".into(), abilities: vec![PassiveAbility::AtkPercent(0.12), PassiveAbility::SpdFlat(5)] },
        Equipment { name: "War Banner".into(), abilities: vec![PassiveAbility::AtkPercent(0.05), PassiveAbility::DefPercent(0.05)] },
        Equipment { name: "Spirit Mask".into(), abilities: vec![PassiveAbility::DamageReduction(0.10)] },
        Equipment { name: "Wind Bow".into(), abilities: vec![PassiveAbility::AtkPercent(0.14), PassiveAbility::SpdFlat(8)] },
        Equipment { name: "Toxic Blade".into(), abilities: vec![PassiveAbility::AtkPercent(0.12), PassiveAbility::CritPercent(0.03)] },
    ]
}
