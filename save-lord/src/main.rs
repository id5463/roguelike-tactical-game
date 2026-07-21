use std::io::{self, BufRead, Write};
use save_lord::commands::{Command, parse_command};
use save_lord::game::GameState;
use save_lord::types::*;
use save_lord::save;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut seed_override = None;
    let mut api_mode = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" | "-s" => {
                if i + 1 < args.len() {
                    seed_override = args[i+1].parse::<u64>().ok();
                    i += 1;
                }
            }
            "--api" => api_mode = true,
            _ => {}
        }
        i += 1;
    }

    println!("╔══════════════════════════════════════════════╗");
    println!("║          SAVE LORD — Roguelite RPG          ║");
    println!("║     Save & Load to conquer impossible odds  ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    println!("Type 'help' for command list. Type 'quit' to exit.");
    if seed_override.is_some() {
        println!("Using fixed seed.");
    } else {
        println!("Seed this run is determined randomly.");
    }
    println!();

    let mut game = GameState::new(seed_override);
    println!("Seed: {}\n", game.seed);

    if api_mode {
        // Print API JSON state and exit
        let api = save_lord::api::ApiState::from_game(&game);
        println!("{}", api.to_json());
        return;
    }

    // Initial render
    print!("{}", game.render());
    io::stdout().flush().ok();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().ok();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }

        let cmd = parse_command(&line);
        let response = handle_command(&mut game, cmd);
        if !response.is_empty() {
            println!("{}", response);
        }

        // Auto-save
        save::auto_save(&game.to_save_slot("autosave"));

        // Check for quit
        if matches!(game.phase, GamePhase::Victory | GamePhase::Defeat) {
            // Allow quit or status only
        }

        print!("{}", game.render());
        stdout.flush().ok();
    }
}

