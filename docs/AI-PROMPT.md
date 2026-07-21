# AI 编程提示词：拯救领主（Save/Load the Lord）

> 请使用此提示词作为完整项目开发的唯一指引。
> 技术栈：Rust + ASCII 终端渲染，纯自研引擎，无外部依赖。
> 目标：用 AI 编程（如下 Claude/GPT 等）一次或分步完成整个项目。

---

## 一、任务概述

用 Rust 构建一个**纯终端 ASCII 渲染**的 Roguelike + 战棋 + 自动编程战斗游戏。游戏的核心是：

- 玩家拥有多个**队伍**，在地图上移动、遭遇敌人
- 战斗基于**双速度条体系**（地图层实时跑条 + 战斗层回合制）
- 战斗自动执行，基于玩家预设的 **Gambit 规则**（条件→行动编程）
- 杀戮尖塔式 Roguelike 节点推关
- 所有渲染用 **ASCII 字符**，纯终端界面

**不允许使用任何外部游戏引擎或游戏框架。** 可以使用 Rust 标准库和基础 crate（如 `crossterm`/`ratatui` 用于终端，`rand` 用于随机，`serde` 用于存档）。

---

## 二、项目结构

```
src/
├── main.rs              # 入口：初始化、主循环
├── app.rs               # 应用状态机（菜单/游戏/战斗/结算）
├── command_parser.rs    # 文本命令解析器（唯一输入方式）
├── render.rs            # ASCII 终端渲染器
├── map/
│   ├── mod.rs
│   ├── tile.rs          # 地图格定义
│   ├── grid.rs          # 100×100 矩阵网格
│   └── node_map.rs      # 杀戮尖塔式节点推图
├── squad/
│   ├── mod.rs
│   ├── squad.rs         # 队伍结构（队长名、排位、角色列表）
│   ├── formation.rs     # 排位系统（1排=移动特性, 2排=技能点技, 后续排位=其他）
│   └── backpack.rs      # 背包系统（替补/轮换池，格数限制）
├── character/
│   ├── mod.rs
│   ├── character.rs     # 角色 = 职业 × 种族 × 阵营
│   ├── stats.rs         # 属性系统
│   ├── level.rs         # 等级系统
│   ├── star.rs          # 升星系统（2合1升星）
│   ├── class.rs         # 职业定义
│   ├── race.rs          # 种族定义
│   └── faction.rs       # 阵营定义
├── skill/
│   ├── mod.rs
│   ├── skill.rs         # 6种技能类型定义
│   ├── speed_cd.rs      # ①速度CD技
│   ├── turn_cd.rs       # ②回合CD技
│   ├── energy_skill.rs  # ③能量技
│   ├── sp_skill.rs      # ④技能点技
│   ├── ultimate.rs      # ⑤终极技
│   └── basic_attack.rs  # ⑥平A
├── combat/
│   ├── mod.rs
│   ├── speed_bar.rs     # 双速度条（大条+小条）
│   ├── big_bar.rs       # 地图层速度条
│   ├── small_bar.rs     # 战斗层 Initiative 顺序
│   ├── ap_pp.rs         # AP/PP 资源系统
│   ├── battle.rs        # 小兔子跳战斗结算
│   └── damage.rs        # 伤害计算
├── gambit/
│   ├── mod.rs
│   ├── rule.rs          # 规则 = 条件 + 行动
│   ├── condition.rs     # 条件系统（HP<50%/敌人数量/状态...）
│   ├── action.rs        # 行动系统（放技能/加buff/治疗...）
│   ├── interpreter.rs   # Gambit 解释器（遍历规则→执行）
│   └── editor.rs        # Gambit 编辑器（命令行/图形/AI辅助）
├── meta/
│   ├── mod.rs
│   ├── progression.rs   # 局外养成（队伍上限/人数上限/背包格数）
│   ├── gacha.rs         # 抽奖系统
│   └── loot_table.rs    # 掉落表
├── items/
│   ├── mod.rs
│   ├── equipment.rs     # 装备系统（2槽位）
│   ├── potion.rs        # 药水（首领一次性）
│   ├── food.rs          # 食物（首领有限次数）
│   └── relic.rs         # 遗物（全局被动）
├── leader.rs            # 首领系统
├── counter.rs           # 克制系统（克制技能，非全局克制）
├── event.rs             # 游戏事件系统
├── save/
│   ├── mod.rs           # 存档/读档入口
│   ├── snapshot.rs      # 全量快照（某一时刻的完整游戏状态）
│   ├── branch.rs        # 分支管理（伪 git：branch / checkout / merge）
│   ├── diff.rs          # 差异比较（两个存档之间改了啥）
│   ├── history.rs       # 时间线管理（所有节点的 DAG）
│   └── human_readable.rs # 人类可读格式导出（叙事版 vs 数据版）
└── cli/
    ├── mod.rs           # 命令处理器入口（从 stdin 读，或从 --command 参数读）
    ├── commands.rs      # 所有 /xxx 命令的解析与分发
    ├── dumper.rs        # 结构化状态导出（纯文本格式）
    ├── simulator.rs     # 战斗模拟器（快速验证平衡性）
    └── formatter.rs     # 状态格式化器（保证 std 输出可被 AI 正则解析）
```

---

## 三、核心机制实现要求

### 3.1 地图系统

- **规格**: 标准矩阵网格 **100×100**，每个格子用 ASCII 字符表示地形/单位
- **推图模式**: 杀戮尖塔式节点图，卷轴式推进
- **节点类型**: 战斗节点、休息节点、事件节点、商店节点、首领节点
- **渲染**: 终端全屏渲染，支持**缩放/平移**查看
- **视野**: 探索过的区域保留可见，未探索区域显示迷雾

### 3.2 队伍系统

- 一个地图单位 = **一支队伍**，默认名 = 队长名
- **队伍人数上限** 和 **队伍数量上限** 可通过局外养成提升
- **排位决定技能位置**:
  - 第1排 → 队伍的**地图移动特性**
  - 第2排 → 队伍的**技能点技**
  - 后续排位 → 其他地图技能（由后续设计细化，先用占位实现）
- 每个角色**独立血量**，战斗中可阵亡
- 自定义队伍名功能

### 3.3 双速度条体系

#### 大条（地图层）
```
实时跑条 → 跑满 → 可以同时做三件事：
  ① 移动（走格子）
  ② 平A（普通攻击）
  ③ 释放技能点技（④号技能，每轮只能使用一次）
```

- 其他4种技能（速度CD技、回合CD技、能量技、终极技）可以在**任意时间插队释放**，包括打断敌人行动
- **推拉条是核心战术**：
  - 拉条（友方速度条前进）
  - 推条（敌方速度条后退）
  - 加速/减速（改变速度属性）
  - 技能变速（改变技能自身CD速度）
  - 技能造成位移（改变位置）
  - 更改单次移动的范围

#### 小条（战斗层）
```
造成伤害 → 触发小兔子跳 → 进入战斗结算
双方按 Initiative 轮流出招 → Gambit 自动执行
AP/PP 消耗 → 战斗结束 → 退回大地图
```

- 攻击方在小条战斗中享有**减伤 + 增伤**

### 3.4 战斗系统

- **触发条件**: 只有**造成伤害**时才触发战斗结算，接触不触发
- **战斗流程**:
  1. 攻方打中守方
  2. 双方全部队员参战
  3. 按 Initiative 决定行动顺序
  4. 轮到的角色消耗 1 AP 放主动技能（基于 Gambit 条件判断）
  5. 符合条件则自动消耗 PP 触发被动技能
  6. 一方 AP 耗尽 或 一方全灭 → 战斗结束
  7. 结算伤害，退回大地图
- **AP/PP 系统**: 每个角色独立，战后重置
- **战斗演出**: 本游戏**不存在演出，不存在动画**。战斗是纯文字输出。支持三种查看模式：
  - **瞬间模式**（默认）：战斗瞬间结算，一次性输出全部行动日志
  - **步进模式**：每次输出一行行动日志，按任意键进入下一个行动，适合人类观看
  - **跳过模式**：不输出任何战斗过程，只显示最终胜负结果
- **大地图速度条**：尽可能快，瞬间推进，不渲染过程动画

### 3.5 Gambit 编程系统

深度对标《最终幻想12》的 Gambit 系统：

- 每条规则 = **条件 + 行动**
- 每个角色可配置近 **100 条**条件判断
- **优先级排序**决定触发顺序
- **三种编程方式**:
  1. **命令行**: 手写条件规则（硬核玩家）
  2. **图形界面**: 可视化拖拽配置（终端内用 ASCII 菜单实现）
  3. **AI 辅助**: 自然语言描述策略，自动生成 Gambit

**条件示例**: HP < 50% / 敌人数量 > 2 / 队友有负面状态 / 自身有增益状态 / 敌人是某个职业...

**行动示例**: 释放技能X / 治疗队友 / 后退 / 前进 / 防御 / 使用道具...

### 3.6 技能系统（6种）

| # | 技能类型 | 释放时机 | 冷却/消耗 | 可造成伤害 |
|---|---------|---------|-----------|-----------|
| ① | **速度CD技** | 任意时间（可打断） | 按角色速度跑冷却 | ✅ |
| ② | **回合CD技** | 任意时间（可打断） | 按实际回合数跑冷却 | ✅ |
| ③ | **能量技** | 任意时间（可打断） | 消耗能量 | ✅ |
| ④ | **技能点技** | 仅在大条满时可放 | 消耗技能点，每轮一次 | ✅ |
| ⑤ | **终极技** | 任意时间（可打断） | 每局一次 | ✅ |
| ⑥ | **平A** | 大条满时可放 | 无消耗 | ✅ |

### 3.7 角色系统

- **角色 = 职业 × 种族 × 阵营**
- **获取方式**: 战斗后抽奖
- **等级系统**: 存在等级区分
- **升星系统**: 2合1升星（类似 TFT 但更激进）
- **装备**: 每个角色 2 个装备槽，装备通过抽奖获得

### 3.8 消耗品系统

| 类型 | 使用限制 | 特点 |
|------|---------|------|
| **药水** | 仅首领使用 | 一次性消耗品 |
| **食物** | 仅首领使用 | 可多次使用，有总次数限制 |
| **遗物** | 全局生效 | 被动型效果，整局有效 |

### 3.9 胜负条件

- **唯一条件**: 击杀对方**首领（Leader）**
- 首领所在队伍获得额外强化（属性提升）

### 3.10 克制系统

- 不存在全局克制关系
- 存在**克制技能**——对特定目标有额外效果
- 克制是可选**战术选择**，非硬性规则

### 3.11 局外养成

| 项目 | 效果 |
|------|------|
| 队伍数量上限 | 能同时拥有的最大队伍数 |
| 队伍人数上限 | 一支队伍能容纳的最大人数 |
| 背包格数 | 背包可存放的角色数量 |

---

## 四、架构与状态机

### 4.1 应用状态机

```
Menu → GameMap → Combat → Result → GameMap
                   ↑         ↓
                GameOver → MainMenu
```

