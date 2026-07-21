#![allow(dead_code)]
#[allow(dead_code)]
use serde::{Serialize, Deserialize};
use crate::rng::SeededRng;
use crate::characters::Character;
use crate::skills::{SkillEffect, SkillTarget};
use crate::team::Team;
use crate::types::{Position, Side, Buff};

/// Result of a sub-battle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubBattleResult {
    AttackerWins,
    DefenderWins,
    Draw,
    Retreat,
}

/// A sub-battle instance between two teams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubBattle {
    pub attacker_side: Side,
    pub attacker_chars: Vec<Character>,
    pub defender_chars: Vec<Character>,
    pub turn: i32,
    pub log: Vec<String>,
    pub skill_points: i32,
    pub total_damage_dealt_by_player: i64,
    pub result: Option<SubBattleResult>,
    pub active_phase_done: bool,
}

impl SubBattle {
    pub fn new(attacker: &Team, defender: &Team, starting_sp: i32, extra_active: i32, energy_start: f64) -> Self {
        let mut attacker_chars: Vec<Character> = attacker.members.iter()
            .filter(|c| !c.is_dead)
            .cloned()
            .collect();
        let mut defender_chars: Vec<Character> = defender.members.iter()
            .filter(|c| !c.is_dead)
            .cloned()
            .collect();

        // Apply extra active points from relics
        for c in &mut attacker_chars {
            c.max_active_points = (c.max_active_points as i32 + extra_active) as u32;
            c.active_points = c.max_active_points;
            c.energy = (100.0 * energy_start) as i32;
        }
        for c in &mut defender_chars {
            c.max_active_points = (c.max_active_points as i32 + extra_active) as u32;
            c.active_points = c.max_active_points;
            c.energy = (100.0 * energy_start) as i32;
        }

        // Sort by position: Front → Mid → Back
        sort_by_position(&mut attacker_chars);
        sort_by_position(&mut defender_chars);

        Self {
            attacker_side: attacker.side,
            attacker_chars,
            defender_chars,
            turn: 0,
            log: Vec::new(),
            skill_points: starting_sp,
            total_damage_dealt_by_player: 0,
            result: None,
            active_phase_done: false,
        }
    }

    pub fn run_complete_battle(&mut self, rng: &mut SeededRng, is_player_attacker: bool) {
        // Run active and passive phases until battle ends
        let mut max_rounds = 20;
        while self.result.is_none() && max_rounds > 0 {
            self.run_active_phase(rng, is_player_attacker);
            if self.result.is_some() { break; }
            self.run_passive_phase(rng, is_player_attacker);
            self.check_battle_end();
            self.turn += 1;
            max_rounds -= 1;
        }
        if self.result.is_none() {
            // Compare remaining HP
            let att_hp = total_hp_percent(&self.attacker_chars);
            let def_hp = total_hp_percent(&self.defender_chars);
            if att_hp > def_hp {
                self.result = Some(SubBattleResult::AttackerWins);
            } else if def_hp > att_hp {
                self.result = Some(SubBattleResult::DefenderWins);
            } else {
                self.result = Some(SubBattleResult::Draw);
            }
        }
    }

    fn run_active_phase(&mut self, rng: &mut SeededRng, is_player_attacker: bool) {
        // Attacker characters act first by SPD descending, then defenders
        let att_order = order_by_spd_desc(&self.attacker_chars);
        let def_order = order_by_spd_desc(&self.defender_chars);

        // Attacker phase
        for idx in att_order {
            if self.result.is_some() { break; }
            if !self.attacker_chars[idx].is_dead && self.attacker_chars[idx].active_points > 0 {
                self.character_act(idx, true, rng, is_player_attacker);
                self.attacker_chars[idx].active_points -= 1;
            }
            self.check_battle_end();
        }

        // Defender phase
        for idx in def_order {
            if self.result.is_some() { break; }
            if !self.defender_chars[idx].is_dead && self.defender_chars[idx].active_points > 0 {
                self.character_act(idx, false, rng, !is_player_attacker);
                self.defender_chars[idx].active_points -= 1;
            }
            self.check_battle_end();
        }

        // Tick cooldowns
        for c in &mut self.attacker_chars {
            if c.cd_remaining > 0 { c.cd_remaining -= 1; }
            // Gain energy from any action
            c.energy = (c.energy + 15).min(100);
        }
        for c in &mut self.defender_chars {
            if c.cd_remaining > 0 { c.cd_remaining -= 1; }
            c.energy = (c.energy + 15).min(100);
        }

        self.active_phase_done = true;
    }

