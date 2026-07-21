use serde::{Serialize, Deserialize};
use crate::rng::SeededRng;
use crate::types::*;
use crate::lord::Lord;
use crate::team::Team;
use crate::map::{Overworld, NodeType};
use crate::combat::TacticalMap;
use crate::sub_battle::SubBattle;
use crate::gacha::GachaPool;
use crate::events::EventDef;
use crate::ui::ShopItem;
use crate::save::SaveSlot;
use crate::characters::{Character, CharacterTemplate, template_by_id};

/// Main game state container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub seed: u64,
    #[serde(skip, default = "crate::rng::SeededRng::from_entropy")]
    pub rng: SeededRng,
    pub phase: GamePhase,
    pub turn_number: i32,
    pub score_damage_dealt: i64,
    pub lord: Lord,
    pub overworld: Overworld,
    pub tactical: Option<TacticalMap>,
    pub sub_battle: Option<SubBattle>,
    pub pool: GachaPool,
    pub selected_team: Option<u32>,
    pub pending_event: Option<EventDef>,
    pub shop_items: Vec<ShopItem>,
    pub global_xp: i64,
    pub global_unlocks: Vec<String>,
    pub current_floor_teams_used: u32,
    pub sub_attacker_id: Option<u32>,
    pub sub_defender_id: Option<u32>,
    #[serde(skip, default)]
    pub sub_attacker_player: Option<bool>,
    #[serde(skip, default)]
    pub sub_defender_player: Option<bool>,
    pub waiting_for_continue: bool,
    pub player_teams_composition: Vec<Vec<u32>>,
}

impl GameState {
    pub fn new(seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or_else(|| {
            use rand::Rng;
            rand::thread_rng().gen::<u64>()
        });
        let mut rng = SeededRng::new(seed);
        let lord = Lord::new();
        let overworld = Overworld::new(4, &mut rng);
        let pool = GachaPool::new();

        let mut game = Self {
            seed,
            rng,
            phase: GamePhase::Map,
            turn_number: 0,
            score_damage_dealt: 0,
            lord,
            overworld,
            tactical: None,
            sub_battle: None,
            pool,
            selected_team: None,
            pending_event: None,
            shop_items: Vec::new(),
            global_xp: 0,
            global_unlocks: Vec::new(),
            current_floor_teams_used: 0,
            sub_attacker_id: None,
            sub_defender_id: None,
            sub_attacker_player: None,
            sub_defender_player: None,
            waiting_for_continue: false,
            player_teams_composition: Vec::new(),
        };

        // Initial gacha pulls to give player starting roster
        let starting = game.pool.pull_n(50, &mut game.rng);
        // Give starter equipment
        let equips = GachaPool::roll_equipment(3, &mut game.rng);
        for e in equips {
            game.pool.owned_equipment.push(e);
        }
        let _ = starting;

        game
    }

    /// Set up initial player teams for floor start.
    pub fn setup_player_teams(&mut self) {
        let mut tactical = TacticalMap::new(&mut self.rng);

        // Lord team at starting position
        let mut lord_team = Team::new(0, "Lord's Vanguard", Side::Player, GridPos::new(5, 20));
        lord_team.has_lord = true;
        lord_team.move_range = 10;
        let mut lord_char = self.lord.character.clone();
        lord_char.hp = lord_char.max_hp;
        lord_team.add_member(lord_char).ok();

        // Add 2nd character to lord team from owned characters
        if let Some(tmpl) = self.pool.owned_characters.first().cloned() {
            let mut ch = Character::from_template(tmpl);
            ch.position = Position::Midline;
            lord_team.add_member(ch).ok();
        }
        let lid = tactical.add_team(lord_team);
        let _ = lid;

        // Additional teams up to lord's max
        let max_teams = self.lord.max_teams();
        let available_chars: Vec<CharacterTemplate> = self.pool.owned_characters.iter()
            .skip(1)
            .take((max_teams as usize) * 2)
            .cloned()
            .collect();

        let mut char_idx = 0;
        for t in 1..max_teams {
            let team_names = ["Iron Bulls", "Silver Wolves", "Storm Hawks", "Bronze Lions",
                              "Obsidian Guard", "Crimson Blades", "Emerald Archers", "Golden Spears",
                              "Sapphire Mages"];
            let name = team_names.get(t as usize - 1).copied().unwrap_or("Reserve Team");
            let mut team = Team::new(t, name, Side::Player, GridPos::new(5, 20 + (t as i32)*4));
            team.move_range = 10;

            // Add 2 characters per team initially
            for _ in 0..2 {
                if char_idx < available_chars.len() {
                    let tmpl = &available_chars[char_idx];
                    let mut ch = Character::from_template(tmpl.clone());
                    ch.position = if char_idx % 3 == 0 { Position::Frontline }
                                  else if char_idx % 3 == 1 { Position::Midline }
                                  else { Position::Backline };
                    team.add_member(ch).ok();
                    char_idx += 1;
                }
            }
            tactical.add_team(team);
        }

        // Spawn enemy teams
        self.spawn_enemies_for_floor(&mut tactical);

        self.tactical = Some(tactical);
        self.phase = GamePhase::CombatTactical;
    }

