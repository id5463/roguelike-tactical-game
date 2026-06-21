# 肉鸽战棋游戏 (Roguelike Tactical Game)

一个基于命令行的回合制战棋游戏，具有高度可扩展的类设计，避免硬编码，所有组件均通过类与API实现。

## 游戏特性

- **100x100 棋盘**：每个格子都是一个独立类，可存储任意状态。
- **速度条系统**：可变的速度条，支持溢出、负值，决定行动顺序。
- **棋子 (Piece)**：拥有位置、速度、攻击、移动等属性，可装备最多4张卡牌。
- **卡牌系统**：
  - 主动技能卡（换位、拉近、推远、全体加速、自伤等）
  - 被动技能卡（速度转攻击、攻击转速度、伤害尾数6翻倍、重生等）
  - 攻击卡、移动卡
- **范围矩阵**：20x20 矩阵存储技能/攻击/移动的范围。
- **障碍物**：可阻挡移动或视线的障碍类。
- **回合制战斗**：自动计算行动顺序，棋子根据行动点数执行动作。

## 项目结构

```
.
├── grid.py              # 格子类 (Grid)
├── board.py             # 棋盘类 (Board)
├── speed.py             # 速度类 (Speed)
├── speed_bar.py         # 速度条类 (SpeedBar)
├── piece.py             # 棋子类 (Piece)
├── action_order.py      # 行动顺序类 (ActionOrder)
├── range_matrix.py      # 范围矩阵类 (RangeMatrix)
├── card.py              # 卡牌基类与具体卡牌 (Card, ActiveSkillCard, ...)
├── obstacle.py          # 障碍物类 (Obstacle)
├── game.py              # 游戏主循环类 (Game)
├── test_game.py         # 测试脚本（演示游戏运行）
├── api_doc_1.md         # 前5个模块的API文档（面向AI）
└── README.md            # 本文件
```

## 快速开始

1. 确保安装 Python 3.7+。
2. 克隆或下载本目录。
3. 运行测试游戏：

```bash
python test_game.py
```

这将模拟一个包含三个棋子、多种卡牌和障碍物的战斗，自动进行10回合。

## 自定义游戏

您可以编辑 `test_game.py` 或创建自己的脚本：

```python
from game import Game
from piece import Piece
from card import SwapPositionCard, SpeedToAttackCard, etc.

# 创建游戏
game = Game()

# 添加棋子
hero = Piece("英雄", x=10, y=10, speed=60.0)
monster = Piece("怪兽", x=20, y=20, speed=40.0)

# 装备卡牌
hero.equip_card(SwapPositionCard("换位术"))
monster.equip_card(SpeedToAttackCard("迅捷攻击"))

# 添加到游戏
game.add_piece(hero)
game.add_piece(monster)

# 运行最多20回合
game.run(max_rounds=20)
```

## 设计理念

- **一切皆类**：格子、速度、速度条、卡牌、障碍物等全部抽象为类。
- **零硬编码**：数值、行为均通过类配置，易于扩展。
- **API 化**：类之间通过定义良好的方法交互，支持替换与扩展。
- **高度可扩展**：可通过添加新卡牌、新状态、新障碍物来丰富游戏内容。

## 后续扩展方向

1. 实现更复杂的卡牌效果（依赖地图位置、条件触发）。
2. 添加玩家输入（命令行交互选择行动）。
3. 增加更多棋子属性（防御、抗性、状态效果）。
4. 实现保存/加载游戏状态。
5. 添加图形界面（可选）。

## 许可证

本项目仅供学习与演示使用。