- **Menu**: 主菜单（新游戏/继续/设置）
- **GameMap**: 地图探索、移动、推图、队伍管理
- **Combat**: 自动战斗（可观看/跳过）
- **Result**: 战斗结算、抽奖
- **GameOver**: 胜负判定

### 4.2 主循环伪代码

```rust
fn main_loop() {
    loop {
        handle_command();         // 处理文本命令（从 stdin 读取一行）
        update_speed_bars();      // 更新所有队伍的速度条
        check_big_bar_full();     // 检查是否有队伍大条满了
        handle_map_actions();     // 处理移动/攻击/技能
        check_damage_trigger();   // 检查是否触发小兔子跳
        if in_combat {
            run_combat_loop();    // 战斗层循环
        }
        render();                 // 重新渲染
        check_game_over();        // 检查胜负
    }
}
```

### 4.3 渲染引擎

- 使用 `crossterm` + `ratatui` 或纯 ANSI escape code
- 全屏终端渲染
- 地图区域 + 信息面板 + 状态栏分区
- 支持滚动/缩放
- ASCII art 角色/单位/技能特效

---

## 五、游戏 = 纯文本交互程序（AI 直接读全部输出）

> **不需要"调试接口"。不需要"结构化导出"。不需要两套输出系统。**
>
> 游戏所有 UI 都是 ASCII 文字。AI 不需要特制的查询命令——它直接读终端输出就行。
> 就像一个人坐在终端前看屏幕一样。

### 5.1 交互模型：stdio

游戏是一个**标准 stdin/stdout 交互程序**：

```
┌──────────────────────────────────────────────┐
│ 游戏输出到 stdout（AI 读到）：                │
│                                              │
│  +----------------------------------------+  │
│  | 地图:                                    │
│  |   . . . M . . . E . . .                 │  │
│  |   . . . . . . . . . . .                 │  │
│  |   Mary's Team | HP:120/150 | SPD:78%    │  │
│  |   技能: [Q]Slash [W]Whirlwind           │  │
│  | > _   ← 输入行                         │  │
│  +----------------------------------------+  │
│                                              │
│ AI 通过 stdin 输入：                          │
│   move east ⏎                                │
│   /squad list ⏎                              │
│   /SL tree ⏎                                 │
└──────────────────────────────────────────────┘
```

**AI 不需要任何特殊接口。它看到的画面跟人类完全一样，因为就是同一串 stdout 文字。**

游戏启动后，循环做这件事：
```
输出完整"屏幕"到 stdout → 等待 stdin 一行输入 → 处理 → 输出新"屏幕" → ...
```

### 5.2 运行模式

```
模式 1: AI 交互模式（默认）
  cargo run -- --seed 42
  → 游戏启动，输出第一帧到 stdout
  → 等待 stdin 输入
  → AI 打字（move east / /squad list / 任何命令）
  → 游戏处理，输出下一帧到 stdout
  → ...

模式 2: 全自动模式（不需要 AI 决策）
  cargo run -- --seed 42 --autoplay
  → 游戏自己玩自己
  → 所有输出到 stdout（AI 可以看，也可以不看）
  → 同时输出 events 日志到文件

模式 3: 无头回放模式（只看结果）
  cargo run -- --seed 42 --headless --autoplay
  → 不输出帧画面，只输出 events 日志
  → 最省 token
```

### 5.3 屏幕底部始终有一个输入行

游戏输出的最后一**行永远是输入提示**：

```
> _    ← 光标在这里等你打字
```

不管是人类还是 AI，都在这里输入命令。

**没有 F12、没有反引号、没有"弹出调试台"**。不需要按任何键——输入行始终在那里。

### 5.4 核心原则

1. **一切皆文本** — 整个"UI"就是 stdout 上的文字。AI 读到的和人看到的是同一串字符
2. **没有两套输出** — 不搞"界面输出"和"调试输出"两套东西。就一套。AI 不需要特殊接口
3. **可录制** — 把 stdout 完整保存下来就是"录像"（纯文本）
4. **可加速** — `--speed 1000` 全自动模式，跳过等待

### 5.5 自检命令

在输入行打 `/debug check` 可执行全面的游戏状态正确性检查：

```
===== SELF CHECK =====
[PASS] All squads have valid positions
[PASS] All HP values within bounds
[PASS] No duplicate characters across squads
[PASS] Speed bars in valid range (0..100)
[PASS] Gambit rules contain no invalid targets
[PASS] Backpack counts match stored total
[WARN] Squad_3 has 0 AP remaining - may be stuck
===== 6 PASS / 1 WARN / 0 FAIL =====
```

### 5.6 战斗日志

战斗中产生的日志会自动显示在 stdout 输出中，AI 直接读到：

```
[T1] Mary Slash → Goblin_1 -22HP (crit!)
[T1] Goblin_1 Bite → Mary -12HP
[T2] Leo Backstab → Goblin_1 -35HP
[T2] Goblin_1 DODGED!
[T3] Tom Fireball → Goblin_2 -28HP
[T3] Goblin_2 Fainted!
```

也可以通过 `/combat log` 命令查看历史战斗日志。

### 5.7 无头模式

`--headless` 参数让游戏不输出帧画面，只输出精简的事件日志：

```bash
cargo run -- --seed 42 --headless --autoplay
```

输出：

```
[EVENT] run_started | seed=42
[EVENT] squad_created | id=squad_0 name="Mary's Team"
[EVENT] battle_started | attacker=squad_0 defender=squad_e2
[EVENT] battle_action | turn=1 char=Mary skill=Slash target=Goblin_1 dmg=22
[EVENT] boss_defeated | winner=squad_0 turns=12
[EVENT] run_completed | result=victory total_turns=47
```

适用于批量测试和最终验收。

### 5.8 布局的 ASCII 友好性

1. **所有边框使用标准 ASCII 字符**：`+` `-` `|`，禁止使用 Unicode box-drawing（`┌┐└┘├┤┬┴┼│─`）
2. **关键信息有纯文本标签**：`HP:` / `AP:` / `SPD:`
3. **颜色仅作为辅助**：关闭颜色后纯文本仍能传达全部信息
4. **每行不超过 80 字符**
5. **单位/队伍有唯一文本 ID**：`squad_0` / `char_a3f8`

### 5.9 设计原理总结

```
游戏输出 → stdout → AI 直接读到 → 不需要额外接口
AI 输入 → stdin → 游戏处理 → 输出下一帧
┌─────────────────────────────────────┐
│ 整个游戏就是一个"读写文字"的程序      │
│ AI 的"眼睛"就是 stdout               │
│ AI 的"手"就是 stdin                  │
└─────────────────────────────────────┘
```

---

## 六、实现优先级

### Phase 1 — 核心骨架（可运行原型）
1. 项目初始化，Cargo.toml 配置
2. 状态机框架 + 主循环
3. 终端渲染引擎（地图 + 简单 UI）
4. 100×100 网格地图生成与渲染
5. 玩家角色和队伍基础结构
6. 文本命令解析器（move north / /squad list 等）
7. 基础移动命令（move north/south/east/west）

### Phase 2 — 核心战斗
8. 大条（地图层速度条）实现
9. 小条（战斗层 Initiative）实现
10. 6 种技能类型框架
11. Gambit 规则系统（条件+行动）
12. Gambit 解释器
13. AP/PP 系统
14. 基础伤害计算
15. 小兔子跳战斗流程

### Phase 3 — 内容填充
16. 职业 × 种族 × 阵营设计（至少各 5~8 种）
17. 具体技能设计（每个职业至少 4~6 个技能）
18. Gambit 编辑器（命令行模式先行）
19. 克制技能系统
20. 装备系统（2槽位）

### Phase 4 — Roguelike 与养成
21. 杀戮尖塔式节点图
22. 战斗结算 + 抽奖系统
23. 升星系统（2合1）
24. 背包系统
25. 消耗品（药水/食物/遗物）
26. 局外养成
27. 存档/读档

### Phase 5 — 打磨
28. AI 敌人 Gambit
29. 首领强化系统
30. ASCII 战斗动画
31. 图形化 Gambit 编辑器（终端内）
32. AI 辅助 Gambit 生成（自然语言→规则）
33. 平衡性调优
34. 命令自动补全（Tab 补全命令/参数）
35. 性能优化

---

## 七、技术约束

1. **语言**: Rust（最新稳定版）
2. **终端**: Windows 终端（需跨平台考虑）
3. **依赖**: 仅可使用以下 crate：
   - `crossterm` — 终端控制（光标、颜色，仅用于输出渲染，不用于键盘事件）
   - `ratatui` — 终端 UI 框架（可选，也可用纯 ANSI）
   - `rand` — 随机数
   - `serde` + `serde_json` — 存档序列化
   - `chrono` — 时间/日期（可选）
   - `anyhow` / `thiserror` — 错误处理
4. **禁止使用**: 任何游戏引擎（Bevy、Amethyst 等），任何图形渲染库
5. **输入**: 纯文本命令（唯一输入方式，在 `>` 提示符后打字）

---

## 八、设计约束与原则

1. **队伍人数是双刃剑**: 更多 AP/PP 资源 vs 更多独立血量单位需照顾
2. **升星 vs 阵容宽度**: 玩家需要在追高星和阵容多样化之间权衡
3. **技能 CD 类型策略**: 速度CD利好高速角色，回合CD利好慢速角色
4. **资源管理**: 首领的药水（一次性）和食物（有限次数）形成资源管理压力
5. **推拉条为核心战术**: 整个游戏的战术轴心
6. **背包管理**: 格数限制迫使玩家做取舍

---

## 九、待定项（需要 AI 做设计决策）

以下是在编码过程中需要你自行设计决策的开放问题。请根据游戏的整体平衡性和趣味性作出合理选择：

1. ~ 具体的职业列表（建议至少 6~8 个职业）
2. ~ 具体的种族列表（建议至少 4~6 个种族）
3. ~ 具体的阵营列表（建议至少 3~4 个阵营）
4. ~ 队伍人数各档位解锁的具体技能类型
5. ~ 后续排位（3~5排）的精确地图功能定义
6. ~ 战斗内是否有前后排站位
7. ~ 角色阵亡处理（可复活/永久死亡/局内复活？）
8. ~ 局外养成的具体数值与资源曲线
9. ~ 地图上敌人的生成逻辑
10. ~ 抽奖的具体机制（卡池设计/概率/保底）
11. ~ 游戏名称

对于以上每一项，请基于游戏设计最佳实践作出决策并记录在单独的 `DESIGN_DECISIONS.md` 中。

---

## 十、输出要求

请逐步完成所有 Phase 的实现。每完成一个 Phase，运行 `cargo build` 确保编译通过。每完成一个功能模块，添加对应测试。

**代码质量标准**:
- 所有 public 函数有文档注释（含 panic 条件、返回值说明、示例代码）
- 使用强类型（避免裸字符串/数字）
- 错误处理完善（不使用 unwrap/expect 在非测试代码中）
- 模块化设计，每个文件不超过 500 行
- 测试代码量 ≥ 生产代码量的 60%

### 10.1 设计过程透明化（每模块必须输出）

**每写一个模块/文件之前，必须先输出完整的设计分析**，格式如下：