    fn spawn_enemies_for_floor(&mut self, tactical: &mut TacticalMap) {
        let floor = self.overworld.current_floor;
        let num_enemies = 3 + floor as i32;
        let difficulty = 1.0 + (floor as f64) * 0.3;

        for i in 0..num_enemies {
            let enemy_char_id = self.rng.gen_range(0, 999);
            let tmpl = template_by_id(enemy_char_id as u32);
            let is_lord = i == num_enemies - 1;
            let team_name = if is_lord { "Enemy Lord" } else { "Enemy Squad" };
            // Spawn enemies closer: x=28-35 instead of 70-90 (grid is 40 wide)
            let mut team = Team::new(100 + i as u32, team_name, Side::Enemy,
                GridPos::new(22 + self.rng.gen_range(0, 6), 10 + self.rng.gen_range(0, 20)));
            if is_lord {
                team.has_lord = true;
                team.is_boss_team = false;
            }

            let mut ch = Character::from_template(tmpl);
            ch.atk = (ch.atk as f64 * difficulty) as i32;
            ch.max_hp = (ch.max_hp as f64 * difficulty) as i32;
            ch.hp = ch.max_hp;
            if is_lord {
                ch.is_lord = true;
                ch.max_hp = (ch.max_hp as f64 * 2.0) as i32;
                ch.hp = ch.max_hp;
            }
            team.add_member(ch).ok();

            // Add 1-2 more enemies per team
            for _ in 0..self.rng.gen_range(0, 2) {
                let eid = self.rng.gen_range(0, 999);
                let etmpl = template_by_id(eid as u32);
                let mut ech = Character::from_template(etmpl);
                ech.atk = (ech.atk as f64 * difficulty) as i32;
                ech.max_hp = (ech.max_hp as f64 * difficulty) as i32;
                ech.hp = ech.max_hp;
                team.add_member(ech).ok();
            }

            tactical.add_team(team);
        }
    }

    pub fn spawn_boss(&mut self) {
        if let Some(ref mut tac) = self.tactical {
            let floor = self.overworld.current_floor;
            let difficulty = 1.5 + (floor as f64) * 0.5;
            let tmpl = template_by_id(500 + floor);
            let mut boss_team = Team::new(200 + floor, "Floor Boss", Side::Enemy, GridPos::new(28, 20));
            boss_team.has_lord = true;
            boss_team.is_boss_team = true;

            let mut boss = Character::from_template(tmpl);
            boss.atk = (boss.atk as f64 * difficulty * 1.5) as i32;
            boss.max_hp = (boss.max_hp as f64 * difficulty * 3.0) as i32;
            boss.hp = boss.max_hp;
            boss.star_level = 2 + floor;
            boss.is_lord = true;
            boss_team.add_member(boss).ok();

            // Minions
            for _ in 0..3 {
                let eid = self.rng.gen_range(0, 999);
                let etmpl = template_by_id(eid as u32);
                let mut ech = Character::from_template(etmpl);
                ech.atk = (ech.atk as f64 * difficulty) as i32;
                ech.max_hp = (ech.max_hp as f64 * difficulty) as i32;
                ech.hp = ech.max_hp;
                boss_team.add_member(ech).ok();
            }

            tac.add_team(boss_team);
            self.phase = GamePhase::BossIntro;
        }
    }