    fn character_act(&mut self, char_idx: usize, is_attacker: bool, rng: &mut SeededRng, _is_player: bool) {
        let (actor_side_chars, other_side_chars) = if is_attacker {
            (&mut self.attacker_chars, &mut self.defender_chars)
        } else {
            (&mut self.defender_chars, &mut self.attacker_chars)
        };

        // AI decision: choose best action
        let (actor_stats, action_type) = decide_action(&actor_side_chars[char_idx], other_side_chars, actor_side_chars, self.skill_points);

        match action_type {
            ActionType::NormalAttack => {
                // Normal attack: grant 1 SP
                self.skill_points += 1;
                let tmpl_normal = actor_side_chars[char_idx].template.normal_attack.clone();
                self.log.push(format!("{} uses {}", actor_side_chars[char_idx].name(), tmpl_normal.name));
                self.apply_skill_effects(&tmpl_normal.effects, char_idx, is_attacker, rng);
            }
            ActionType::SpSkill => {
                self.skill_points -= 1;
                if let Some(sp_skill) = actor_side_chars[char_idx].template.sp_skill.clone() {
                    self.log.push(format!("{} uses {}", actor_side_chars[char_idx].name(), sp_skill.name));
                    self.apply_skill_effects(&sp_skill.effects, char_idx, is_attacker, rng);
                }
            }
            ActionType::CdSkill => {
                if let Some(cd_skill) = actor_side_chars[char_idx].template.cd_skill.clone() {
                    self.log.push(format!("{} uses {}", actor_side_chars[char_idx].name(), cd_skill.name));
                    actor_side_chars[char_idx].cd_remaining = cd_skill.cooldown_max;
                    self.apply_skill_effects(&cd_skill.effects, char_idx, is_attacker, rng);
                }
            }
            ActionType::EnergySkill => {
                if let Some(es) = actor_side_chars[char_idx].template.energy_skill.clone() {
                    self.log.push(format!("{} uses {}", actor_side_chars[char_idx].name(), es.name));
                    actor_side_chars[char_idx].energy = 0;
                    self.apply_skill_effects(&es.effects, char_idx, is_attacker, rng);
                }
            }
            ActionType::Ultimate => {
                if let Some(ult) = actor_side_chars[char_idx].template.ultimate.clone() {
                    self.log.push(format!("{} uses ULTIMATE {}", actor_side_chars[char_idx].name(), ult.name));
                    actor_side_chars[char_idx].ult_used = true;
                    self.apply_skill_effects(&ult.effects, char_idx, is_attacker, rng);
                }
            }
            ActionType::Wait => {
                self.log.push(format!("{} waits", actor_side_chars[char_idx].name()));
            }
        }
        let _ = actor_stats; // used in decision
    }