```
===== 设计分析: <模块名> =====
[需求]  这个模块需要做什么？
[方案A] <方案描述>  优点:... 缺点:...
[方案B] <方案描述>  优点:... 缺点:...
[方案C] <方案描述>  优点:... 缺点:...
[选择]  我选择方案X，理由：...
[接口]  pub fn / pub struct 预览
[测试]  如何测试这个模块？
===== 设计分析结束 =====
```

**遇到编译错误时，必须输出错误分析**：

```
===== 编译错误分析 =====
[错误原文] <编译器输出的完整错误信息>
[根因分析] 为什么会有这个错误？
[修复方案] 怎么改？
[修改内容] 实际改了什么代码？
===== 错误修复完毕 =====
```

### 10.2 全流程通关验收（最终交付物）

**所有 Phase 完成后，必须执行一次完整的游戏通关流程**，并输出：

```
===== 完整通关录像开始 =====
[SESSION] test-run-001
[SEED] 42

[STEP 1] 启动游戏
  $ cargo run -- --seed 42
  > [终端输出全文粘贴]

[STEP 2] 主菜单 → 新游戏
  > 输入: New Game
  > [终端显示节点图全文]

[STEP 3] 第一场战斗
  > 进入战斗节点
  > [战斗日志全文]
  > 战果: 胜利/失败 HP余额:...

...（每个步骤都记录）

[STEP N] 击杀首领
  > [首领战完整日志]
  > 胜利！

[FINAL] 游戏结束画面全文
===== 完整通关录像结束 =====
```

**录像格式要求**：
- 纯文本 `.txt` 文件
- 每一帧 = 终端的逐行输出（包括所有 ASCII 界面、状态栏、战斗日志）
- 等效于"把整个终端的输出原样复制粘贴"
- 文件名格式：`playback_<Phase号>_<日期>.txt`
- 所有录像文件集中存放在 `playbacks/` 目录

### 10.3 每 Phase 结算报告

**每完成一个 Phase，必须输出结算报告**：

```
===== PHASE N 结算报告 =====
[耗时]   开始时间 → 结束时间
[文件清单]
  - src/xxx.rs (N行) - 功能描述
  - ...
[编译]   第1次: fail (错误原因) → 第X次: pass
[测试]   新增N个测试 通过M个
[cargo clippy] pass/fail
[cargo test]   pass/fail
[录制]   录像文件: playbacks/phase_N.txt
[检查点评估]
  ✅ 条目1: 证据...
  ✅ 条目2: 证据...
  ❌ 条目3: 未完成，原因...
[下阶段计划]
  打算怎么实现下一阶段
===== 结算报告结束 =====
```

**开始实现 Phase 1 吧。**

---

## 附录 A：Gambit 编程语言规范（Gambit DSL）

这是游戏最核心的系统之一。AI 需要为 Gambit 设计一套完整的领域特定语言。

### A.1 语法定义

```
rule       := "IF" condition "THEN" action ";" priority?
condition  := simple_cond | compound_cond
simple_cond:= target "." property operator value
compound_cond:= condition ("AND" | "OR") condition | "NOT" condition
target     := "self" | "ally" | "enemy" | "lowest_hp_ally" | "lowest_hp_enemy"
            | "nearest_enemy" | "most_dangerous_enemy" | "all_allies" | "all_enemies"
property   := "hp_pct" | "hp" | "ap" | "pp" | "energy" | "level" | "star"
            | "buff_count" | "debuff_count" | "distance" | "has_buff" | "has_debuff"
            | "class" | "race" | "faction" | "is_leader" | "is_low_hp" | "count"
operator   := "<" | "<=" | "=" | ">=" | ">" | "!=" | "has" | "not_has"
value      := number | string_literal | percentage
action     := "use_skill" "(" skill_id ")"
            | "guard"
            | "retreat"
            | "advance"
            | "use_item" "(" item_id ")"
            | "skip"
priority   := "PRIORITY" number
```

### A.2 示例规则

```
// 经典 Gambit 编程（支持注释）
IF self.hp_pct < 30% THEN use_skill(heal); PRIORITY 1
IF enemy.count >= 3 AND self.ap >= 3 THEN use_skill(whirlwind); PRIORITY 2
IF ally.hp_pct < 50% AND self.has_buff(haste) THEN use_skill(cure); PRIORITY 3
IF enemy.nearest.distance <= 2 THEN use_skill(slash); PRIORITY 4
IF enemy.has_buff(shield) THEN use_skill(dispel); PRIORITY 5
IF self.always THEN use_skill(attack);
```

### A.3 三种编辑模式

**命令行模式**：直接输入上述 DSL 文本

```
> /gambit edit Mary
[Gambit Editor - Mary (Warrior)]
当前规则（5条）:
  P1: IF self.hp_pct < 30% THEN use_skill(heal)
  P2: IF enemy.count >= 3 THEN use_skill(whirlwind)
  P3: ...

输入新规则，或:
  /add     - 添加规则
  /del <n> - 删除第n条
  /swap <a> <b> - 交换优先级
   /clear   - 清空全部
   /load    - 从文件加载
   /save    - 保存到文件
   /test    - 在当前状态下模拟测试规则（不会实际消耗 AP）
   /validate - 校验当前规则集：检查目标是否存在、技能名是否有效、条件字段是否存在
```

**图形模式**：终端内 TUI 界面，用命令操作

```
+- Gambit Editor: Mary ----------------------------+
| 命令: select <n> | edit <n> | del <n> | /help      |
+--------------------------------------------------+
| P1 | IF  self.hp_pct  <  30%  ->  use_skill(heal) |
| P2 | IF  enemy.count  >=  3  ->  use_skill(whirl) |
| P3 | IF  ally.hp_pct  <  50%  ->  use_skill(cure) |
| P4 | IF  enemy.has_buff(shield) -> use_skill(disp)|
| P5 | always  ->  use_skill(attack)                |
+--------------------------------------------------+
| 命令: add | import | export | test | save | back     |
+--------------------------------------------------+
```

**AI 辅助模式**：自然语言 → Gambit 代码（调用外部 LLM API，如 OpenAI / Anthropic 等）

> 注意：此功能需要外部 AI API 才能工作。游戏本身不内置 LLM。连接方式见附录 L（AI 玩家交互协议）。此功能对所有人开放——正常人类玩家玩到半程可能没兴趣写繁琐的规则，可以用自然语言直接生成。

```
> /gambit ai "当我的血量低于30%时优先治疗，
   旁边有3个以上敌人就放旋风斩，
   队友血量低于50%且我有加速buff时治疗队友，
   否则普通攻击"

[AI 生成结果]
  P1: IF self.hp_pct < 30% THEN use_skill(heal)
  P2: IF enemy.count >= 3 THEN use_skill(whirlwind)
  P3: IF ally.hp_pct < 50% AND self.has_buff(haste) THEN use_skill(cure)
  P4: IF self.always THEN use_skill(attack)
[确认应用？ Y/N]
```

### A.3.5 Gambit 校验系统（`/validate`）

Gambit 规则在保存前必须通过校验，防止运行时出错：

```
/validate 校验内容:
  [CHECK] 技能名是否存在（如 heal, whirlwind 必须是该角色已习得的技能）
  [CHECK] 条件字段是否合法（如 hp_pct 是存在的字段，xxx 不是）
  [CHECK] 运算符是否合法（< > = >= <= != has not_has）
  [CHECK] 目标选择器是否合法（self ally enemy lowest_hp_ally 等）
  [CHECK] 类型匹配：数值字段不能和字符串比较，布尔字段不能用 > <
  [CHECK] 是否有兜底规则（至少一条 always 或其他必然触发的规则）
  [WARN]  规则数量 > 20 条可能影响解释器性能
  [WARN]  高优先级规则条件过于苛刻可能导致永不被触发

示例输出:
  ===== GAMBIT VALIDATION =====
  [PASS]  heal: 角色 Mary 已习得此技能
  [PASS]  条件字段 self.hp_pct 合法
  [PASS]  运算符 < 合法
  [PASS]  存在兜底规则: P5 always -> use_skill(attack)
  [WARN]  P4: enemy.class == "Mage" -> 目前场上没有法师类型敌人
  ===== 4 PASS / 1 WARN / 0 FAIL =====
```

### A.4 解释器执行规则

```
每轮轮到角色行动时:
  1. 按优先级 P1→P99 遍历 Gambit 规则
  2. 评估条件的"目标选择器"（target）
     - "self" → 直接返回自身
     - "lowest_hp_ally" → 扫描所有队友找出HP最低的
     - "nearest_enemy" → 扫描所有敌人找最近的
     - 目标选择器返回 Option<Entity>，如果为 None 则该规则不满足
  3. 评估条件的"属性判断"（property operator value）
     - 从目标实体读取对应属性，与值比较
     - 复合条件（AND/OR/NOT）递归求值
  4. 条件满足 → 执行 action
  5. action 消耗 AP/PP → 检查是否还有 AP
  6. 角色 AP 耗尽 → 轮到下一个 Initiative 的角色
```

### A.5 条件系统完整清单

| 条件 | 写法示例 | 说明 |
|------|---------|------|
| 自身HP百分比 | `self.hp_pct < 30%` | 0~100% |
| 自身HP绝对值 | `self.hp < 50` | 具体数值 |
| 自身AP | `self.ap >= 3` | 行动点 |
| 自身PP | `self.pp > 0` | 被动点 |
| 自身能量 | `self.energy > 50` | 能量槽 |
| 自身等级 | `self.level >= 5` | 角色等级 |
| 自身星级 | `self.star >= 3` | ★数量 |
| 自身buff数量 | `self.buff_count >= 2` | 正面状态 |
| 自身debuff数量 | `self.debuff_count == 0` | 负面状态 |
| 是否有某buff | `self.has_buff(shield)` | 指定buff |
| 是否有某debuff | `self.has_debuff(poison)` | 指定debuff |
| 自身是首领 | `self.is_leader` | 布尔 |
| 敌人数量 | `enemy.count >= 3` | 场上敌人数 |
| 盟友血量最低 | `lowest_hp_ally.hp_pct < 50%` | 自动寻目标 |
| 最近敌人距离 | `nearest_enemy.distance <= 2` | 格数 |
| 敌人职业 | `enemy.class == "Mage"` | 职业名 |
| 敌人种族 | `enemy.race == "Elf"` | 种族名 |
| 敌人阵营 | `enemy.faction == "Arcane"` | 阵营名 |
| 敌人是否首领 | `enemy.is_leader` | 布尔 |
| 总是执行 | `self.always` | 兜底规则 |
| 复合条件 | `cond1 AND cond2 OR NOT cond3` | 任意组合 |

---

## 附录 B：战斗数值公式

### B.1 速度条（大条）进度公式

```
每 Tick 速度条增量 = base_speed × (1 + speed_buff_pct) × tick_multiplier

base_speed = 队伍中第1排角色的速度属性（取平均）
  （第1排决定队伍移动特性）

大条满值 = 100（固定）
角色每 Tick 增加的速度 = speed_stat / 100.0

首次满条 = 初始速度偏移随机化（0~30），避免所有队伍同时行动
```

### B.2 推拉条数值