    /// Advance to the next acting team on the speed bar.
    pub fn advance_turn(&mut self) -> String {
        let mut output = String::new();
        let mut enemy_attack: Option<(u32, u32)> = None;
        let mut player_attack: Option<(u32, u32)> = None;
        let mut next_team_name: Option<String> = None;
        let mut next_team_id: Option<u32> = None;

        if let Some(ref mut tac) = self.tactical {
            if let Some(next_id) = tac.advance_speed_bar() {
                let is_enemy = tac.get_team(next_id).map(|t| t.side == Side::Enemy).unwrap_or(false);
                if is_enemy {
                    if let Some(action) = tac.enemy_ai_step(&mut self.rng) {
                        output.push_str(&format!("Enemy action: {}\n", action));
                    }
                    if let Some(target) = tac.find_adjacent_enemy(next_id) {
                        enemy_attack = Some((next_id, target));
                    }
                } else {
                    // Player team: auto-move toward nearest enemy
                    if let Some(action) = tac.player_ai_step(&mut self.rng) {
                        output.push_str(&format!("{}\n", action));
                    }
                    if let Some(target) = tac.find_adjacent_enemy(next_id) {
                        player_attack = Some((next_id, target));
                    }
                    next_team_name = tac.get_team(next_id).map(|t| t.name.clone());
                }
                next_team_id = Some(next_id);
            }
            tac.remove_dead_teams();
        }

        // Now drop the borrow and trigger sub-battle if needed
        if let Some((aid, did)) = enemy_attack {
            output.push_str(&format!("Enemy team {} attacks team {}!\n", aid, did));
            self.initiate_sub_battle(aid, did);
        }
        if let Some((aid, did)) = player_attack {
            output.push_str(&format!("Team {} attacks enemy team {}!\n", aid, did));
            self.initiate_sub_battle(aid, did);
        }

        if let Some(name) = next_team_name {
            output.push_str(&format!("Team '{}' is now acting.\n", name));
        }
        if let Some(nid) = next_team_id {
            self.selected_team = Some(nid);
        }

        // Check victory
        if let Some(ref mut tac) = self.tactical {
            match tac.check_victory() {
                Some(Side::Player) => {
                    self.handle_victory();
                    output.push_str("\n★ VICTORY! ★\n");
                }
                Some(Side::Enemy) => {
                    self.phase = GamePhase::Defeat;
                    output.push_str("\n☠ DEFEAT — Your Lord has fallen.\n");
                }
                Some(Side::Neutral) => {}
                None => {}
            }
        }
        output
    }

    pub fn initiate_sub_battle(&mut self, attacker_id: u32, defender_id: u32) {
        if let Some(ref mut tac) = self.tactical {
            let attacker = tac.get_team(attacker_id).cloned();
            let defender = tac.get_team(defender_id).cloned();
            if let (Some(att), Some(def)) = (attacker, defender) {
                let starting_sp = tac.skill_points;
                // Relic bonuses
                let extra_active = self.pool.owned_relics.iter()
                    .filter_map(|r| match &r.effect {
                        crate::relics::RelicEffect::ExtraActivePoints(n) => Some(*n),
                        _ => None,
                    }).sum();
                let energy_start = self.pool.owned_relics.iter()
                    .filter_map(|r| match &r.effect {
                        crate::relics::RelicEffect::EnergyStartPercent(p) => Some(*p),
                        _ => None,
                    }).fold(0.0, f64::max);

                let mut sb = SubBattle::new(&att, &def, starting_sp, extra_active, energy_start);
                let is_player_attacker = att.side == Side::Player;
                sb.run_complete_battle(&mut self.rng, is_player_attacker);
                self.sub_battle = Some(sb);
                self.sub_attacker_id = Some(attacker_id);
                self.sub_defender_id = Some(defender_id);
                self.phase = GamePhase::SubBattle;
                tac.sub_battle_active = true;
            }
        }
    }