    fn apply_skill_effects(&mut self, effects: &[SkillEffect], actor_idx: usize, actor_is_attacker: bool, rng: &mut SeededRng) {
        // Pre-compute actor info
        let actor_star;
        let actor_atk;
        let actor_is_lord;
        {
            let actor_chars = if actor_is_attacker { &self.attacker_chars } else { &self.defender_chars };
            let actor = &actor_chars[actor_idx];
            actor_star = actor.star_level;
            actor_atk = actor.effective_atk() as i32;
            actor_is_lord = actor.is_lord;
        }

        for effect in effects {
            match effect {
                SkillEffect::Damage { multiplier, target } => {
                    let target_idx = self.select_target(*target, actor_is_attacker, rng);
                    if let Some((idx, on_attacker_side)) = target_idx {
                        let dmg = self.calc_damage(actor_atk, actor_star, idx, on_attacker_side, actor_is_attacker, *multiplier, rng, actor_is_attacker && self.turn == 0);
                        self.apply_damage(idx, on_attacker_side, dmg);
                    }
                }
                SkillEffect::AoEDamage { multiplier } => {
                    // Damage all enemies
                    let enemy_is_attacker_side = !actor_is_attacker;
                    let enemy_count = if enemy_is_attacker_side { self.attacker_chars.len() } else { self.defender_chars.len() };
                    for i in 0..enemy_count {
                        let dmg = self.calc_damage(actor_atk, actor_star, i, enemy_is_attacker_side, actor_is_attacker, *multiplier * 0.7, rng, actor_is_attacker && self.turn == 0);
                        self.apply_damage(i, enemy_is_attacker_side, dmg);
                    }
                }
                SkillEffect::Heal { multiplier, target } => {
                    let target_idx = self.select_target(*target, actor_is_attacker, rng);
                    if let Some((idx, on_actor_side)) = target_idx {
                        let heal_amt = (actor_atk as f64 * *multiplier) as i32;
                        if on_actor_side == actor_is_attacker {
                            self.attacker_chars[idx].heal(heal_amt);
                            self.log.push(format!("  -> {} heals {} for {}",
                                self.actor_name(actor_idx, actor_is_attacker),
                                self.attacker_chars[idx].name(), heal_amt));
                        } else {
                            self.defender_chars[idx].heal(heal_amt);
                            self.log.push(format!("  -> {} heals {} for {}",
                                self.actor_name(actor_idx, actor_is_attacker),
                                self.defender_chars[idx].name(), heal_amt));
                        }
                    }
                }
                SkillEffect::BuffAtk { percent, duration, target } => {
                    self.apply_buff(*target, actor_idx, actor_is_attacker, "ATK Boost".into(), *percent as f64, 0.0, 0.0, 0, false, *duration, rng);
                }
                SkillEffect::BuffDef { percent, duration, target } => {
                    self.apply_buff(*target, actor_idx, actor_is_attacker, "DEF Boost".into(), 0.0, *percent as f64, 0.0, 0, false, *duration, rng);
                }
                SkillEffect::BuffSpd { percent, duration, target } => {
                    self.apply_buff(*target, actor_idx, actor_is_attacker, "SPD Boost".into(), 0.0, 0.0, *percent as f64, 0, false, *duration, rng);
                }
                SkillEffect::DebuffAtk { percent, duration, target } => {
                    self.apply_buff(*target, actor_idx, actor_is_attacker, "ATK Down".into(), -(*percent as f64), 0.0, 0.0, 0, true, *duration, rng);
                }
                SkillEffect::DebuffDef { percent, duration, target } => {
                    self.apply_buff(*target, actor_idx, actor_is_attacker, "DEF Down".into(), 0.0, -(*percent as f64), 0.0, 0, true, *duration, rng);
                }
                SkillEffect::DebuffSpd { percent, duration, target } => {
                    self.apply_buff(*target, actor_idx, actor_is_attacker, "SPD Down".into(), 0.0, 0.0, -(*percent as f64), 0, true, *duration, rng);
                }
                SkillEffect::Shield { amount, target } => {
                    let target_idx = self.select_target(*target, actor_is_attacker, rng);
                    if let Some((idx, on_actor_side)) = target_idx {
                        if on_actor_side == actor_is_attacker {
                            self.attacker_chars[idx].shield += *amount;
                        } else {
                            self.defender_chars[idx].shield += *amount;
                        }
                    }
                }
                SkillEffect::Cleanse { .. } => {
                    if actor_is_attacker {
                        for c in &mut self.attacker_chars {
                            c.buffs.retain(|b| !b.is_debuff);
                        }
                    } else {
                        for c in &mut self.defender_chars {
                            c.buffs.retain(|b| !b.is_debuff);
                        }
                    }
                }
                SkillEffect::Revive { hp_percent, target } => {
                    let target_idx = self.select_target(*target, actor_is_attacker, rng);
                    if let Some((idx, on_actor_side)) = target_idx {
                        let aname = self.actor_name(actor_idx, actor_is_attacker).to_string();
                        let chars = if on_actor_side == actor_is_attacker { &mut self.attacker_chars } else { &mut self.defender_chars };
                        if chars[idx].is_dead {
                            chars[idx].is_dead = false;
                            chars[idx].hp = (chars[idx].max_hp as f64 * *hp_percent) as i32;
                            self.log.push(format!("  -> {} revives {}!", aname, chars[idx].name()));
                        }
                    }
                }
                SkillEffect::EnergyGain { amount, .. } => {
                    let chars = if actor_is_attacker { &mut self.attacker_chars } else { &mut self.defender_chars };
                    chars[actor_idx].energy = (chars[actor_idx].energy + *amount).min(100);
                }
                SkillEffect::SpeedBarBoost { percent, .. } => {
                    let chars = if actor_is_attacker { &mut self.attacker_chars } else { &mut self.defender_chars };
                    chars[actor_idx].spd = (chars[actor_idx].spd as f64 * (1.0 + *percent)) as i32;
                }
                SkillEffect::Dot { damage, duration, target } => {
                    self.apply_buff(*target, actor_idx, actor_is_attacker, "DoT".into(), 0.0, 0.0, 0.0, *damage, true, *duration, rng);
                }
                SkillEffect::Stun { duration, target } => {
                    self.apply_buff(*target, actor_idx, actor_is_attacker, "Stun".into(), 0.0, 0.0, 0.0, 0, true, *duration, rng);
                }
                _ => {}
            }
        }
        let _ = actor_is_lord;
    }