```
拉条效果（友方速度条前进）:
  目标速度条 += push_power × (1 + caster_push_bonus)
  例: 拉条技能 power=20 → 速度条 +20

推条效果（敌方速度条后退）:
  目标速度条 -= pull_power × (1 + caster_pull_bonus)
  例: 推条技能 power=15 → 速度条 -15

加速效果（永久改变速度属性）:
  目标速度属性 ×= (1 + speed_change_pct)
  持续 duration 回合/秒

速度条边界: [0, 200]，超过100的部分在跑满后溢出保留
```

### B.3 伤害计算公式

```
物理伤害:
  base_damage = attacker.atk - defender.def × (1 - penetration%)
  final_damage = base_damage × skill_multiplier × (1 + attacker_dmg_bonus%)
               × (1 - defender_dmg_reduction%) × crit_multiplier

魔法伤害:
  base_damage = attacker.matk - defender.mdef × (1 - penetration%)
  final_damage = base_damage × skill_multiplier × (1 + attacker_dmg_bonus%)
               × (1 - defender_dmg_reduction%) × crit_multiplier

真实伤害（无视防御）:
  final_damage = skill_base_damage × skill_multiplier
               × (1 + attacker_dmg_bonus%)

暴击:
  暴击率 = attacker.crit_rate - defender.crit_resist
  暴击倍率 = 1.5 + attacker.crit_dmg_bonus

小条战斗攻方加成:
  攻击方减伤: final_damage ×= 0.85   （攻方受伤 -15%）
  攻击方增伤: final_damage ×= 1.15   （攻方输出 +15%）
```

### B.4 Initiative 行动顺序

```
每轮开始:
  all_characters.sort_by(|c| c.initiative.reverse())
  initiative = base_spd × (1 + buff_modifier) + random(0, 10)

行动消耗:
  主动技能 → 消耗 1 AP
  被动技能 → 消耗 1 PP（若条件满足自动触发）
  普攻     → 消耗 1 AP

每轮 AP 上限:
  每个角色每轮 AP = ceil(总AP / 预期轮数)，约 3~5 点
  战后重置
```

### B.5 冷却系统

```
速度CD（技能①）:
  每 Tick 冷却减少量 = 角色速度 × cd_speed_multiplier
  冷却到 0 → 技能可用

回合CD（技能②）:
  每经过一个小条回合 → 冷却减少 1
  冷却到 0 → 技能可用

技能点技（技能④）:
  大条满时消耗 1 技能点释放
  技能点池：队伍共享，每场战斗开始重置为满
```

### B.6 升星数值

```
★1 → ★2: 属性 +20%，技能效果 +10%
★2 → ★3: 属性 +15%，技能效果 +10%
★3 → ★4: 属性 +12%，技能效果 +8%
★4 → ★5: 属性 +10%，技能效果 +8%
★5 → ★6: 属性 +8%， 技能效果 +5%

（升星边际递减，鼓励阵容宽度而非单追高星）
```

### B.7 属性成长公式

```
角色每升一级:
  HP:    base_hp × (1 + 0.12 × level)
  ATK:   base_atk × (1 + 0.10 × level)
  DEF:   base_def × (1 + 0.08 × level)
  MATK:  base_matk × (1 + 0.10 × level)
  MDEF:  base_mdef × (1 + 0.08 × level)
  SPD:   base_spd × (1 + 0.05 × level)
  成长系数因职业而异（战士HP成长高，法师MATK成长高）
```

---

## 附录 C：各界面 ASCII 布局原型

### C.1 地图主界面

```
+----------------------------------------------------------------+
| Roguelike Auto-Tactics  回合: 12   Gold: 245  | 队伍: 2       |
+----------------------------------------------------------------+
|                                                                |
|     ^  ^     ^  ^     ^  ^     ^  ^                            |
|    .### ##..## ##....## ###....###                              |
|    .## ~~~ +-+ ##..+-+ ##..+-+##~                              |
|    .## ~~~ |M| ##..|E| ##..|G|##~   小地图:                    |
|    .## ~~~ +-+ ##..+-+ ##..+-+##~   +---+---+---+             |
|    .### ##..## ##....## ###....###   | M |   |   |             |
|     ^  ^     ^  ^     ^  ^     ^    +---+---+---+             |
|    ...............~~...........      |   | E |   |             |
|    ..........~~~~~.......~~...       +---+---+---+             |
|     ^  ^     ~~+-+~~......^  ^      |   | G | ? |             |
|                |P|                   +---+---+---+             |
|                +-+                                             |
|     Legend: M=MyTeam E=Enemy G=Goblin P=Player                 |
|     . = plain  ~ = water  # = mountain                         |
+----------------------------------------------------------------+
| Mary's Team [########..] 78%  HP: 120/150  AP: 5/5            |
| 技能: Slash | Whirlwind | Charge | Ult(ready!)                 |
| > _                                                             |
+----------------------------------------------------------------+
```

### C.2 战斗界面（观看模式）

```
+--------------------- Battle -----------------------------+
| Mary's Team          vs          Goblin Patrol           |
| +---+ +---+ +---+              +---+ +---+              |
| | M | | T | | L |              | G | | G |              |
| +---+ +---+ +---+              +---+ +---+              |
| HP:120  HP:80  HP:90           HP:60   HP:55            |
| AP:5/5  AP:3/5  AP:4/5         AP:3/3  AP:2/3           |
+------------------- Action Log ---------------------------+
| [T1] Mary  Slash -> Goblin_1          -22HP  ########.. |
| [T1] Goblin_1  Bite -> Mary           -12HP  ####...... |
| [T2] Leo  Backstab -> Goblin_1        -35HP  ########.. |
| [T2] Goblin_1  DODGED!                                   |
| [T3] Tom  Fireball -> Goblin_2        -28HP  ########.. |
| [T3] Goblin_2  Fainted!                                   |
| [T4] Goblin_1  Rage -> Leo            -18HP  ######.... |
+----------------------------------------------------------+
| 命令: next | skip | quit | speed <1|10|100|1000>          |
+----------------------------------------------------------+
```

### C.3 队伍管理界面

```
+-------------- Squad Manager -----------------------------+
| Mary's Team  (位置: 23,45)  人数: 3/4                     |
+----------------------------------------------------------+
| 排位 | 角色  | 职业/种族/阵营    | Lv *  | HP    | 功能 |
+------+-------+------------------+-------+-------+------+
| P1   | Mary  | Warrior/Human/   | Lv3   | 120   | 移动 |
|      |       |   Crusaders      | **    | /150  | 特性 |
| P2   | Tom   | Mage/Elf/        | Lv2   | 80    | 技能 |
|      |       |   Arcane         | *     | /80   | 点技 |
| P3   | Leo   | Rogue/Dwarf/     | Lv2   | 90    | 待定 |
|      |       |   Free           | *     | /90   |      |
| P4   | [空]  |                  |       |       |      |
+----------------------------------------------------------+
| 命令: select <n> | inspect <n> | backpack add/remove <n>   |
|       gambit <name> | equip <char> <item>                  |
+----------------------------------------------------------+
| 背包 (5/10): Anna(Priest) Lv2* | Kai(Archer) Lv1*       |
+----------------------------------------------------------+
```

### C.4 节点图（Roguelike 推图）

```
+----------- Stage 3: Goblin Forest ----------------------+
|                                                          |
|             +---+                                       |
|     +-------+ B +-------+                               |
|     |       +---+       |                               |
|   +-v-+               +-v-+                             |
|   | R |               | ? |                             |
|   +---+               +---+                             |
|     |                   |                               |
|   +-v-+               +-v-+                             |
|   | S +---------------+ E |                             |
|   +---+               +---+                             |
|     |                   |                               |
|   +-v-+               +-v-+                             |
|   | ? |               | B |                             |
|   +---+               +---+                             |
|     |                   |                               |
|     +-------+   +-------+                               |
|             +-v-v-+                                     |
|             |  L  |      Legend: B=Battle R=Rest         |
|             +-----+      ?=Event S=Shop E=Elite L=Leader|
|                                                          |
| 命令: select <node_name> | map | backpack                  |
+---------------------------------------------------------+
```

### C.5 游戏输出示例（AI 从 stdout 读到的内容）

AI 运行 `cargo run -- --seed 42` 后，从 stdout 读到的第一帧：

```
+--------------------------------------------------+
| 拯救领主 Save/Load the Lord       回合: 1  金:100|
+--------------------------------------------------+
|                                                    |
|     . . . . . . . . . . . . . . .                  |
|     . . . M . . . . . . . . . . .                  |
|     . . . . . . . . . . . . . . .                  |
|     . . . . . . . . . . . E . . .                  |
|     . . . . . . . . . . . . . . .                  |
|                                                    |
|  M=Mary's Team  E=Enemy  .=平原  ~=水  #=山       |
+--------------------------------------------------+
| Mary's Team | HP:120/150 | AP:5/5 | SPD:42        |
| Slash | Whirlwind | Charge | Ult(锁)                 |
| 命令: /squad | /backpack | /gambit | /map           |
+--------------------------------------------------+
> _                                                    ← AI 在这打字
```

AI 直接读这段文字，就能看到地图布局、队伍状态、可用命令。

AI 输入 `move right` → 游戏处理 → 输出下一帧 → AI 再读 → 循环。

---

## 附录 D：测试策略

### D.1 测试层级

```
层级 1: 单元测试（每个模块）
  位置: 每个 .rs 文件末尾的 #[cfg(test)] mod tests
  覆盖: 核心算法、数据验证、边界条件
  命令: cargo test --lib

层级 2: 集成测试（模块间交互）
  位置: tests/ 目录
  覆盖: 战斗流程、Gambit 解释、速度条交互
  命令: cargo test

层级 3: 场景测试（完整游戏流程模拟）
  位置: tests/scenarios/
  覆盖: 从开局到首领击杀的完整流程
  命令: cargo test --test scenarios

层级 4: 无头批量测试（平衡性验证）
  位置: tests/headless/
  覆盖: 随机种子 × 阵容组合 × 1000场模拟
  命令: cargo run -- --headless --batch tests/headless/cases.json
```

### D.2 必须覆盖的测试场景

```
战斗系统:
  - 速度条从0跑到满，验证正确增量
  - 推条到边界（0 和 200）不溢出
  - 拉条让行动提前
  - 无伤害接触不进战斗
  - 一方全灭战斗结束
  - AP 耗尽战斗结束

Gambit:
  - 条件 true → action 执行
  - 条件 false → 跳过到下一条
  - 优先级排序正确
  - 目标选择器返回正确的目标
  - 复合条件（AND/OR/NOT）正确求值
  - 所有规则都不满足 → 执行默认攻击

角色系统:
  - 升星 2合1 正确
  - 升级属性增长符合公式
  - 装备 2 槽位限制
  - 背包格数限制

地图系统:
  - 100×100 边界不越界
  - 障碍物不可通行
  - 节点图生成连通性（所有节点可达）
  - 视野迷雾正确

Debug 控制台:
  - 所有命令返回非空输出
  - /dump 输出可解析
  - /debug check 在正确状态下全部 PASS

存档:
  - 存档 → 读档 → 状态完全一致
```