    pub fn resolve_sub_battle(&mut self) -> String {
        let mut output = String::new();
        if let (Some(sb), Some(aid), Some(did)) = (self.sub_battle.take(), self.sub_attacker_id, self.sub_defender_id) {
            let total_player_dmg = sb.total_damage_dealt_by_player;
            let sb_result = sb.result.clone();
            output.push_str(&format!("Sub-battle resolved. Damage dealt: {}\n", total_player_dmg));

            // Apply results by cloning teams back in to avoid double mutable borrow
            if let Some(ref mut tac) = self.tactical {
                // Find indices
                let a_idx = tac.teams.iter().position(|t| t.id == aid);
                let d_idx = tac.teams.iter().position(|t| t.id == did);
                if let (Some(ai), Some(di)) = (a_idx, d_idx) {
                    // Determine sides before moving snapshots
                    let attacker_was_player = tac.teams[ai].side == Side::Player;
                    let defender_was_player = tac.teams[di].side == Side::Player;
                    // We need two simultaneous mutable references → use split_at_mut or process sequentially
                    let (att_snapshot, def_snapshot) = {
                        let att_clone = tac.teams[ai].clone();
                        let def_clone = tac.teams[di].clone();
                        (att_clone, def_clone)
                    };
                    // Create a throwaway SubBattle state to apply results on clones
                    let sb_clone = sb;
                    // We need to apply to the actual teams — use indices carefully
                    // Create fresh teams for result application using the existing SubBattle logic
                    // by temporarily constructing two teams matching the snapshots
                    let mut att = att_snapshot;
                    let mut def = def_snapshot;
                    let dmg = sb_clone.apply_results(&mut att, &mut def);
                    self.score_damage_dealt += dmg;
                    self.overworld.score_damage_dealt += dmg;
                    tac.teams[ai] = att;
                    tac.teams[di] = def;

                    // Stash sides for later use
                    self.sub_attacker_player = Some(attacker_was_player);
                    self.sub_defender_player = Some(defender_was_player);
                }
                tac.sub_battle_active = false;
                tac.remove_dead_teams();

                match &sb_result {
                    Some(crate::sub_battle::SubBattleResult::AttackerWins) => {
                        output.push_str("Attacker wins!\n");
                    }
                    Some(crate::sub_battle::SubBattleResult::DefenderWins) => {
                        output.push_str("Defender wins!\n");
                    }
                    Some(crate::sub_battle::SubBattleResult::Draw) => {
                        output.push_str("Draw.\n");
                    }
                    Some(crate::sub_battle::SubBattleResult::Retreat) => {
                        output.push_str("Retreat!\n");
                    }
                    None => {}
                }

                // Post-battle rewards (if player won a fight)
                let attacker_was_player = self.sub_attacker_player.take().unwrap_or(false);
                let defender_was_player = self.sub_defender_player.take().unwrap_or(false);
                let player_won = (sb_result.as_ref().map(|r| matches!(r, crate::sub_battle::SubBattleResult::AttackerWins)) == Some(true)
                    && attacker_was_player)
                    || (sb_result.as_ref().map(|r| matches!(r, crate::sub_battle::SubBattleResult::DefenderWins)) == Some(true)
                    && defender_was_player);

                if player_won {
                    // Battle rewards: 100 gacha pulls, 5 equipment, consumables, possible relic
                    let _pulled = self.pool.pull_n(100, &mut self.rng);
                    let extra_pulls: i32 = self.pool.owned_relics.iter()
                        .filter_map(|r| match &r.effect {
                            crate::relics::RelicEffect::ExtraGachaPulls(n) => Some(*n),
                            _ => None,
                        }).sum();
                    if extra_pulls > 0 {
                        self.pool.pull_n(extra_pulls as u32, &mut self.rng);
                    }

                    let equips = crate::gacha::GachaPool::roll_equipment(5, &mut self.rng);
                    for e in equips { self.pool.owned_equipment.push(e.clone()); }

                    let consumables = crate::gacha::GachaPool::roll_consumables(&mut self.rng);
                    for c in consumables {
                        match c.kind {
                            ConsumableKind::Potion => self.pool.owned_potions.push(c),
                            ConsumableKind::Food => self.pool.owned_food.push(c),
                        }
                    }

                    if let Some(relic) = crate::gacha::GachaPool::roll_relic(&mut self.rng) {
                        if !self.pool.owned_relics.iter().any(|r| r.name == relic.name) {
                            output.push_str(&format!("★ Rare relic found: {}!\n", relic.name));
                            self.pool.owned_relics.push(relic);
                        }
                    }

                    // Gold
                    let gold_bonus: i32 = self.pool.owned_relics.iter()
                        .filter_map(|r| match &r.effect {
                            crate::relics::RelicEffect::GoldPerBattle(n) => Some(*n),
                            _ => None,
                        }).sum();
                    self.overworld.gold += 30 + gold_bonus;
                    output.push_str("Rewards: 100 gacha pulls, 5 equipment, consumables, +30g\n");
                }

                // Check for victory/defeat
                match tac.check_victory() {
                    Some(Side::Player) => { self.handle_victory(); }
                    Some(Side::Enemy) => { self.phase = GamePhase::Defeat; }
                    Some(Side::Neutral) => { self.phase = GamePhase::CombatTactical; }
                    None => { self.phase = GamePhase::CombatTactical; }
                }
            }
            self.sub_attacker_id = None;
            self.sub_defender_id = None;
            self.sub_attacker_player = None;
            self.sub_defender_player = None;
        }
        output
    }

