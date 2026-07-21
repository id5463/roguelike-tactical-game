use serde::{Serialize, Deserialize};
use crate::rng::SeededRng;
use crate::characters::{CharacterTemplate, Character, all_templates};
use crate::equipment::{Equipment, all_equipment_templates, PassiveAbility};
use crate::relics::{Relic, all_relic_templates};
use crate::types::Consumable;
use crate::types::ConsumableKind;

/// Pool of unlocked characters (meta-progression).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GachaPool {
    pub unlocked_char_ids: Vec<u32>,
    pub owned_characters: Vec<CharacterTemplate>,
    pub owned_equipment: Vec<Equipment>,
    pub owned_relics: Vec<Relic>,
    pub owned_potions: Vec<Consumable>,
    pub owned_food: Vec<Consumable>,
    pub duplicate_char_counts: std::collections::HashMap<u32, u32>,
}

impl GachaPool {
    pub fn new() -> Self {
        Self {
            unlocked_char_ids: Vec::new(),
            owned_characters: Vec::new(),
            owned_equipment: Vec::new(),
            owned_relics: Vec::new(),
            owned_potions: vec![
                Consumable { name: "Healing Potion".into(), kind: ConsumableKind::Potion, uses: 1 },
                Consumable { name: "Healing Potion".into(), kind: ConsumableKind::Potion, uses: 1 },
            ],
            owned_food: vec![
                Consumable { name: "Travel Bread".into(), kind: ConsumableKind::Food, uses: 3 },
            ],
            duplicate_char_counts: std::collections::HashMap::new(),
        }
    }

    /// Perform N gacha pulls, returns the pulled character templates.
    pub fn pull_n(&mut self, n: u32, rng: &mut SeededRng) -> Vec<CharacterTemplate> {
        let templates = all_templates();
        let mut pulled = Vec::new();
        for _ in 0..n {
            let idx = rng.gen_range(0, templates.len() as i32 - 1) as usize;
            let tmpl = templates[idx].clone();
            // Track duplicates
            *self.duplicate_char_counts.entry(tmpl.id).or_insert(0) += 1;
            // Add to owned if not already present
            if !self.owned_characters.iter().any(|c| c.id == tmpl.id) {
                self.owned_characters.push(tmpl.clone());
                self.unlocked_char_ids.push(tmpl.id);
            }
            pulled.push(tmpl);
        }
        pulled
    }

    /// Roll N random equipment pieces.
    pub fn roll_equipment(n: u32, rng: &mut SeededRng) -> Vec<Equipment> {
        let templates = all_equipment_templates();
        let mut result = Vec::new();
        for _ in 0..n {
            let idx = rng.gen_range(0, templates.len() as i32 - 1) as usize;
            result.push(templates[idx].clone());
        }
        result
    }

    /// Maybe roll a relic (rare).
    pub fn roll_relic(rng: &mut SeededRng) -> Option<Relic> {
        if rng.gen_bool(0.15) {
            let templates = all_relic_templates();
            let idx = rng.gen_range(0, templates.len() as i32 - 1) as usize;
            Some(templates[idx].clone())
        } else {
            None
        }
    }

    /// Roll consumables (food/potions).
    pub fn roll_consumables(rng: &mut SeededRng) -> Vec<Consumable> {
        let mut result = Vec::new();
        if rng.gen_bool(0.5) {
            result.push(Consumable {
                name: "Healing Potion".into(),
                kind: ConsumableKind::Potion,
                uses: 1,
            });
        }
        if rng.gen_bool(0.4) {
            result.push(Consumable {
                name: "Strength Elixir".into(),
                kind: ConsumableKind::Potion,
                uses: 1,
            });
        }
        if rng.gen_bool(0.3) {
            result.push(Consumable {
                name: "Travel Bread".into(),
                kind: ConsumableKind::Food,
                uses: 3,
            });
        }
        result
    }

    /// Merge two identical characters to raise star level (star up).
    pub fn merge_characters(&mut self, char_id: u32) -> bool {
        let count = self.duplicate_char_counts.get(&char_id).copied().unwrap_or(0);
        if count >= 2 {
            self.duplicate_char_counts.insert(char_id, count - 2);
            true
        } else {
            false
        }
    }

    /// Apply equipment to a character by adding passive ability modifiers.
    pub fn apply_equipment_stats(ch: &mut Character, equips: &[Equipment]) {
        for eq in equips {
            for ab in &eq.abilities {
                match ab {
                    PassiveAbility::AtkPercent(p) => { ch.atk = (ch.atk as f64 * (1.0 + p)) as i32; }
                    PassiveAbility::DefPercent(p) => { ch.def = (ch.def as f64 * (1.0 + p)) as i32; }
                    PassiveAbility::HpPercent(p) => {
                        ch.max_hp = (ch.max_hp as f64 * (1.0 + p)) as i32;
                        ch.hp = ch.max_hp;
                    }
                    PassiveAbility::SpdFlat(s) => { ch.spd += s; }
                    PassiveAbility::CritPercent(_) => {} // handled in damage calc
                    PassiveAbility::MoveBonus(m) => { ch.atk += 0; let _ = m; }
                    _ => {}
                }
            }
        }
    }
}