    fn apply_buff(&mut self, target: SkillTarget, _actor_idx: usize, actor_is_attacker: bool,
                  name: String, atk_mod: f64, def_mod: f64, spd_mod: f64, dot: i32,
                  is_debuff: bool, duration: i32, rng: &mut SeededRng) {
        let buff = Buff {
            name,
            duration,
            atk_mod,
            def_mod,
            spd_mod,
            dot,
            is_debuff,
        };
        // Determine target side (always allies for buffs)
        let apply_to_actor_side = match target {
            SkillTarget::EnemyAll | SkillTarget::EnemyFront | SkillTarget::EnemyMid | SkillTarget::EnemyBack
            | SkillTarget::EnemyLowestHp | SkillTarget::EnemyHighestAtk | SkillTarget::AllEnemies => !actor_is_attacker,
            _ => actor_is_attacker,
        };
        let targets = self.select_targets_for_buff(target, actor_is_attacker, apply_to_actor_side, rng);
        for (idx, on_attacker) in targets {
            if on_attacker {
                self.attacker_chars[idx].buffs.push(buff.clone());
            } else {
                self.defender_chars[idx].buffs.push(buff.clone());
            }
        }
    }

    fn select_targets_for_buff(&self, target: SkillTarget, actor_is_attacker: bool, on_actor_side: bool, _rng: &mut SeededRng) -> Vec<(usize, bool)> {
        let chars = if on_actor_side { &self.attacker_chars } else { &self.defender_chars };
        let mut results = Vec::new();
        match target {
            SkillTarget::Self_ => {
                // Actor is on actor_is_attacker side
                results.push((0, actor_is_attacker)); // will be corrected by caller; approximate
            }
            SkillTarget::AllAllies | SkillTarget::AllyAll => {
                for i in 0..chars.len() {
                    if !chars[i].is_dead { results.push((i, on_actor_side)); }
                }
            }
            SkillTarget::AllEnemies => {
                for i in 0..chars.len() {
                    if !chars[i].is_dead { results.push((i, on_actor_side)); }
                }
            }
            SkillTarget::AllyLowestHp | SkillTarget::EnemyLowestHp => {
                if let Some((idx, _)) = lowest_hp_idx(chars) {
                    results.push((idx, on_actor_side));
                }
            }
            _ => {
                // Default to first alive
                for i in 0..chars.len() {
                    if !chars[i].is_dead { results.push((i, on_actor_side)); break; }
                }
            }
        }
        results
    }