    fn handle_victory(&mut self) {
        // Check if this was a floor boss
        if let Some(ref tac) = self.tactical {
            let boss_defeated = tac.teams.iter()
                .filter(|t| t.side == Side::Enemy)
                .all(|t| !t.is_boss_team || !t.is_alive());
            let boss_present = tac.teams.iter().any(|t| t.is_boss_team && t.side == Side::Enemy);
            if boss_defeated && boss_present {
                // Advance floor
                if self.overworld.advance_floor() {
                    self.phase = GamePhase::Map;
                    self.tactical = None;
                } else {
                    self.phase = GamePhase::Victory;
                }
            } else {
                // Normal combat victory on tactical map → node cleared
                self.phase = GamePhase::Map;
                self.tactical = None;
            }
        }
    }

    pub fn move_to_node(&mut self, node_index: u32) -> Result<String, String> {
        let node_id = NodeId { floor: self.overworld.current_floor, index: node_index };
        let floor = self.overworld.current_floor_map().clone();
        let available = floor.available_nodes();
        if !available.iter().any(|n| n.id == node_id) {
            return Err("That node is not reachable from your current position".into());
        }

        // Find the node type
        let node = available.iter().find(|n| n.id == node_id).unwrap();
        let node_type = node.node_type;

        self.overworld.current_floor_map_mut().visit_node(node_id);

        Ok(match node_type {
            NodeType::Monster => {
                self.setup_player_teams();
                // Give some SP
                if let Some(ref mut t) = self.tactical {
                    let starting_sp: i32 = self.pool.owned_relics.iter()
                        .filter_map(|r| match &r.effect {
                            crate::relics::RelicEffect::StartingSkillPoints(n) => Some(*n),
                            _ => None,
                        }).sum();
                    t.skill_points += starting_sp;
                }
                "Entering combat!".into()
            }
            NodeType::Boss => {
                self.setup_player_teams();
                self.spawn_boss();
                "Boss fight!".into()
            }
            NodeType::Campfire => {
                self.phase = GamePhase::Campfire;
                "You arrive at a campfire.".into()
            }
            NodeType::Shop => {
                self.phase = GamePhase::Shop;
                self.generate_shop_items();
                "You enter the shop.".into()
            }
            NodeType::Event => {
                let evt = crate::events::generate_random_event(&mut self.rng);
                self.pending_event = Some(evt.clone());
                self.phase = GamePhase::Event;
                format!("Event: {}", evt.title)
            }
            NodeType::Start => "Already at start.".into(),
        })
    }

    fn generate_shop_items(&mut self) {
        self.shop_items = Vec::new();
        let discount: f64 = self.pool.owned_relics.iter()
            .filter_map(|r| match &r.effect {
                crate::relics::RelicEffect::ShopDiscount(d) => Some(*d),
                _ => None,
            }).fold(1.0, |a, b| a.min(b));

        let equips = crate::equipment::all_equipment_templates();
        for _ in 0..5 {
            let idx = self.rng.gen_range(0, equips.len() as i32 - 1) as usize;
            let price = ((20 + self.rng.gen_range(0, 30)) as f64 * discount) as i32;
            self.shop_items.push(ShopItem {
                name: equips[idx].name.clone(),
                price,
                kind: crate::ui::ShopItemKind::Equipment(idx),
            });
        }
        let relics = crate::relics::all_relic_templates();
        let ridx = self.rng.gen_range(0, relics.len() as i32 - 1) as usize;
        let rprice = (80 as f64 * discount) as i32;
        self.shop_items.push(ShopItem {
            name: format!("Relic: {}", relics[ridx].name),
            price: rprice,
            kind: crate::ui::ShopItemKind::Relic(ridx),
        });
        self.shop_items.push(ShopItem { name: "Healing Potion".into(), price: (15.0 * discount) as i32, kind: crate::ui::ShopItemKind::Potion });
        self.shop_items.push(ShopItem { name: "Travel Bread".into(), price: (25.0 * discount) as i32, kind: crate::ui::ShopItemKind::Food });
    }

    pub fn to_save_slot(&self, name: &str) -> SaveSlot {
        SaveSlot {
            name: name.to_string(),
            seed: self.seed,
            turn: self.turn_number,
            score: self.score_damage_dealt,
            phase: self.phase,
            overworld: self.overworld.clone(),
            tactical: self.tactical.clone(),
            lord: self.lord.clone(),
            pool: self.pool.clone(),
            global_xp: self.global_xp,
            global_unlocks: self.global_unlocks.clone(),
        }
    }