### D.3 场景测试示例（Rust）

```rust
#[test]
fn test_gambit_execution_priority() {
    let mut battle = BattleBuilder::new()
        .add_attacker(create_warrior())
        .add_defender(create_goblin())
        .build();

    // 设置 Gambit: P1=自回血, P2=攻击
    battle.squads[0].members[0].gambit = vec![
        Rule::new(Condition::HpPct(LessThan, 30), Action::UseSkill("heal"), 1),
        Rule::new(Condition::Always, Action::UseSkill("attack"), 2),
    ];

    // HP > 30% → 应执行 P2（攻击）
    battle.squads[0].members[0].hp = battle.squads[0].members[0].max_hp;
    let action = battle.resolve_gambit(&battle.squads[0].members[0]);
    assert_eq!(action, "attack");

    // HP < 30% → 应执行 P1（治疗）
    battle.squads[0].members[0].hp = 10;
    let action = battle.resolve_gambit(&battle.squads[0].members[0]);
    assert_eq!(action, "heal");
}
```

### D.4 无头批量测试配置

```json
{
  "runs": 1000,
  "seed": 42,
  "matchups": [
    { "attacker": "warrior_mage_rogue", "defender": "goblin_patrol" },
    { "attacker": "tank_healer_dps",    "defender": "orc_raiders" },
    { "attacker": "full_mage_squad",    "defender": "knight_order" }
  ],
  "checks": [
    "win_rate > 40%",
    "avg_turns < 20",
    "no_side_ap_starvation == true",
    "max_damage_variance < 30%"
  ]
}
```

---

## 附录 E：Rust 实现最佳实践

### E.1 错误处理模式

```rust
// 正例：使用 thiserror 定义领域错误
#[derive(thiserror::Error, Debug)]
pub enum GameError {
    #[error("位置越界: ({0}, {1}) 不在 0..100 范围内")]
    PositionOutOfBounds(usize, usize),

    #[error("队伍已满: 最多 {max} 人")]
    SquadFull { max: usize },

    #[error("背包已满: 最多 {max} 格")]
    BackpackFull { max: usize },

    #[error("角色 {0} 已阵亡")]
    CharacterDead(String),

    #[error("技能 {0} 正在冷却中，剩余 {1} 回合")]
    SkillOnCooldown(String, u32),

    #[error("Gambit 规则语法错误: {0}")]
    GambitSyntaxError(String),

    #[error("存档损坏: {0}")]
    SaveCorrupted(String),
}

pub type GameResult<T> = Result<T, GameError>;

// 使用 anyhow 在顶层统一处理
// 库代码用 GameError，应用层用 anyhow
```

### E.2 强类型模式

```rust
// 正例：使用 newtype pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: u16,  // 0..100
    pub y: u16,  // 0..100
}

impl Position {
    pub fn new(x: u16, y: u16) -> GameResult<Self> {
        if x >= 100 || y >= 100 {
            return Err(GameError::PositionOutOfBounds(x as usize, y as usize));
        }
        Ok(Self { x, y })
    }

    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = self.x as f64 - other.x as f64;
        let dy = self.y as f64 - other.y as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

// 正例：用枚举而非字符串
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillType {
    SpeedCd,
    TurnCd,
    Energy,
    SkillPoint,
    Ultimate,
    BasicAttack,
}

// 反例：不要用裸字符串
// pub fn use_skill(skill_name: &str) { }  // ❌
// pub fn use_skill(skill: SkillId) { }      // ✅
```

### E.3 模块化与文件大小控制

```rust
// 每个文件不超过 500 行
// 超过时拆分为：
//   skill/
//   ├── mod.rs       // 重新导出 + 公共接口
//   ├── types.rs     // 类型定义（枚举、结构体）
//   ├── skill_impl.rs // 核心逻辑实现
//   └── test.rs      // 测试（如果单元测试太多）

// mod.rs 典型结构：
mod types;
mod skill_impl;

pub use types::*;
pub use skill_impl::*;
```

### E.4 性能注意事项

```
1. 100×100 地图全量渲染优化:
   - 只渲染视口范围内的格子（~40×20 = 800个）
   - 使用 Buffer 双缓冲，避免逐字符输出
   - 视野外的格子跳过渲染

2. 速度条更新:
   - 每 Tick 只需要更新活跃队伍的条
   - 使用优先队列（BinaryHeap）管理下一个满条队伍

3. Gambit 解释:
   - 预编译 Gambit 规则为中间表示，不每次都解析字符串
   - 规则满足时短路，不继续检查低优先级规则

4. 战斗日志:
   - 环形缓冲区固定大小（1000条），避免无限增长
   - 只在 debug 模式下记录详细信息，release 只记录摘要
```

### E.5 存档格式

```rust
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,                    // 格式版本号
    pub timestamp: String,               // 存档时间
    pub seed: u64,                       // 随机种子（用于复现）
    pub meta_progression: MetaProgress,  // 局外养成
    pub current_run: Option<RunData>,    // 当前进行中的一局
}

#[derive(Serialize, Deserialize)]
pub struct RunData {
    pub stage: u32,
    pub node_map: NodeMapState,
    pub squads: Vec<SquadData>,
    pub backpack: BackpackData,
    pub relics: Vec<RelicData>,
    pub gold: u64,
    pub event_log: Vec<String>,
}

// 存档文件: saves/autosave.json 和 saves/slot_1.json ~ slot_5.json
// 使用 serde_json，人类可读方便调试
```

---

## 附录 F：终端兼容性指南

### F.1 跨平台注意事项

```
Windows (终端):    使用 crossterm，确保 cmd/powershell/Windows Terminal 兼容
macOS (Terminal):  标准 ANSI，颜色支持完整
Linux:             同上

关键点:
  - 不要依赖特定终端的扩展功能
  - 颜色用 16 标准色，避扩展 256色/真彩色
  - 检测终端大小，自适应布局
  - 最小终端要求: 80×24 字符
```

### F.2 输入处理

游戏唯一输入方式：游戏循环等待 stdin 读取一行文本。

读入的文本交给**命令解析器**（`command_parser.rs`），它负责：

1. 解析命令类型（move/select//SL/...）
2. 解析参数（方向/数量/名称）
3. 派发给对应的系统处理
4. 如果解析失败 → 输出 "未知命令，输入 /help 查看帮助"

没有键盘事件、没有鼠标事件、没有 F-键、没有方向键。**只有文本行。**

```
游戏循环:
  stdin.read_line() → 得到一行文本
  → command_parser.parse(line)
  → 匹配命令处理器
  → 执行
  → 渲染下一帧到 stdout
  → 回到顶部
```

### F.3 渲染优化

```rust
// 双缓冲模式
struct RenderBuffer {
    chars: Vec<Vec<char>>,      // [行][列]
    styles: Vec<Vec<Style>>,    // 每个字符的颜色/样式
    dirty: Vec<Vec<bool>>,      // 脏标记，只更新变化的区域
}

impl RenderBuffer {
    fn new(width: u16, height: u16) -> Self { ... }
    fn set_cell(&mut self, x: u16, y: u16, ch: char, style: Style) { ... }
    fn flush(&mut self) { /* 只输出 dirty=true 的单元格 */ }
    fn clear(&mut self) { /* 全部标记为 dirty */ }
}
```

---

## 附录 G：项目启动快速参考

### G.1 Cargo.toml 初始配置

```toml
[package]
name = "roguelike-auto-tactics"
version = "0.1.0"
edition = "2021"

[dependencies]
crossterm = "0.28"
ratatui = "0.29"       # 可选，也可用纯 ANSI
rand = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
chrono = "0.4"          # 存档时间戳
clap = { version = "4", features = ["derive"] }  # 命令行参数解析

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### G.2 命令行参数（clap）

```rust
#[derive(Parser)]
#[command(name = "拯救领主 Save/Load the Lord")]
pub struct CliArgs {
    // ========== 运行模式 ==========

    /// 无头模式：不输出帧画面，只输出精简事件日志
    #[arg(long)]
    pub headless: bool,

    /// 随机种子（固定后所有 RNG 确定）
    #[arg(long, default_value = "0")]
    pub seed: u64,

    /// 全自动通关（内置 AI 自己玩）
    #[arg(long)]
    pub autoplay: bool,

    // ========== 存档加载 ==========

    /// 加载指定存档目录（不指定则开新局）
    #[arg(long)]
    pub load: Option<String>,

    // ========== 非交互模式（一次性命令）==========

    /// 执行一条命令后立即退出（不启动交互循环）
    /// 示例: --oneshot "/SL tree"
    #[arg(long)]
    pub oneshot: Option<String>,

    // ========== 测试 ==========

    /// 模拟两队对战: "squadA vs squadB"
    #[arg(long)]
    pub simulate: Option<String>,

    /// 批量测试配置文件路径
    #[arg(long)]
    pub batch: Option<String>,

    // ========== 录制与输出 ==========

    /// 录制模式: event | frame
    #[arg(long, default_value = "event")]
    pub record_mode: String,

    /// 录像文件输出路径
    #[arg(long)]
    pub record: Option<String>,

    /// 速度倍率: 1|10|100|1000|max
    #[arg(long, default_value = "1")]
    pub speed: String,

    /// Debug 模式
    #[arg(short, long)]
    pub debug: bool,
}
```

**运行示例**：

```bash
# 交互模式（AI 或人类用）：游戏输出画面 → 等待 stdin → AI 打字 → 循环
cargo run -- --seed 42

# 全自动通关 + 录像
cargo run -- --seed 42 --speed 1000 --autoplay --record final_run.events

# 无头自动通关（只输出事件日志）
cargo run -- --seed 42 --headless --autoplay

# 一次性查询（不启动交互循环）
cargo run -- --oneshot "/SL tree"

# 加载已有存档交互
cargo run -- --load saves/run_42/
```

### G.3 .gitignore

```
target/
saves/
*.log
*.json
.DS_Store
```

---

## 附录 H：AI 辅助 Gambit 生成的 NLP 接口规范

### H.1 自然语言 → Gambit DSL 映射表

```
用户说 "残血治疗" →
  IF self.hp_pct < 30% THEN use_skill(heal)

用户说 "集火首领" →
  IF enemy.is_leader THEN use_skill(attack)

用户说 "人多放AOE" →
  IF enemy.count >= 3 THEN use_skill(aoe_skill)

用户说 "先加buff再打" →
  IF self.has_buff(attack_up) = false THEN use_skill(buff_skill)
  IF self.always THEN use_skill(attack)

用户说 "保持距离放风筝" →
  IF nearest_enemy.distance <= 2 THEN use_skill(retreat)
  IF self.always THEN use_skill(ranged_attack)
```

### H.2 AI 接口命令

```
/gambit ai <自然语言描述>
  → AI 解析为 Gambit 规则列表
  → 显示给玩家确认
  → 确认后写入角色 Gambit

/strategy ai "描述我的整体战术"
  → AI 为全队生成一套配合的 Gambit
  → 示例: "坦克拉仇恨，奶妈保坦克，DPS集火"
    → Tank: IF enemy.nearest THEN use_skill(taunt); ...
    → Healer: IF ally.hp_pct < 50% THEN use_skill(heal); ...
    → DPS: IF enemy.has_debuff(taunted) THEN use_skill(backstab); ...