    fn select_target(&self, target: SkillTarget, actor_is_attacker: bool, rng: &mut SeededRng) -> Option<(usize, bool)> {
        let (enemies, enemy_on_attacker_side) = match target {
            SkillTarget::Self_ => {
                // Actor is at index 0 in their own list... but we don't know idx here. Handle at caller.
                return Some((0, actor_is_attacker));
            }
            SkillTarget::EnemyFront | SkillTarget::EnemyMid | SkillTarget::EnemyBack
            | SkillTarget::EnemyAll | SkillTarget::EnemyLowestHp | SkillTarget::EnemyHighestAtk
            | SkillTarget::AllEnemies | SkillTarget::PositionTarget(_) => {
                // Target is on the enemy side relative to actor
                if actor_is_attacker { (&self.defender_chars, false) }
                else { (&self.attacker_chars, true) }
            }
            _ => {
                // Ally target
                if actor_is_attacker { (&self.attacker_chars, true) }
                else { (&self.defender_chars, false) }
            }
        };

        let filtered: Vec<usize> = enemies.iter().enumerate()
            .filter(|(_, c)| !c.is_dead)
            .map(|(i, _)| i)
            .collect();
        if filtered.is_empty() { return None; }

        let chosen = match target {
            SkillTarget::EnemyFront => {
                let f: Vec<usize> = filtered.clone();
                f.iter().copied().find(|&i| enemies[i].position == Position::Frontline)
                    .or_else(|| f.iter().copied().find(|&i| enemies[i].position == Position::Midline))
                    .or_else(|| filtered.first().copied())
            }
            SkillTarget::EnemyMid => {
                let f: Vec<usize> = filtered.clone();
                f.iter().copied().find(|&i| enemies[i].position == Position::Midline)
                    .or_else(|| f.iter().copied().find(|&i| enemies[i].position == Position::Backline))
                    .or_else(|| filtered.first().copied())
            }
            SkillTarget::EnemyBack => {
                let f: Vec<usize> = filtered.clone();
                f.iter().rev().copied().find(|&i| enemies[i].position == Position::Backline)
                    .or_else(|| f.iter().rev().copied().find(|&i| enemies[i].position == Position::Midline))
                    .or_else(|| filtered.last().copied())
            }
            SkillTarget::EnemyLowestHp => {
                let live: Vec<(usize, &Character)> = enemies.iter().enumerate()
                    .filter(|(_, c)| !c.is_dead).collect();
                live.iter().min_by_key(|(_, c)| c.hp).map(|&(i, _)| i)
            }
            SkillTarget::EnemyHighestAtk => {
                let live: Vec<(usize, &Character)> = enemies.iter().enumerate()
                    .filter(|(_, c)| !c.is_dead).collect();
                live.iter().max_by_key(|(_, c)| c.atk).map(|&(i, _)| i)
            }
            SkillTarget::EnemyAll | SkillTarget::AllEnemies => {
                filtered.first().copied()
            }
            SkillTarget::AllyLowestHp => {
                let live: Vec<(usize, &Character)> = enemies.iter().enumerate()
                    .filter(|(_, c)| !c.is_dead).collect();
                live.iter().min_by_key(|(_, c)| c.hp).map(|&(i, _)| i)
            }
            _ => {
                let _ = rng;
                filtered.first().copied()
            }
        };
        chosen.map(|i| (i, enemy_on_attacker_side))
    }

