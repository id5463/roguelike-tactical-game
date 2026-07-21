use serde::{Serialize, Deserialize};
use crate::rng::SeededRng;

/// Event choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChoice {
    pub label: String,
    pub result_text: String,
    pub effect: EventEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventEffect {
    GainEquipments(u32),
    GainRelic,
    HealAllPercent(f64),
    HealOnePercent(f64),
    DamageOnePercent(f64),
    GainGold(i32),
    GainPotions(u32),
    GainFood(u32),
    LordXp(f64),
    TeamAtkBuff(f64),
    PoisonOne(u32, i32),
    SummonEnemies,
    SacrificeCharacter,
    Nothing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    pub title: String,
    pub description: String,
    pub choices: Vec<EventChoice>,
}

pub fn generate_random_event(rng: &mut SeededRng) -> EventDef {
    let all_events = all_events();
    let idx = rng.gen_range(0, all_events.len() as i32 - 1) as usize;
    all_events[idx].clone()
}

pub fn all_events() -> Vec<EventDef> {
    vec![
        EventDef {
            title: "Ancient Treasure".into(),
            description: "You discover an ancient treasure chest half-buried in the dirt.".into(),
            choices: vec![
                EventChoice {
                    label: "A: Open carefully".into(),
                    result_text: "You gently pry open the chest, finding 3 pieces of equipment, but a needle trap pricks one character for 10% HP.".into(),
                    effect: EventEffect::GainEquipments(3),
                },
                EventChoice {
                    label: "B: Smash it open".into(),
                    result_text: "You shatter the chest, grabbing 5 pieces of equipment. A blade trap slices one character for 20% HP.".into(),
                    effect: EventEffect::GainEquipments(5),
                },
                EventChoice {
                    label: "C: Walk away".into(),
                    result_text: "You leave the chest undisturbed.".into(),
                    effect: EventEffect::Nothing,
                },
            ],
        },
        EventDef {
            title: "Wandering Merchant".into(),
            description: "A mysterious merchant in a hooded cloak offers to trade.".into(),
            choices: vec![
                EventChoice {
                    label: "A: Buy a relic (50 gold)".into(),
                    result_text: "You exchange 50 gold for a mysterious relic.".into(),
                    effect: EventEffect::GainRelic,
                },
                EventChoice {
                    label: "B: Trade 2 equipment for better one".into(),
                    result_text: "The merchant upgrades two of your items into a stronger piece.".into(),
                    effect: EventEffect::GainEquipments(1),
                },
                EventChoice {
                    label: "C: Decline".into(),
                    result_text: "You politely refuse.".into(),
                    effect: EventEffect::Nothing,
                },
            ],
        },
        EventDef {
            title: "Healing Spring".into(),
            description: "A crystal-clear spring bubbles with restorative waters.".into(),
            choices: vec![
                EventChoice {
                    label: "A: Drink from the spring".into(),
                    result_text: "All characters recover 50% HP.".into(),
                    effect: EventEffect::HealAllPercent(0.50),
                },
                EventChoice {
                    label: "B: Bathe fully".into(),
                    result_text: "All characters fully recover, but one is poisoned for 3 turns.".into(),
                    effect: EventEffect::HealAllPercent(1.00),
                },
                EventChoice {
                    label: "C: Bottle the water".into(),
                    result_text: "You collect 2 healing potions.".into(),
                    effect: EventEffect::GainPotions(2),
                },
            ],
        },
        EventDef {
            title: "Training Ground".into(),
            description: "An abandoned training ground with rusted practice dummies.".into(),
            choices: vec![
                EventChoice {
                    label: "A: Train the Lord".into(),
                    result_text: "The Lord gains valuable experience (+20% level progress).".into(),
                    effect: EventEffect::LordXp(0.20),
                },
                EventChoice {
                    label: "B: Train all teams".into(),
                    result_text: "All teams gain +20% ATK for this floor.".into(),
                    effect: EventEffect::TeamAtkBuff(0.20),
                },
                EventChoice {
                    label: "C: Rest".into(),
                    result_text: "You take a peaceful rest.".into(),
                    effect: EventEffect::Nothing,
                },
            ],
        },
        EventDef {
            title: "Mysterious Altar".into(),
            description: "An ancient stone altar hums with dark energy.".into(),
            choices: vec![
                EventChoice {
                    label: "A: Sacrifice a character".into(),
                    result_text: "The character vanishes; you gain a powerful relic.".into(),
                    effect: EventEffect::SacrificeCharacter,
                },
                EventChoice {
                    label: "B: Offer 10% max HP".into(),
                    result_text: "All characters gain +10% ATK for this run.".into(),
                    effect: EventEffect::DamageOnePercent(0.10),
                },
                EventChoice {
                    label: "C: Ignore it".into(),
                    result_text: "You pass by the altar.".into(),
                    effect: EventEffect::Nothing,
                },
            ],
        },
        EventDef {
            title: "Supply Cache".into(),
            description: "A hidden supply depot stocked with provisions.".into(),
            choices: vec![
                EventChoice {
                    label: "A: Take food".into(),
                    result_text: "You gain 3 rations of food.".into(),
                    effect: EventEffect::GainFood(3),
                },
                EventChoice {
                    label: "B: Take potions".into(),
                    result_text: "You gain 3 potions.".into(),
                    effect: EventEffect::GainPotions(3),
                },
                EventChoice {
                    label: "C: Grab everything".into(),
                    result_text: "You get 2 food + 2 potions, but trigger an alarm (enemies pursue)!".into(),
                    effect: EventEffect::SummonEnemies,
                },
            ],
        },
        EventDef {
            title: "Wounded Traveler".into(),
            description: "An injured traveler lies by the roadside begging for help.".into(),
            choices: vec![
                EventChoice {
                    label: "A: Share a potion".into(),
                    result_text: "The grateful traveler gives you 30 gold and a relic.".into(),
                    effect: EventEffect::GainRelic,
                },
                EventChoice {
                    label: "B: Give food".into(),
                    result_text: "The traveler thanks you with a piece of equipment.".into(),
                    effect: EventEffect::GainEquipments(1),
                },
                EventChoice {
                    label: "C: Pass by".into(),
                    result_text: "You continue on your way.".into(),
                    effect: EventEffect::Nothing,
                },
            ],
        },
        EventDef {
            title: "Ancient Library".into(),
            description: "Dusty shelves hold forgotten tomes of knowledge.".into(),
            choices: vec![
                EventChoice {
                    label: "A: Study tactics".into(),
                    result_text: "The Lord gains experience (+15% level progress).".into(),
                    effect: EventEffect::LordXp(0.15),
                },
                EventChoice {
                    label: "B: Take a valuable book".into(),
                    result_text: "You sell the tome for 40 gold.".into(),
                    effect: EventEffect::GainGold(40),
                },
                EventChoice {
                    label: "C: Leave".into(),
                    result_text: "You depart.".into(),
                    effect: EventEffect::Nothing,
                },
            ],
        },
    ]
}
