# Save Lord — Roguelite CLI Tactical RPG

A turn-based tactical roguelite built in Rust where **saving and loading is the core mechanic**.
Every random event is seeded from the start — reload to discover the perfect path.

## Features

- **Seeded randomness** — every run is a fixed, discoverable puzzle
- **Save/Load anywhere** — human-readable JSON saves, inspectable and editable
- **4-floor Slay-the-Spire style overworld** (Campfire / Shop / Monster / Event / Boss)
- **Tactical grid combat** (100×100 board, speed-bar turn order, terrain)
- **Sub-battle system** with position-based damage modifiers, first-strike, crits
- **1000 procedurally generated characters** with unique skills
- **Skill priority system**: Normal → Move → SP Skill → CD Skill → Energy → Ultimate
- **Relics, Equipment, Consumables, Gacha, Merge/Star-up**
- **AI opponent with multi-factor decision engine**
- **ASCII "Grail Layout" UI**- **Text API for AI agents** (`cargo run -- api` exposes JSON state snapshots)
- **Meta-progression** (XP between runs, unlocks)

## Build & Run

```bash
cd save-lord
cargo build --release
cargo run
```

### Command-line options

```bash
cargo run -- --seed <number>   # Fixed seed for reproducible runs
cargo run -- --api              # Print JSON API state snapshot and exit
```

### Controls / Commands

Type a command and press Enter. After most actions, type `continue` to advance.

| Command | Description |
|---------|-------------|
| `help` | Show full command reference |
| `status` | Global status overview |
| `map` | View the overworld tree map |
| `node <id>` | Travel to a reachable map node |
| `select <team_id>` | Select a team to view details |
| `move <id> <dir> [n]` | Move team (N/S/E/W/NE/NW/SE/SW) |
| `attack <id>` | Attack adjacent enemy (starts sub-battle) |
| `wait <id>` | Skip a team's turn |
| `continue` | Advance to next speed bar tick / next screen |
| `save <name>` | Save game to named slot (JSON in saves/) |
| `load <name>` | Load a named save |
| `listsaves` | List all saves |
| `viewsave <name>` | Inspect a save file (human-readable) |
| `inventory` | View characters/equipment/relics/potions/food |
| `teamlist` | List all active teams |
| `gacha` | View the gacha/character pool |
| `merge <id> <id>` | Star up a character (2 copies → 1 higher-star) |
| `quit` | Exit |

During **campfire**: type `A` (revive), `B` (upgrade), `rest` (heal 30%), or `leave`.
During **shop**: `buy <number>` to purchase, `leave` to exit.
During **events**: type `1`, `2`, or `3` to choose.
During **sub-battle**: type `continue` to resolve; `sub_retreat` to flee.

## Save Files

Saves are stored in `saves/save-lord/*.json` as pretty-printed JSON:

```json
{
  "name": "mysave",
  "seed": 123456789,
  "lord": { "level": 2, ... },
  "overworld": { "current_floor": 0, ... },
  "tactical": { "teams": [...], "skill_points": 5, ... },
  "pool": { "owned_characters": [...], ... }
}
```

You can freely edit saves to experiment, then `load` them.

## API for AI Agents

The game exposes a structured JSON snapshot of its state for AI interaction.
After every screen render, an `ApiState` is computed. To see it:

- Call the in-game command `status` for a readable overview.
- The `api::ApiState` struct is serialized to JSON when running with `--api`.

Key state fields:

```
phase: Map | CombatTactical | SubBattle | Campfire | Shop | Event | Victory | Defeat
teams: [ { id, side, position, members: [{name,hp,max_hp,atk,def,spd,star,position,energy,...}] } ]
skill_points, acting_team_id, available_commands
```

## Architecture

- `src/game.rs` — top-level state and game loop logic
- `src/combat.rs` — 100×100 tactical grid, speed bar, enemy AI movement
- `src/sub_battle.rs` — turn-based duel engine with damage formulas and decision engine
- `src/characters.rs` — procedural generation of 1000 distinct characters
- `src/skills.rs` — skill definitions and effects (damage/heal/buff/debuff/cleanse/revive)
- `src/equipment.rs`, `src/relics.rs` — items and global passives
- `src/events.rs` — choice-based event nodes
- `src/gacha.rs` — character pull/duplicate/star-up system
- `src/map.rs` — overworld tree (Slay the Spire style)
- `src/lord.rs` — Lord progression (max team count, level bonuses)
- `src/save.rs` — JSON save/load with human-readable inspection
- `src/ui.rs` — Grail layout ASCII renderer, team detail panels, help text
- `src/commands.rs` — command parser
- `src/api.rs` — structured state serialization for AI agents
- `src/rng.rs` — seedable ChaCha RNG for deterministic runs

## License

Provided as-is for the model crowdtest.