    fn calc_damage(&self, attacker_atk: i32, attacker_star: u32, target_idx: usize, target_on_attacker_side: bool,
                   attacker_is_attacker: bool, multiplier: f64, rng: &mut SeededRng, first_strike_active: bool) -> i32 {
        let target = if target_on_attacker_side { &self.attacker_chars[target_idx] } else { &self.defender_chars[target_idx] };
        let attacker_pos = self.get_actor_position(attacker_is_attacker);
        let target_pos = target.position;

        let mut base = (attacker_atk as f64 - target.effective_def()).max(1.0);
        base *= multiplier;

        // Position modifier
        let pos_mod = position_modifier(attacker_pos, target_pos);
        base *= pos_mod;

        // First strike modifier: if attacker is on the attacking side (first phase)
        let fs_mod = if first_strike_active && attacker_is_attacker {
            1.2
        } else if first_strike_active && !attacker_is_attacker {
            0.8
        } else {
            1.0
        };
        base *= fs_mod;

        // Star modifier
        let target_star = target.star_level;
        let star_diff = attacker_star as i32 - target_star as i32;
        let star_mod = 1.1f64.powi(star_diff);
        base *= star_mod;

        // Critical hit 5% chance × 2.0
        let crit = rng.gen_bool(0.05);
        if crit { base *= 2.0; }

        base as i32
    }

    fn get_actor_position(&self, actor_is_attacker: bool) -> Position {
        let chars = if actor_is_attacker { &self.attacker_chars } else { &self.defender_chars };
        chars.first().map(|c| c.position).unwrap_or(Position::Frontline)
    }

    fn apply_damage(&mut self, target_idx: usize, target_on_attacker_side: bool, dmg: i32) {
        let actual = if target_on_attacker_side {
            self.attacker_chars[target_idx].take_damage(dmg)
        } else {
            self.defender_chars[target_idx].take_damage(dmg)
        };
        let target_name = if target_on_attacker_side {
            self.attacker_chars[target_idx].name().to_string()
        } else {
            self.defender_chars[target_idx].name().to_string()
        };
        // Track damage dealt for scoring (only count player-attacker damage)
        if (target_on_attacker_side && self.attacker_side == Side::Enemy) ||
           (!target_on_attacker_side && self.attacker_side == Side::Player) {
            self.total_damage_dealt_by_player += actual as i64;
        }
        self.log.push(format!("  -> {} takes {} damage", target_name, actual));
        if target_on_attacker_side {
            if self.attacker_chars[target_idx].is_dead {
                self.log.push(format!("  -> {} falls!", target_name));
            }
        } else {
            if self.defender_chars[target_idx].is_dead {
                self.log.push(format!("  -> {} falls!", target_name));
            }
        }
    }

    fn actor_name(&self, _idx: usize, is_attacker: bool) -> &str {
        let chars = if is_attacker { &self.attacker_chars } else { &self.defender_chars };
        chars.first().map(|c| c.name().as_ref()).unwrap_or("?")
    }

    fn run_passive_phase(&mut self, rng: &mut SeededRng, _is_player_attacker: bool) {
        // Check passive skills by SPD order (simplified: any triggered effects)
        // Process DoT ticks
        for side in 0..2 {
            let chars = if side == 0 { &mut self.attacker_chars } else { &mut self.defender_chars };
            for c in chars.iter_mut() {
                if c.is_dead { continue; }
                let mut total_dot = 0;
                c.buffs.retain_mut(|b| {
                    if b.dot > 0 {
                        total_dot += b.dot;
                    }
                    b.duration -= 1;
                    b.duration > 0
                });
                if total_dot > 0 {
                    c.take_damage(total_dot);
                }
            }
        }

        // Simple passive reactions: counter-attack when attacked, low-HP guard
        // For simplicity, run a "counter" round where characters with counter chance hit back
        // (Decision engine already picks appropriate skills; passives here handle triggered effects.)

        // Tick down buffs duration for next turn
        self.check_battle_end();
        let _ = rng;
    }