fn handle_command(game: &mut GameState, cmd: Command) -> String {
    match cmd {
        Command::Quit => {
            println!("Farewell.");
            std::process::exit(0);
        }
        Command::Help => save_lord::ui::render_help(),
        Command::Status => save_lord::ui::render_status(game),
        Command::Inventory => save_lord::ui::render_inventory(game),
        Command::ListSaves => {
            let saves = save::list_saves();
            if saves.is_empty() {
                "No saves found.\n".into()
            } else {
                format!("Saves:\n{}\n", saves.join("\n"))
            }
        }
        Command::Save(name) => {
            match save::save_game(&name, &game.to_save_slot(&name)) {
                Ok(()) => format!("Game saved to slot '{}'.\n", name),
                Err(e) => format!("Save failed: {}\n", e),
            }
        }
        Command::Load(name) => {
            match save::load_game(&name) {
                Ok(slot) => {
                    game.load_from_slot(slot);
                    format!("Loaded save '{}'.\n", name)
                }
                Err(e) => format!("Load failed: {}\n", e),
            }
        }
        Command::ViewSave(name) => {
            match save::view_save(&name) {
                Ok(s) => s,
                Err(e) => format!("Could not view save: {}\n", e),
            }
        }
        Command::Continue => {
            match game.phase {
                GamePhase::Map => {
                    // Nothing to continue on map
                    "On overworld map. Use 'node <id>' to travel.\n".into()
                }
                GamePhase::CombatTactical => {
                    game.advance_turn()
                }
                GamePhase::SubBattle => {
                    game.resolve_sub_battle()
                }
                GamePhase::Campfire => {
                    "Choose an option (A/B/C/D) at the campfire.\n".into()
                }
                GamePhase::Shop => {
                    "Use 'buy <n>' or 'leave'.\n".into()
                }
                GamePhase::Event => {
                    "Choose event option (1/2/3).\n".into()
                }
                GamePhase::BossIntro => {
                    game.phase = GamePhase::CombatTactical;
                    "Battle begins!\n".into()
                }
                GamePhase::Victory | GamePhase::Defeat => {
                    "Game over. Start a new run or quit.\n".into()
                }
            }
        }
        Command::Select(tid) => {
            game.selected_team = Some(tid);
            if let Some(t) = game.tactical.as_ref().and_then(|t| t.get_team(tid)) {
                save_lord::ui::render_team_details(t)
            } else {
                format!("Team {} not found on tactical map.\n", tid)
            }
        }
        Command::TeamList => {
            let mut out = String::from("Teams:\n");
            if let Some(ref tac) = game.tactical {
                for t in &tac.teams {
                    out.push_str(&format!("  [{}] {} ({:?}) at ({},{}) alive={} members={}\n",
                        t.id, t.name, t.side, t.position.x, t.position.y, t.is_alive(), t.members.len()));
                }
            } else {
                out.push_str("  No tactical battle active.\n");
            }
            out
        }
        Command::ShowMap => {
            save_lord::ui::render_overworld(game.overworld.current_floor_map())
        }
        Command::GotoNode(nid) => {
            if game.phase != GamePhase::Map {
                return "You can only travel to nodes on the overworld map.\n".into();
            }
            match game.move_to_node(nid) {
                Ok(msg) => msg + "\n",
                Err(e) => format!("Error: {}\n", e),
            }
        }
        Command::Move { team_id, direction, distance } => {
            if game.phase != GamePhase::CombatTactical {
                return "Can only move during tactical combat.\n".into();
            }
            if let Some(ref mut tac) = game.tactical {
                let dist = distance.unwrap_or(1);
                match tac.move_team(team_id, direction, dist) {
                    Ok(pos) => format!("Team {} moved to ({},{})\n", team_id, pos.x, pos.y),
                    Err(e) => format!("Move failed: {}\n", e),
                }
            } else { "No tactical map.\n".into() }
        }
        Command::Attack(tid) => {
            if game.phase != GamePhase::CombatTactical {
                return "Can only attack during tactical combat.\n".into();
            }
            if let Some(ref tac) = game.tactical {
                match tac.find_adjacent_enemy(tid) {
                    Some(enemy_id) => {
                        game.initiate_sub_battle(tid, enemy_id);
                        format!("Attacking enemy team {}! Sub-battle begins...\n", enemy_id)
                    }
                    None => "No adjacent enemy to attack. Move next to an enemy first.\n".into(),
                }
            } else { "No tactical map.\n".into() }
        }
        Command::Normal(tid) | Command::Skill { team_id: tid, .. } |
        Command::CdSkill { team_id: tid, .. } | Command::EnergySkill { team_id: tid, .. } |
        Command::Ultimate(tid) | Command::Wait(tid) => {
            // These are resolved when in sub-battle or used via the acting team flow.
            // For simplicity, treat as "use normal attack" context — the sub-battle auto-resolves.
            if game.phase == GamePhase::CombatTactical {
                // If an adjacent enemy exists, initiate battle (same as attack).
                if let Some(ref tac) = game.tactical {
                    if let Some(enemy_id) = tac.find_adjacent_enemy(tid) {
                        game.initiate_sub_battle(tid, enemy_id);
                        return format!("Engaging enemy team {}!\n", enemy_id);
                    }
                }
                "No adjacent enemy. Use 'move' to position first.\n".into()
            } else {
                "This command only applies during combat.\n".into()
            }
        }
        Command::SubStatus => {
            if let Some(ref sb) = game.sub_battle {
                save_lord::ui::render_sub_battle(sb)
            } else { "No sub-battle active.\n".into() }
        }
        Command::SubRetreat => {
            if game.sub_battle.is_some() {
                game.sub_battle.as_mut().unwrap().result = Some(save_lord::sub_battle::SubBattleResult::Retreat);
                "Retreating!\n".into()
            } else { "No sub-battle active.\n".into() }
        }
        Command::SubSkill { .. } | Command::SubPassive { .. } => {
            "Sub-battle skills are resolved automatically by the decision engine.\nType 'continue' to resolve.\n".into()
        }
        Command::CampfireRevive => {
            if game.phase != GamePhase::Campfire {
                return "Not at a campfire.\n".into();
            }
            // Revive all dead characters across all teams
            if let Some(ref mut tac) = game.tactical {
                for team in &mut tac.teams {
                    if team.side == Side::Player {
                        for m in &mut team.members {
                            if m.is_dead {
                                m.is_dead = false;
                                m.hp = m.max_hp / 2;
                            }
                        }
                    }
                }
            } else {
                // No tactical — revive lord's team in pool
                game.lord.character.is_dead = false;
                game.lord.character.hp = game.lord.character.max_hp;
            }
            "All fallen characters revived!\n".into()
        }
        Command::CampfireUpgrade(_tid) => {
            if game.phase != GamePhase::Campfire {
                return "Not at a campfire.\n".into();
            }
            let mut upgraded = false;
            if let Some(ref mut tac) = game.tactical {
                // Increase member cap for one player team
                for team in &mut tac.teams {
                    if team.side == Side::Player && team.max_members < 6 {
                        team.max_members += 1;
                        for m in &mut team.members {
                            m.atk += 5;
                            m.max_hp += 10;
                            m.hp = m.hp.max(m.max_hp / 2);
                        }
                        upgraded = true;
                        break;
                    }
                }
            }
            if upgraded {
                "Team upgraded! Member cap increased by 1, members gained +5 ATK and +10 max HP.\n".into()
            } else {
                "All teams already at max size (6). Members healed and buffed slightly.\n".into()
            }
        }
        Command::CampfireRest => {
            if game.phase != GamePhase::Campfire {
                return "Not at a campfire.\n".into();
            }
            if let Some(ref mut tac) = game.tactical {
                for team in &mut tac.teams {
                    if team.side == Side::Player {
                        for m in &mut team.members {
                            m.heal(m.max_hp / 3);
                        }
                    }
                }
            }
            "All units heal 30% HP.\n".into()
        }
        Command::CampfireLeave | Command::ShopLeave => {
            game.phase = GamePhase::Map;
            "You leave and return to the map.\n".into()
        }
        Command::ShopBuy(n) => {
            if game.phase != GamePhase::Shop {
                return "Not in a shop.\n".into();
            }
            let idx = (n as usize).saturating_sub(1);
            if idx >= game.shop_items.len() {
                return format!("No item with number {}.\n", n);
            }
            let item = game.shop_items[idx].clone();
            if game.overworld.gold < item.price {
                return format!("Not enough gold (need {}, have {}).\n", item.price, game.overworld.gold);
            }
            game.overworld.gold -= item.price;
            match item.kind {
                save_lord::ui::ShopItemKind::Equipment(eq_idx) => {
                    let all_eq = save_lord::equipment::all_equipment_templates();
                    if let Some(eq) = all_eq.get(eq_idx) {
                        game.pool.owned_equipment.push(eq.clone());
                        format!("Bought {}!\n", eq.name)
                    } else { "Item not found.\n".into() }
                }
                save_lord::ui::ShopItemKind::Relic(r_idx) => {
                    let all_r = save_lord::relics::all_relic_templates();
                    if let Some(r) = all_r.get(r_idx) {
                        game.pool.owned_relics.push(r.clone());
                        format!("Bought relic: {}!\n", r.name)
                    } else { "Item not found.\n".into() }
                }
                save_lord::ui::ShopItemKind::Potion => {
                    game.pool.owned_potions.push(save_lord::types::Consumable {
                        name: "Healing Potion".into(), kind: ConsumableKind::Potion, uses: 1,
                    });
                    "Bought a Healing Potion!\n".into()
                }
                save_lord::ui::ShopItemKind::Food => {
                    game.pool.owned_food.push(save_lord::types::Consumable {
                        name: "Travel Bread".into(), kind: ConsumableKind::Food, uses: 3,
                    });
                    "Bought Travel Bread!\n".into()
                }
                save_lord::ui::ShopItemKind::Character(_) => {
                    "Character purchases not implemented directly.\n".into()
                }
            }
        }
        Command::EventChoice(n) => {
            if game.phase != GamePhase::Event {
                return "Not in an event.\n".into();
            }
            if n < 1 || n > 3 {
                return "Choose 1, 2, or 3.\n".into();
            }
            let evt = game.pending_event.clone().unwrap();
            let choice = &evt.choices[(n as usize) - 1];
            let mut out = format!("{}\n", choice.result_text);

            match choice.effect {
                save_lord::events::EventEffect::GainEquipments(count) => {
                    let equips = save_lord::gacha::GachaPool::roll_equipment(count, &mut game.rng);
                    for e in equips {
                        out.push_str(&format!("  Got equipment: {}\n", e.name));
                        game.pool.owned_equipment.push(e);
                    }
                }
                save_lord::events::EventEffect::GainRelic => {
                    if let Some(r) = save_lord::gacha::GachaPool::roll_relic(&mut game.rng)
                        .or_else(|| {
                            let all = save_lord::relics::all_relic_templates();
                            let idx = game.rng.gen_range(0, all.len() as i32 - 1) as usize;
                            Some(all[idx].clone())
                        })
                    {
                        out.push_str(&format!("  Got relic: {}\n", r.name));
                        game.pool.owned_relics.push(r);
                    }
                }
                save_lord::events::EventEffect::HealAllPercent(pct) => {
                    if let Some(ref mut tac) = game.tactical {
                        for t in &mut tac.teams {
                            if t.side == Side::Player {
                                for m in &mut t.members {
                                    m.heal((m.max_hp as f64 * pct) as i32);
                                }
                            }
                        }
                    } else {
                        game.lord.character.heal((game.lord.character.max_hp as f64 * pct) as i32);
                    }
                    out.push_str("  All healed.\n");
                }
                save_lord::events::EventEffect::DamageOnePercent(pct) => {
                    let dmg = (game.lord.character.max_hp as f64 * pct) as i32;
                    game.lord.character.take_damage(dmg);
                    out.push_str(&format!("  Lord takes {} damage.\n", dmg));
                }
                save_lord::events::EventEffect::LordXp(xp) => {
                    if game.lord.add_xp(xp) {
                        out.push_str("  ★ Level up! Lord level increased!\n");
                    } else {
                        out.push_str(&format!("  Lord gains XP (+{:.0}%).\n", xp * 100.0));
                    }
                }
                save_lord::events::EventEffect::TeamAtkBuff(pct) => {
                    if let Some(ref mut tac) = game.tactical {
                        for t in &mut tac.teams {
                            if t.side == Side::Player {
                                for m in &mut t.members {
                                    m.atk = (m.atk as f64 * (1.0 + pct)) as i32;
                                }
                            }
                        }
                    }
                    out.push_str("  All teams gain ATK buff.\n");
                }
                save_lord::events::EventEffect::GainGold(amt) => {
                    game.overworld.gold += amt;
                    out.push_str(&format!("  Gained {} gold.\n", amt));
                }
                save_lord::events::EventEffect::GainPotions(n) => {
                    for _ in 0..n {
                        game.pool.owned_potions.push(save_lord::types::Consumable {
                            name: "Healing Potion".into(), kind: ConsumableKind::Potion, uses: 1,
                        });
                    }
                    out.push_str(&format!("  Gained {} potions.\n", n));
                }
                save_lord::events::EventEffect::GainFood(n) => {
                    for _ in 0..n {
                        game.pool.owned_food.push(save_lord::types::Consumable {
                            name: "Travel Bread".into(), kind: ConsumableKind::Food, uses: 3,
                        });
                    }
                    out.push_str(&format!("  Gained {} food.\n", n));
                }
                save_lord::events::EventEffect::SummonEnemies => {
                    if let Some(ref mut tac) = game.tactical {
                        // Extra enemy
                        let tmpl = save_lord::characters::template_by_id(game.rng.gen_range(0, 999) as u32);
                        let mut enemy = save_lord::team::Team::new(300, "Ambush", Side::Enemy,
                            save_lord::types::GridPos::new(50, 50));
                        let mut ch = save_lord::characters::Character::from_template(tmpl);
                        ch.atk = (ch.atk as f64 * 1.2) as i32;
                        ch.max_hp = (ch.max_hp as f64 * 1.2) as i32;
                        ch.hp = ch.max_hp;
                        enemy.add_member(ch).ok();
                        tac.add_team(enemy);
                    }
                    out.push_str("  ⚠ Ambush! An enemy appears!\n");
                }
                save_lord::events::EventEffect::SacrificeCharacter => {
                    // Simplified: lose 10% HP
                    game.lord.character.take_damage(game.lord.character.max_hp / 10);
                    if let Some(r) = save_lord::gacha::GachaPool::roll_relic(&mut game.rng)
                        .or_else(|| {
                            let all = save_lord::relics::all_relic_templates();
                            Some(all[0].clone())
                        })
                    {
                        game.pool.owned_relics.push(r.clone());
                        out.push_str(&format!("  Sacrifice made. Got relic: {}\n", r.name));
                    }
                }
                save_lord::events::EventEffect::PoisonOne(_, _) => {
                    out.push_str("  Poison applied.\n");
                }
                save_lord::events::EventEffect::HealOnePercent(_) => {}
                save_lord::events::EventEffect::Nothing => {
                    out.push_str("  Nothing happens.\n");
                }
            }

            game.pending_event = None;
            game.phase = GamePhase::Map;
            out
        }
        Command::Gacha => {
            let mut out = String::new();
            out.push_str("=== Gacha Pool ===\n");
            out.push_str(&format!("Characters owned: {}\n", game.pool.owned_characters.len()));
            out.push_str("You receive 100 gacha pulls after every victorious battle.\n");
            out.push_str("Characters have no rarity — all 1000 are equally likely.\n");
            out
        }
        Command::Merge(a, b) => {
            if a != b {
                "You can only merge two copies of the same character (same ID).\n".into()
            } else if game.pool.merge_characters(a) {
                format!("Characters #{} merged! Star level increased.\n", a)
            } else {
                format!("Not enough duplicates of character #{} to merge (need 2).\n", a)
            }
        }
        Command::UsePotion(name) => {
            if let Some(pos) = game.pool.owned_potions.iter().position(|p| p.name.eq_ignore_ascii_case(&name)) {
                game.pool.owned_potions.remove(pos);
                // Heal lord
                game.lord.character.heal(50);
                format!("Used {} — Lord healed 50 HP.\n", name)
            } else {
                format!("No potion named '{}'.\n", name)
            }
        }
        Command::UseFood(name) => {
            if let Some(pos) = game.pool.owned_food.iter().position(|p| p.name.eq_ignore_ascii_case(&name)) {
                game.pool.owned_food[pos].uses -= 1;
                if game.pool.owned_food[pos].uses == 0 {
                    game.pool.owned_food.remove(pos);
                }
                // Heal all 15%
                if let Some(ref mut tac) = game.tactical {
                    for t in &mut tac.teams {
                        if t.side == Side::Player {
                            for m in &mut t.members {
                                m.heal((m.max_hp as f64 * 0.15) as i32);
                            }
                        }
                    }
                }
                format!("Used {} — All allies heal 15%.\n", name)
            } else {
                format!("No food named '{}'.\n", name)
            }
        }
        Command::Unknown(msg) => {
            format!("Unknown command or error: {}\nType 'help' for command list.\n", msg)
        }
    }
}