```

---

## 附录 I：项目里程碑检查清单

### Phase 1 完成标志
- [ ] `cargo run` 启动后显示主菜单
- [ ] 可以进入游戏，看到 100×100 ASCII 地图
- [ ] 输入 `move north` 可以移动玩家队伍
- [ ] 地图上有地形（平地/山脉/水域）
- [ ] 状态栏显示队伍名称、HP、速度条

### Phase 2 完成标志
- [ ] 大条实时跑条，满时提示可行动
- [ ] 遇到敌人触发小兔子跳战斗
- [ ] Gambit 规则可以生效（至少 "always → attack"）
- [ ] AP 耗尽战斗结束
- [ ] 战斗结算正确扣血

### Phase 3 完成标志
- [ ] 至少 6 个职业可玩
- [ ] 每个职业有独特技能
- [ ] 命令行 Gambit 编辑器正常工作
- [ ] 克制技能在战斗中有效
- [ ] 装备可穿戴/卸下

### Phase 4 完成标志
- [ ] 节点图可推，路线分支有效
- [ ] 战斗后抽奖获得新角色
- [ ] 2合1升星正常
- [ ] 背包调换正常
- [ ] 存档/读档完整

### Phase 5 完成标志
- [ ] AI 敌人有智能 Gambit 而非纯随机
- [ ] 首领战有挑战性
- [ ] 可以观战战斗动画
- [ ] `/debug check` 全部 PASS
- [ ] `cargo run -- --headless --batch tests/cases.json` 可跑通

---

---

## 附录 J：速度倍率 + 事件级录制 + 全自动通关

> ⚠️ **核心约束**：本游戏人类通关约需 10 小时（走图 + 战斗 + 配队）。
> 因此验收不可能逐帧录制 10 小时的终端输出——那会产生几百 MB 的 txt 和天量的 token。
> 
> **解决方案**：游戏内置时间加速 + 事件级录制 + 全自动无人通关模式。
> 三者配合，10 小时游戏可在几十秒内跑完，录制文件只有几十 KB。

### J.1 速度倍率系统

游戏内置 `/speed` 命令，随时调整时间流速：

```
/speed          → 查看当前倍率
/speed 1        → 1x（正常速度，人类游玩）
/speed 10       → 10x（快速推图）
/speed 100      → 100x（瞬间战斗）
/speed 1000     → 1000x（全自动速通）
/speed max      → 极限速度（无延迟，CPU 全速跑）
```

**速度倍率影响范围**：

| 系统 | 1x | 10x | 100x | 1000x |
|------|:--:|:---:|:----:|:-----:|
| 大条速度条 | 正常 | 10x | 100x | 1000x |
| 战斗内 Initiative 间隔 | 500ms | 50ms | 5ms | 0ms（瞬间结算）|
| 地图移动动画 | 300ms/步 | 30ms/步 | 3ms/步 | 瞬间 |
| Gambit 规则评估 | 实时渲染 | 快速渲染 | 仅输出关键事件 | 不渲染只输出摘要 |
| 节点过渡 | 2s 动画 | 200ms | 20ms | 瞬间 |
| `cargo run -- --speed max` | — | — | — | 纯 CPU 运算，无帧延迟 |

**无头模式下速度倍率自动设为 max**，终端输出关闭，只写日志文件。

### J.2 录制模式：事件级 vs 帧级

游戏录制有**两级模式**，用 `/record mode` 切换：

```
/record mode frame     → 帧级录制（逐帧记录终端输出，10小时 ≈ 200MB txt）
/record mode event     → 事件级录制（仅记录关键事件，10小时 ≈ 50KB txt）← 默认
```

**事件级录制的内容**（默认推荐，也是最终验收使用的模式）：

```
===== EVENT LOG — seed=42 speed=1000x =====
[EVENT] run_started             | stage=1 seed=42
[EVENT] node_map_generated      | nodes=12 branches=3
[EVENT] squad_created           | id=squad_0 name="Mary's Team" members=3
[EVENT] gambit_configured       | squad=squad_0 char=Mary rules=5
[EVENT] battle_started          | node=battle_1 attacker=squad_0 defender=squad_e2
[EVENT] battle_action           | turn=1 char=Mary skill=Slash target=Goblin_1 dmg=22
[EVENT] battle_action           | turn=1 char=Goblin_1 skill=Bite target=Mary dmg=12
[EVENT] battle_action           | turn=2 char=Leo skill=Backstab target=Goblin_1 dmg=35
[EVENT] battle_turn_end         | turn=2 ap_remaining: Mary=3 Leo=4 Tom=5
[EVENT] battle_victory          | winner=squad_0 casualties=0
[EVENT] gacha_result            | pool=common got=Kai(Archer/Elf/Free)★
[EVENT] squad_updated           | squad=squad_0 members=4
[EVENT] shop_purchased          | item=Potion cost=50 gold_remaining=200
[EVENT] rest_node               | squad=squad_0 hp_restored=45
[EVENT] boss_battle_started     | boss="Goblin King" squad_size=5
[EVENT] boss_battle_action      | turn=8 char=Mary skill=Ultimate target=Boss dmg=120
[EVENT] boss_defeated           | winner=squad_0 turns=12
[EVENT] run_completed           | result=victory total_turns=47 duration_game=2h13m
[EVENT] final_state             | /debug/dump → see final_state.txt
===== LOG END ===== total_events=342
```

**事件格式规范**（每行一条，AI 可精确正则解析）：

```
[EVENT] <事件类型> | <key=value> [| <key=value> ...]
```

所有事件类型清单：

| 事件 | 关键字段 |
|------|---------|
| `run_started` | seed, stage, squad_count |
| `node_map_generated` | nodes, branches, node_types |
| `squad_created` | id, name, members, leader |
| `squad_moved` | id, from_x, from_y, to_x, to_y |
| `character_added` | squad, char_id, name, class |
| `character_leveled` | char_id, old_level, new_level |
| `character_starred` | char_id, old_star, new_star |
| `gambit_configured` | char_id, rules_count |
| `battle_started` | attacker, defender, node |
| `battle_action` | turn, char, skill, target, dmg, crit |
| `battle_reaction` | turn, char, passive, effect |
| `battle_turn_end` | turn, ap_remaining_map |
| `battle_victory` | winner, casualties, turns |
| `battle_defeat` | loser, survivors |
| `gacha_result` | pool, char_id, name, star |
| `item_equipped` | char_id, item, slot |
| `item_used` | char_id, item, effect |
| `shop_purchased` | item, cost, gold_left |
| `rest_node` | hp_restored, debuffs_cleaned |
| `event_node` | event_type, choice, outcome |
| `speed_changed` | old_speed, new_speed |
| `save_created` | slot, turn_count |
| `boss_battle_started` | boss, squad_size |
| `boss_defeated` | winner, turns, damage_taken |
| `run_completed` | result, total_turns, duration |
| `debug_command` | command, output_size |
| `self_check` | pass, warn, fail |

### J.3 全自动通关模式（Auto-Play）

游戏内置 `/autoplay` 命令，让 AI 玩家自动完成一局游戏：

```
/autoplay start          → 启动自动推图（使用默认策略）
/autoplay stop           → 停止自动推图
/autoplay status         → 查看自动推图状态
/autoplay strategy <s>   → 设置自动策略（aggressive/balanced/defensive）
```

**自动推图的行为逻辑**（内置在游戏中，不需要外部 AI 控制）：

```
自动推图状态机:
  1. 当前节点为战斗节点 → 自动进入 → 用已配置的 Gambit 自动战斗
  2. 当前节点为休息节点 → 自动选择"全体恢复"
  3. 当前节点为商店节点 → 自动购买: 药水 > 食物 > 装备（按优先级）
  4. 当前节点为事件节点 → 自动选择奖励最高的选项
  5. 无可用节点 → 推进到下一层
  6. 首领节点 → 自动检查背包，自动调配最优队伍 → 自动战斗
  7. 领袖阵亡 → 自动结束
```

**命令行触发全自动通关**：

```bash
# 无头模式 + 1000x 速度 + 事件录制 + 自动推图
cargo run -- --headless --speed 1000 --record-mode event \
  --record playbacks/final_run.txt --seed 42 --autoplay

# 等价于：
# 1. 启动游戏
# 2. 自动 /speed 1000
# 3. 自动 /record mode event
# 4. 自动 /record start
# 5. 进入新游戏
# 6. 自动 /autoplay start
# 7. 等待游戏结束（首领被击杀）
# 8. 自动 /record stop
# 9. 保存录像文件
# 10. 退出
```

### J.4 录像文件的最终形态

最终交付的录像是一个**大小合理、结构清晰、可被 AI 和人类双方阅读的 txt 文件**：

```
===== PLAYBACK: final_run =====
Game:       Roguelike Auto Tactics
Date:       2026-07-07
Seed:       42
Speed:      1000x
Mode:       headless + autoplay
Duration:   10h 23m (game time) → 37.4s (real time)
File size:  128 KB
Events:     342 lines

===== GAME START =====
...事件日志全文（每行一个 [EVENT]）...

===== RUN RESULT =====
result: victory
turns: 47
battles: 12
casualties: 2
boss: Goblin King → defeated in 12 turns
mvp: Mary (Warrior) — 15 kills, 2,340 damage

===== FINAL STATE SUMMARY =====
squads: 2 (4 members, 3 members)
backpack: 7/10 slots used
stars: 1x ★★★ 2x ★★ 4x ★
gold: 1,240
relics: 2
unlocks: +1 squad slot

===== SELF CHECK =====
[PASS] 24/24 assertions passed
===== PLAYBACK END =====
```

### J.5 断言检查表

验收时的断言检查保持不变（21 条），但改成按 Phase 分阶段可执行：

```
===== 通关断言检查 =====
[BUILD]    cargo build --release 通过
[CLIPPY]   cargo clippy 无警告
[TEST]     cargo test 全部通过
[RECORD]   录像文件存在, 342 events

--- Phase 1 检查项 ---
[P01] 新游戏能正常启动                              PASS
[P02] 节点图能正常生成且连通                        PASS
[P03] 队伍能在地图上移动                            PASS

--- Phase 2 检查项 ---
[P04] 大地图速度条正常跑动                          PASS
[P05] 进入战斗节点触发小兔子跳                      PASS
[P06] Gambit 规则被正确执行                         PASS
[P07] AP/PP 消耗正确                                PASS
[P08] 伤害计算符合公式                              PASS
[P09] 战斗结算后退回地图                            PASS

--- Phase 3 检查项 ---
[P10] 战后抽奖获得新角色                            PASS
[P11] 2合1升星生效                                  PASS
[P12] 背包存入/取出正常                              PASS
[P13] 装备可穿戴/卸下                               PASS
[P14] 商店可购买物品                                PASS

--- Phase 4 检查项 ---
[P15] 消耗品（药水/食物）可使用                     PASS
[P16] 遗物全局生效                                  PASS
[P17] 节点图可推进至首领                            PASS
[P18] 首领战正常触发和结算                          PASS