    fn check_battle_end(&mut self) {
        let att_alive = self.attacker_chars.iter().any(|c| !c.is_dead);
        let def_alive = self.defender_chars.iter().any(|c| !c.is_dead);

        if !att_alive {
            self.result = Some(SubBattleResult::DefenderWins);
        } else if !def_alive {
            self.result = Some(SubBattleResult::AttackerWins);
        } else {
            // Check point depletion
            let att_points: u32 = self.attacker_chars.iter().map(|c| c.active_points + c.passive_points).sum();
            let def_points: u32 = self.defender_chars.iter().map(|c| c.active_points + c.passive_points).sum();
            if att_points == 0 && def_points == 0 {
                let att_hp = total_hp_percent(&self.attacker_chars);
                let def_hp = total_hp_percent(&self.defender_chars);
                self.result = Some(if att_hp > def_hp {
                    SubBattleResult::AttackerWins
                } else if def_hp > att_hp {
                    SubBattleResult::DefenderWins
                } else {
                    SubBattleResult::Draw
                });
            }
        }
    }

    /// Apply sub-battle results back to the teams, returning damage dealt by player.
    pub fn apply_results(&self, attacker: &mut Team, defender: &mut Team) -> i64 {
        match &self.result {
            Some(SubBattleResult::AttackerWins) => {
                // Defender team wiped
                for m in &mut defender.members {
                    m.hp = 0;
                    m.is_dead = true;
                }
                // Attacker keeps current HP
                for (i, m) in self.attacker_chars.iter().enumerate() {
                    if i < attacker.members.len() {
                        attacker.members[i].hp = m.hp;
                        attacker.members[i].is_dead = m.is_dead;
                        attacker.members[i].buffs = m.buffs.clone();
                        attacker.members[i].energy = m.energy;
                        attacker.members[i].cd_remaining = m.cd_remaining;
                        attacker.members[i].ult_used = m.ult_used;
                    }
                }
            }
            Some(SubBattleResult::DefenderWins) => {
                for m in &mut attacker.members {
                    m.hp = 0;
                    m.is_dead = true;
                }
                for (i, m) in self.defender_chars.iter().enumerate() {
                    if i < defender.members.len() {
                        defender.members[i].hp = m.hp;
                        defender.members[i].is_dead = m.is_dead;
                        defender.members[i].buffs = m.buffs.clone();
                        defender.members[i].energy = m.energy;
                        defender.members[i].cd_remaining = m.cd_remaining;
                        defender.members[i].ult_used = m.ult_used;
                    }
                }
            }
            Some(SubBattleResult::Draw) => {
                // Both keep remaining HP
                for (i, m) in self.attacker_chars.iter().enumerate() {
                    if i < attacker.members.len() {
                        attacker.members[i].hp = m.hp;
                        attacker.members[i].is_dead = m.is_dead;
                    }
                }
                for (i, m) in self.defender_chars.iter().enumerate() {
                    if i < defender.members.len() {
                        defender.members[i].hp = m.hp;
                        defender.members[i].is_dead = m.is_dead;
                    }
                }
            }
            Some(SubBattleResult::Retreat) | None => {
                // Retreat: attacker HP halved, back to pre-battle position
                for m in &mut attacker.members {
                    m.hp = (m.hp / 2).max(1);
                }
            }
        }
        self.total_damage_dealt_by_player
    }
}

// --- Helper functions ---

fn sort_by_position(chars: &mut [Character]) {
    chars.sort_by_key(|c| (c.position as u8, c.name().to_string()));
}

fn order_by_spd_desc(chars: &[Character]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..chars.len()).collect();
    order.sort_by(|&a, &b| chars[b].effective_spd().partial_cmp(&chars[a].effective_spd()).unwrap_or(std::cmp::Ordering::Equal));
    order
}

fn total_hp_percent(chars: &[Character]) -> f64 {
    let mut total_cur = 0i32;
    let mut total_max = 0i32;
    for c in chars {
        if !c.is_dead {
            total_cur += c.hp;
            total_max += c.max_hp;
        }
    }
    if total_max == 0 { 0.0 } else { total_cur as f64 / total_max as f64 }
}