    pub fn load_from_slot(&mut self, slot: SaveSlot) {
        self.seed = slot.seed;
        self.rng = SeededRng::new(slot.seed);
        self.phase = slot.phase;
        self.turn_number = slot.turn;
        self.score_damage_dealt = slot.score;
        self.lord = slot.lord;
        self.overworld = slot.overworld;
        self.tactical = slot.tactical;
        self.pool = slot.pool;
        self.global_xp = slot.global_xp;
        self.global_unlocks = slot.global_unlocks;
    }

    pub fn current_node_cleared(&self) -> bool {
        // We only mark node cleared when leaving combat; for simplicity any Monster/Boss node with no tactical active is cleared
        self.tactical.is_none()
    }

    /// Render current screen based on phase.
    pub fn render(&self) -> String {
        match self.phase {
            GamePhase::Map => {
                let mut out = String::new();
                out.push_str("=== SAVE LORD — Overworld Map ===\n");
                out.push_str(&format!("Floor: {}/{} | Gold: {} | Lord Lv.{} | Score: {}\n\n",
                    self.overworld.current_floor + 1, self.overworld.total_floors,
                    self.overworld.gold, self.lord.level, self.score_damage_dealt));
                out.push_str(&crate::ui::render_overworld(self.overworld.current_floor_map()));
                out.push_str("\nType 'node <id>' to travel, 'status' for details, 'help' for commands.\n");
                out
            }
            GamePhase::CombatTactical => {
                if let Some(ref tac) = self.tactical {
                    crate::ui::render_combat(tac, self.selected_team)
                } else { "No active combat.\n".into() }
            }
            GamePhase::SubBattle => {
                if let Some(ref sb) = self.sub_battle {
                    let mut out = crate::ui::render_sub_battle(sb);
                    out.push_str("\nType 'continue' to resolve and return to tactical map.\n");
                    out
                } else { "No sub-battle active.\n".into() }
            }
            GamePhase::Campfire => crate::ui::render_campfire(),
            GamePhase::Shop => crate::ui::render_shop(self.overworld.gold, &self.shop_items),
            GamePhase::Event => {
                if let Some(ref evt) = self.pending_event {
                    let mut out = String::new();
                    out.push_str(&format!("=== {} ===\n{}\n\n", evt.title, evt.description));
                    for ch in &evt.choices {
                        out.push_str(&format!("  {}\n     → {}\n\n", ch.label, ch.result_text));
                    }
                    out.push_str("Choose by typing 1, 2, or 3.\n");
                    out
                } else { "No event.\n".into() }
            }
            GamePhase::BossIntro => {
                let mut out = String::new();
                out.push_str("╔══════════════════════════════════╗\n");
                out.push_str("║   ⚠️  BOSS ENCOUNTER! ⚠️         ║\n");
                out.push_str("╚══════════════════════════════════╝\n");
                out.push_str("Type 'continue' to begin the fight!\n");
                out
            }
            GamePhase::Victory => {
                let xp = self.score_damage_dealt / 100;
                let mut out = String::new();
                out.push_str("╔══════════════════════════════════╗\n");
                out.push_str("║        ★ VICTORY! ★              ║\n");
                out.push_str("╠══════════════════════════════════╣\n");
                out.push_str(&format!("║  Final Score: {:<18} ║\n", self.score_damage_dealt));
                out.push_str(&format!("║  XP Earned:   {:<18} ║\n", xp));
                out.push_str("╚══════════════════════════════════╝\n");
                out
            }
            GamePhase::Defeat => {
                let xp = self.score_damage_dealt / 100;
                let mut out = String::new();
                out.push_str("╔══════════════════════════════════╗\n");
                out.push_str("║       ☠ DEFEAT ☠                ║\n");
                out.push_str("╠══════════════════════════════════╣\n");
                out.push_str(&format!("║  Final Score: {:<18} ║\n", self.score_damage_dealt));
                out.push_str(&format!("║  XP Earned:   {:<18} ║\n", xp));
                out.push_str("║  The Lord has fallen.           ║\n");
                out.push_str("║  Use your XP for meta-progression.║\n");
                out.push_str("╚══════════════════════════════════╝\n");
                out
            }
        }
    }
}