--- Phase 5 检查项 ---
[P19] 击败首领 → 胜利画面                          PASS
[P20] 首领被击杀 → 失败画面                        SKIP(未测试)
[P21] /debug check 全部 PASS                        PASS
[P22] 存档 → 读档 → 状态一致                       PASS
[P23] --headless --autoplay --seed 42 可完整通关   PASS

--- 存档系统专项检查（附录 K） ---
[K01] /save snapshot "取名" 创建快照                PASS
[K02] /save branch "分支名" 创建分支                PASS
[K03] /save checkout <id> 回退到历史节点            PASS
[K04] 回退后再前进，分支拓扑不丢失                  PASS
[K05] /save diff <a> <b> 输出差异变化               PASS
[K06] /save graph 输出 ASCII 分支拓扑图             PASS
[K07] /save export <id> 输出人类可读格式            PASS
[K08] 人类可读格式包含：队伍/Gambit/背包/事件       PASS
[K09] 存档 → 修改 Gambit → 存档 → diff 可见变化    PASS
[K10] 1 小时游戏时间的存档，人类可读 < 5 秒理解     PASS

===== 总评: 33/33 PASS =====
```

### J.6 playbacks/ 目录结构

```
playbacks/
├── phase_1_skeleton.events     # Phase 1 事件级录像
├── phase_2_combat.events       # Phase 2 事件级录像
├── phase_3_content.events      # Phase 3 事件级录像
├── phase_4_roguelike.events    # Phase 4 事件级录像
├── phase_5_polish.events       # Phase 5 事件级录像
├── final_run.events            # 最终完整通关录像
├── final_state.txt              # /debug/dump 最终状态导出
└── assert_report.txt           # 断言检查表报告
```

### J.7 录像是"可复现"的

所有录像使用**固定随机种子**（`--seed 42`），因此：

- 同一份代码 + 同一 seed + 同一 autoplay 策略 → 事件日志**完全一致**
- 验收时可以用 `diff playbacks/final_run.events playbacks/expected.events` 对比
- 如果某次修改导致行为变化，事件日志会精确反映差异点
- 每行事件都有时间戳和上下文，可以精准定位到具体函数

---

## 附录 K：节点式存档系统——SL 是本游戏的核心玩法

> 🎮 **《拯救领主》的"拯救"不是 rescue——是 Save/Load。**
>
> 存档不是游戏的附属功能，不是"怕丢进度所以存一下"的保险措施。
> **SL（Save/Load）就是这游戏怎么玩的。**
>
> 核心循环：
> ```
> 面临决策 → 存档（创建一个节点）→ 尝试一条路 →
> 看结果 → 不满意 → 回到之前的节点 → 尝试另一条路
> ```
> 这游戏的"好玩"不来自于"一把通关"的快感，而来自于**反复实验、对比结果、优化策略**的乐趣。

### K.1 核心哲学

#### 原则 1：唯一格式 = 人类可读格式

存档**不存在"机器格式"和"人类可读格式"两种东西**。只有一个格式，它既是机器能解析的，也是人类能直接阅读的。

```
存盘文件 = 一个 .txt 文件
机器视角: 按固定语法解析每一行，重建 GameState
人类视角: 打开直接读，就像读一份战报

没有任何二进制格式，没有任何 "export" 命令。
/save export 这种命令不存在——因为存档本身就是可读的。
```

#### 原则 2：存档 = 节点，不是时间线

不是"时间旅行"（你有一个线性的时间线，只是能往回跳）。

而是**决策树节点**：
```
                  初始节点（第1天，RNG已确定）
                 /         |          \
          走左路          走右路      休息
         /      \         /    \
    战斗节点  事件节点  战斗  商店
    /    \      /    \
  赢    输   拿钱  拿装备
```

每个节点是**游戏世界的一个快照**。你从父节点分叉到子节点。子节点之间彼此独立——它们不是"同一个世界线的不同版本"，而是**从同一个决策点生长出的平行世界**。

#### 原则 3：随机性在诞生时已死

一切的随机性——抽奖结果、敌人阵容、掉落物——**在 run 创建的那一刻就已经固定了**。

```
run 开始: 用 seed=42 预生成一整套随机数表
  第1次抽奖: 必定是 角色A
  第2次抽奖: 必定是 角色B
  第3场战斗的敌人: 必定是 Goblin Patrol（3人队）
  第5层的事件: 必定是 "迷路的商人"

SL 可以改变 你的决策：买药水还是买装备？升这个角色还是那个？
SL 不会改变 RNG：同一节点抽奖，100 次都是同一个结果。
```

**设计意图**：防止"SL 刷 RNG"的无聊行为。SL 是用来**尝试不同策略**的，不是用来**凹极品装备**的。

### K.2 游戏内命令

```
/SL                     → 查看当前节点树
/SL node "取名"         → 在当前节点下创建一个子节点（= 存档）
/SL goto <node_id>      → 回到指定节点（= 读档）
/SL branch "分支标签"   → 从当前节点分叉（给节点加标签）
/SL diff <a> <b>        → 比较两个节点的差异
/SL tree                → ASCII 树形图显示所有节点
/SL list                → 列出当前路径上的所有节点
/SL info <node_id>      → 查看节点详情
```

**注意**：没有 `/SL delete`。**节点一旦创建永不删除**。整个决策树完整保留。

### K.3 存档 = 文本文件，唯一格式

每个存档节点是一个 **.txt 文件**，文件名 = `node_<id>.txt`。

内容格式如下——**这既是机器格式，也是人读格式**：

```
NODE: a1b2c3d4
PARENT: 9f8e7d6c
BRANCH: main
LABEL: "刚打完第3层精英，准备去商店"
RNG_SEED: 42
RNG_STEPS: 47
TIME_CREATED: 2026-07-07 14:23:15
TIME_GAME: 3h 47m
TURN: 47
STAGE: 3
NODE_TYPE: shop

[SQUADS]
squad_0: "Mary's Team" | pos(45,23) | leader: Mary | 4 members
  [0] Mary | Warrior | Human | Crusaders | Lv5 ★★ | HP:180/200 | SPD:42
  [1] Tom | Mage | Elf | Arcane | Lv4 ★ | HP:90/90 | SPD:38
  [2] Leo | Rogue | Dwarf | Free | Lv4 ★★ | HP:130/130 | SPD:55
  [3] Anna | Priest | Human | Crusaders | Lv3 ★ | HP:80/80 | SPD:32

squad_1: "Sky's Team" | pos(45,22) | leader: Sky | 2 members
  [0] Sky | Archer | Elf | Free | Lv3 ★ | HP:110/110 | SPD:48
  [1] Kai | Monk | Human | Order | Lv2 ★ | HP:150/150 | SPD:35

[BACKPACK]
slots: 5/12
- Anna (Priest) Lv2 ★ | backup healer
- Iron Sword | equipment | ATK+5
- Speed Ring | equipment | SPD+3
- Scroll of Fire | consumable | single-use

[CONSUMABLES]
leader_potion: 3 | leader only | one-time use
leader_food: 2 | leader only | 5 uses each
relic: 1 | Revive Cross | global passive | revives leader once

[GAMBITS]
Mary:
  P1: IF self.hp_pct < 30% | THEN use_skill(defend)
  P2: IF enemy.count >= 3 | THEN use_skill(whirlwind)
  P3: IF ally.hp_pct < 40% | THEN use_skill(taunt)
  P4: always | THEN use_skill(slash)

Tom:
  P1: IF enemy.count >= 3 | THEN use_skill(fireball)
  P2: IF enemy.has_buff(shield) | THEN use_skill(dispel)
  P3: IF self.energy >= 80 | THEN use_skill(thunder)
  P4: always | THEN use_skill(magic_arrow)

[META]
squad_max: 3 | next: 500xp
squad_size_max: 5 | next: 300xp
backpack_max: 15 | next: 200xp
gold: 340
total_battles: 18 | wins: 15 | losses: 3

[STATS]
mvp: Mary | 32 kills | 8,450 damage
total_damage: 12,300
bosses_defeated: 1

[EVENTS] (recent, max 20)
  #46 | stage:3 | battle | victory | vs Goblin Scout | loot: 45g
  #45 | stage:3 | battle | victory | vs Goblin Mage | loot: Scroll of Fire
  #44 | stage:3 | rest | full heal | all squad members
  #43 | stage:2 | shop | bought: Speed Ring | cost: 120g
  #42 | stage:2 | battle | ELITE | victory | loot: ★★ Tom dupe → ★★→★★★

[CHILDREN]
- b2c3d4e5 | "商店买完药水，准备去精英"
- f6g7h8i9 | "没买药水直接开干，试试水"
```

**严肃性说明**——这个格式就是存档的**唯一格式**。代码里没有 JSON、没有 binary、没有 bincode、没有 protobuf。就是纯文本，按段（SECTION）组织，每段内 key: value 或表格。解析器直接读这个 txt 重建状态。

### K.4 节点树可视化（/SL tree）

```
===== NODE TREE: run_42 =====
root -- "首领领养计划" [seed=42]
  |
  +-- a1b2 "第1层·出发"
  |   |
  |   +-- b2c3 "打完第1场战斗"
  |   |   |
  |   |   +-- c3d4 "选了左路"
  |   |   |   |
  |   |   |   +-- d4e5 [main] "商店买了铁剑"
  |   |   |   |   |
  |   |   |   |   +-- e5f6 [main] "精英战——赢了！" <-- 你现在在这里
  |   |   |   |
  |   |   |   +-- d4e6 [尝试] "没买剑，直接打精英"
  |   |   |       |
  |   |   |       +-- e6f7 [尝试] "精英战——输了..."
  |   |   |
  |   |   +-- c3d5 "选了右路（事件节点）"
  |   |
  |   +-- b2c4 "原地休息"
  |
  +-- a1b3 "重开？" [此路不通]

legend: 当前=<-- 分支=[标签] 末端=叶子
```

### K.5 差异对比（/SL diff）

```
===== DIFF: d4e5(买了铁剑) vs d4e6(没买剑直接打) =====

[差异原因]
  在节点 c3d4（第1层岔路）做了不同选择

[状态差异]
  HP: Mary 180→200（+20，因为精英战受伤少？不对，没打？）
  等一下——d4e6 输了，Mary 没受伤但士气低落了。

[关键差异]
  gold:   340 → 220（-120，没买剑省了钱？等等——d4e5 买了剑花了120，d4e6 没买）
  ——不对，d4e6 输了，但输了也有保底掉落 20g。

  战斗记录:
  d4e5: 赢了 → 获得 65g + 铁剑装备
  d4e6: 输了 → 获得 20g + 士气低落 debuff

[Gambit 差异]  无（两个分支 Gambit 相同）

[结论]
  买铁剑 → 赢了精英战 → 收益更高
  不买剑 → 输了精英战 → 收益低 + 士气惩罚
  推荐走 d4e5 路线。
```

### K.6 实现架构

```rust
/// 存档节点——唯一格式，纯文本，人类可读
#[derive(Debug, Clone)]
pub struct SaveNode {
    pub id: String,                    // 7位 hash (a1b2c3d)
    pub parent_id: Option<String>,     // 父节点 ID
    pub branch_label: String,          // 分支标签（如 "main", "尝试"）
    pub label: String,                 // 玩家给的名称
    pub file_path: PathBuf,            // node_<id>.txt 的路径
}