fn lowest_hp_idx(chars: &[Character]) -> Option<(usize, &Character)> {
    chars.iter().enumerate()
        .filter(|(_, c)| !c.is_dead)
        .min_by_key(|(_, c)| c.hp)
}

fn position_modifier(attacker: Position, target: Position) -> f64 {
    use Position::*;
    match (attacker, target) {
        (Frontline, Frontline) => 1.0,
        (Frontline, Midline) => 0.7,
        (Frontline, Backline) => 0.4,
        (Midline, Frontline) => 1.1,
        (Midline, Midline) => 1.0,
        (Midline, Backline) => 0.7,
        (Backline, Frontline) => 1.2,
        (Backline, Midline) => 1.0,
        (Backline, Backline) => 1.0,
    }
}

#[derive(Debug, Clone, Copy)]
enum ActionType {
    NormalAttack,
    SpSkill,
    CdSkill,
    EnergySkill,
    Ultimate,
    Wait,
}

/// Decision engine — evaluates up to ~100 conditions to choose optimal action.
fn decide_action(actor: &Character, enemies: &[Character], allies: &[Character], current_sp: i32) -> ((), ActionType) {
    let enemy_alive: Vec<&Character> = enemies.iter().filter(|c| !c.is_dead).collect();
    let ally_alive: Vec<&Character> = allies.iter().filter(|c| !c.is_dead).collect();
    let lowest_enemy_hp_pct = enemy_alive.iter().map(|c| c.hp as f64 / c.max_hp as f64).fold(1.0, f64::min);
    let lowest_ally_hp_pct = ally_alive.iter().map(|c| c.hp as f64 / c.max_hp as f64).fold(1.0, f64::min);
    let multiple_enemies = enemy_alive.len() >= 2;
    let self_hp_pct = actor.hp as f64 / actor.max_hp as f64;
    let enemy_has_buff = enemy_alive.iter().any(|c| c.buffs.iter().any(|b| !b.is_debuff && b.atk_mod > 0.0));
    let self_has_debuff = actor.buffs.iter().any(|b| b.is_debuff);
    let can_kill_enemy = enemy_alive.iter().any(|e| e.hp <= actor.atk / 2); // approx

    // Priority: Ultimate if ready and not used
    if actor.template.ultimate.is_some() && !actor.ult_used && (can_kill_enemy || lowest_enemy_hp_pct < 0.3 || self_hp_pct < 0.3) {
        return ((), ActionType::Ultimate);
    }

    // Energy skill if energy full
    if actor.energy >= 100 && actor.template.energy_skill.is_some() {
        if multiple_enemies || lowest_enemy_hp_pct < 0.5 || lowest_ally_hp_pct < 0.4 {
            return ((), ActionType::EnergySkill);
        }
    }

    // CD skill if available
    if actor.cd_remaining <= 0 && actor.template.cd_skill.is_some() {
        if multiple_enemies || (lowest_ally_hp_pct < 0.6) || (lowest_enemy_hp_pct < 0.5) {
            return ((), ActionType::CdSkill);
        }
    }

    // SP skill if SP available and situation warrants
    if current_sp > 0 && actor.template.sp_skill.is_some() {
        let use_sp = multiple_enemies
            || lowest_ally_hp_pct < 0.5
            || enemy_has_buff
            || (lowest_enemy_hp_pct < 0.4 && actor.atk > 100)
            || self_has_debuff;
        if use_sp {
            return ((), ActionType::SpSkill);
        }
    }

    // Default: normal attack
    ((), ActionType::NormalAttack)
}

// Helper retain_mut for older Rust compat
trait RetainMutExt<T> {
    fn retain_mut<F: FnMut(&mut T) -> bool>(&mut self, f: F);
}
impl<T> RetainMutExt<T> for Vec<T> {
    fn retain_mut<F: FnMut(&mut T) -> bool>(&mut self, mut f: F) {
        let mut i = 0;
        while i < self.len() {
            if f(&mut self[i]) {
                i += 1;
            } else {
                self.remove(i);
            }
        }
    }
}