/// 节点解析器——读 .txt 文件重建 GameState
pub struct NodeParser;
impl NodeParser {
    pub fn parse(path: &Path) -> Result<GameState> {
        // 逐行读取 .txt 文件
        // 遇到 [SQUADS] 开始解析队伍段
        // 遇到 [GAMBITS] 开始解析 Gambit 段
        // ...
        // 纯文本解析，无外部依赖
    }
}

/// 节点树——管理所有节点的 DAG
pub struct NodeTree {
    pub run_seed: u64,
    pub nodes: HashMap<String, SaveNode>,
    pub current: String,  // 当前所在的节点 ID
}

impl NodeTree {
    /// 创建子节点：在当前节点下创建一个新节点
    pub fn create_child(&mut self, label: &str, state: &GameState) -> Result<String> {
        let id = generate_node_id();
        let path = self.nodes_dir.join(format!("node_{}.txt", id));
        // 将 GameState 序列化为人类可读格式并写入文件
        state.write_human_readable(&path)?;
        let node = SaveNode {
            id: id.clone(),
            parent_id: Some(self.current.clone()),
            branch_label: "main".to_string(),
            label: label.to_string(),
            file_path: path,
        };
        self.nodes.insert(id.clone(), node);
        self.current = id.clone();
        Ok(id)
    }

    /// 跳转到指定节点（读档）
    pub fn goto(&mut self, id: &str) -> Result<GameState> {
        let node = self.nodes.get(id).ok_or("node not found")?;
        self.current = id.to_string();
        NodeParser::parse(&node.file_path)
    }

    /// 差异比较
    pub fn diff(&self, id_a: &str, id_b: &str) -> Result<String> {
        // 读取两个节点的 .txt 文件
        // 逐段比较差异
        // 输出人类可读的 diff 报告
    }

    /// 导出整棵树为 ASCII 可视化
    pub fn tree_to_ascii(&self) -> String {
        // DFS 遍历 DAG
        // 生成如 K.4 所示的树形图
    }
}
```

**存档目录结构**：

```
saves/
└── run_20260707_142315_42/        # 每一局一个目录
    ├── tree.json                  # 节点树拓扑（纯引用关系，不带状态）
    ├── node_a1b2c3d.txt           # 每个节点一个 .txt 文件——人类可读
    ├── node_b2c3d4e.txt
    ├── node_c3d4e5f.txt
    └── ...
```

`tree.json` 很小，只存节点间的引用关系（父子链接 + 标签）。真正的游戏状态在 `node_*.txt` 里。

### K.7 SL 作为玩法的游戏设计

SL 不是"作弊工具"，而是**游戏机制的一部分**。体现在：

```
节点分叉是免费的 → 鼓励尝试
  每遇到一个决策点，鼓励玩家分叉：
  "买不买这把剑？→ 分两条路都走走看"

RNG 锁定 → 防止刷子行为
  "在本节点抽奖结果已定，SL 100 次也一样"
  "但你可以选择：接受这个结果继续，还是回到过去走另一条路"

节点永不删除 → 决策树就是你的游戏记录
  打完 10 小时，回看整棵决策树：
  "哦，原来我在第 3 层做过这个选择，难怪后面那么顺"
```

### K.8 AI 测试场景

```
场景 1: AI 尝试 Gambit 策略 A vs 策略 B
  → 在战斗前创建 node_pre_battle
  → 分叉: node_strat_A 和 node_strat_B
  → 各跑一次战斗
  → /SL diff node_strat_A node_strat_B
  → 哪个策略更好，数字说话

场景 2: AI 不小心团灭了
  → /SL goto 上一个节点
  → 不需要重跑 3 小时的进度
  → 最多损失 5 分钟

场景 3: 验收者（你）检查 AI 的工作
  → 打开 node_*.txt 直接读
  → 不需要加载游戏
  → 每一行的状态都能用人脑验证
  → /SL tree → 看看 AI 尝试了多少种策略
  → 分支越多 → 证明 AI 越努力（token 烧得越值）
```

### K.9 "拯救领主"的双关

```
游戏标题:   拯救领主
英文:      Save/Load the Lord
拼音:      Zhěngjiù Lǐngzhǔ

第一层意思（剧情）:
  你的领主被囚禁了，你要去救他。
  游戏目标 = 保护你的首领（Leader）不被击杀。

第二层意思（玩法）:
  Save/Load the Lord。
  拯救（Save/Load）你的领主。
  每次领主快死了 → /SL goto 上一个节点 → 换一种策略 → 再试一次。
  SL 本身就是"拯救领主"的方式。

第三层意思（元幽默）:
  玩家对 AI 说："帮我拯救一下领主。"
  AI 打开 /SL tree，找到一个最优分支，checkout 过去。
  人类玩家永远不知道 AI 在后台 SL 了多少次才通关。
```

---

## 附录 L：游戏交互方式汇总

> **总结**：这个游戏有三种运行模式，适用于不同的角色和场景。

### L.1 三种模式对比

```
模式         | 谁用     | 干什么                                   | 命令
-------------|---------|------------------------------------------|-----------------------------
交互模式     | AI/人类  | 输出"屏幕" → 等待输入 → 处理 → 循环      | cargo run -- --seed 42
全自动模式   | 验收者  | 内置 AI 自己通关，输出 events 文件       | cargo run -- --autoplay
一次性查询   | AI/验收者| 执行一条命令，打印结果，立即退出          | cargo run -- --oneshot "/SL tree"
```

**交互模式是核心**。其他两种是辅助。

### L.2 交互模式（AI 或人类玩）

游戏启动后，进入一个循环：

```
循环:
  1. 渲染完整"屏幕"到 stdout
     - 地图、队伍信息、战斗日志、输入行
     - AI 直接读到这段文字
  2. 等待 stdin 一行输入
  3. 处理命令（移动、查看、SL、Gambit 等）
  4. 回到 1
```

AI 看到的 stdout 内容示例：

```
+--------------------------------------------------+
| 拯救领主 Save/Load the Lord       回合: 1  金:100|
+--------------------------------------------------+
|     . . . M . . . . . . . . . . .                 |
|     . . . . . . . . . . . E . . .                 |
+--------------------------------------------------+
| Mary's Team | HP:120/150 | AP:5/5                  |
+--------------------------------------------------+
> _                                                    ← AI 在这打字
```

AI 输入 `move east` → 游戏处理 → 输出下一帧。

**不需要 `--command`。不需要特殊协议。就是标准 stdin/stdout 交互。**

### L.3 全自动模式（AI 不参与决策）

```bash
# 全自动通关，输出 events 到文件
cargo run -- --seed 42 --speed 1000 --autoplay --record playbacks/run.events

# 无头模式（不输出帧画面，只输出 events 日志）
cargo run -- --seed 42 --headless --autoplay
```

AI 或验收者跑完命令后，直接打开 events 文件看结果。

### L.4 一次性查询（oneshot）

```bash
cargo run -- --oneshot "/SL tree"
# 打印存档树 → 退出

cargo run -- --load saves/run_42/ --oneshot "/SL diff a1b2 c3d4"
# 加载存档 → 对比两个节点 → 退出
```

`--oneshot` 用于快速查信息，不启动完整交互循环。

### L.5 验收者（你）怎么检查 AI 的工作

你不需要运行游戏。你只看文件：

```bash
# 看事件日志
cat playbacks/run.events

# 看存档
cat saves/run_42/node_a1b2c3d.txt

# 对比两个存档节点
cargo run -- --load saves/run_42/ --oneshot "/SL diff a1b2 c3d4"
```

**你只需要能打开 .txt 文件和执行 `cargo run -- --oneshot`。**

### L.6 评测场景

```
场景 A: AI 写 Gambit
  1. AI 启动游戏，进入交互模式
  2. AI 通过 stdin 输入 Gambit 配置命令
  3. AI 输入 /sim fight squad_0 squad_e0 触发战斗
  4. AI 读 stdout 里的战斗日志判断输赢

场景 B: 全自动通关
  1. 验收者执行: cargo run -- --speed 1000 --autoplay --seed 42
  2. 游戏自己玩到通关
  3. 验收者看 events 文件和存档树

场景 C: 人工审查
  验收者直接打开存档 .txt 文件:
  - 队伍配置是否合理
  - Gambit 规则是否完整
  - 背包管理是否有条理
  不需要运行游戏，只需要读文本
```

---

## 附录 M：新手引导——"看 AI 玩一遍"

> 本游戏不设传统新手引导关卡或教程弹窗。
> **新手引导 = 玩家观看 AI 玩一局。**

### M.1 引导方式

游戏首次启动时，提示：

```
========================================
  欢迎来到《拯救领主》（Save/Load the Lord）
  
  这是一个为 AI 设计的 Roguelike 战棋游戏。
  人类也可以玩，但有一定复杂度。
  
  你希望如何开始？
  
  [1] 观看 AI 玩一局（推荐）——让 AI 演示一遍完整的游戏流程
  [2] 直接开始新游戏——你已经熟悉了游戏机制
  [3] 查看游戏机制说明——文本介绍
========================================
```

### M.2 "看 AI 玩"模式的行为

选择 [1] 后：
- 游戏创建一个新存档（seed=42）
- 使用内置 autoplay 自动推进
- 人类玩家可以输入 `pause` 随时暂停 AI 操作，手动接管
- 输入 `auto` 恢复 AI 控制
- 游戏以步进模式输出所有行动，人类可以看清每一步
- 全程事件日志记录，结束后可以回放

### M.3 人类玩家参考材料

游戏中内置 `/howtoplay` 命令，随时查看：

```
> /howtoplay
===== 游戏玩法速览 =====
1. Gambit 编程 = 给角色写"条件-行动"规则
   AI 能帮你写：/gambit ai "残血治疗，人多放AOE"
   你自己写：/gambit edit Mary

2. 双速度条
   大条 = 地图行动（走格子/平A/技能点技）
   小条 = 战斗内 Initiative（自动按 Gambit 打）

3. SL = 核心玩法
   遇到难题 → /SL node "存档" → 尝试 → 不满意 → /SL goto 回去
   详细用法：/help SL

4. 战斗=纯文字
   没有动画，没有演出。只有日志。
   瞬间模式/步进模式/跳过模式可选。

5. 纯命令操作
   所有操作通过打字完成。没有快捷键。
   移动: move north/south/east/west
   查看: /squad list | /char inspect <n>
   帮助: /help
========================================
```

---

**现在，开始实现 Phase 1 吧。记住：**
1. **每一步都要输出完整的设计分析（3 方案对比）**
2. **每次编译错误都要输出根因分析**
3. **每 Phase 完成后输出结算报告 + 事件级录像**
4. **全部完成后运行 `--headless --speed 1000 --autoplay --record-mode event --seed 42` 输出最终通关录像**
5. **断言检查表 22 条逐条 PASS 才算完成**
6. **存档系统必须支持无限 SL / 分支 / 人类可读导出——这是人类能通关 10 小时的保障**
