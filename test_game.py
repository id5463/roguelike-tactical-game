"""
测试游戏：创建棋子、卡牌、障碍，并运行若干回合。
包含AI怪物与玩家角色（玩家角色无AI，由随机行动模拟）。
"""

from game import Game
from piece import Piece
from card import (
    SwapPositionCard,
    PullCard,
    PushCard,
    GlobalSpeedCard,
    SelfDamageCard,
    SpeedToAttackCard,
    AttackToSpeedCard,
    DamageSixMultiplierCard,
    RebirthCard,
    AttackCard,
    MoveCard,
)
from obstacle import Obstacle
from range_matrix import RangeMatrix
from ai import SimpleMonsterAI, PassiveAI


def create_range_matrix(pattern: int = 0) -> RangeMatrix:
    """创建一个简单的范围矩阵（中心3x3区域为1）。"""
    matrix = RangeMatrix(default_value=0)
    for x in range(9, 12):
        for y in range(9, 12):
            matrix.set_cell(x, y, 1)
    return matrix


def main():
    print("初始化游戏（AI测试）...")
    game = Game()

    # 创建AI实例
    monster_ai = SimpleMonsterAI(aggressiveness=0.9)
    passive_ai = PassiveAI()

    # 创建三个棋子，分配AI
    piece1 = Piece("英雄", x=10, y=10, speed=60.0, attack_power=15.0, move_power=8.0, ai=None)  # 玩家控制
    piece2 = Piece("怪兽", x=20, y=20, speed=40.0, attack_power=20.0, move_power=5.0, ai=monster_ai)
    piece3 = Piece("法师", x=30, y=30, speed=80.0, attack_power=10.0, move_power=10.0, ai=passive_ai)

    # 创建卡牌
    swap_card = SwapPositionCard("换位术", "交换双方位置")
    pull_card = PullCard("引力牵引", "将目标拉近")
    push_card = PushCard("冲击波", "将目标推远")
    speed_card = GlobalSpeedCard("疾风祝福", "全体加速")
    self_damage_card = SelfDamageCard("自残", "减少自身血量")
    speed_to_attack_card = SpeedToAttackCard("迅捷攻击", "速度转攻击", factor=0.2)
    attack_to_speed_card = AttackToSpeedCard("狂暴加速", "攻击转速度", factor=0.1)
    six_mult_card = DamageSixMultiplierCard("六之幸运", "伤害尾数6时翻倍")
    rebirth_card = RebirthCard("重生", "死亡后复活", revive_health=30.0)
    attack_card = AttackCard("火球术", "造成火焰伤害", damage_multiplier=1.5)
    move_card = MoveCard("瞬步", "额外移动", move_bonus=3)

    # 为卡牌设置范围矩阵（示例）
    range_matrix = create_range_matrix()
    swap_card.set_range_matrix(range_matrix)
    pull_card.set_range_matrix(range_matrix)

    # 装备卡牌
    piece1.equip_card(swap_card)
    piece1.equip_card(speed_to_attack_card)
    piece2.equip_card(pull_card)
    piece2.equip_card(attack_to_speed_card)
    piece3.equip_card(rebirth_card)
    piece3.equip_card(attack_card)

    # 添加到游戏
    game.add_piece(piece1)
    game.add_piece(piece2)
    game.add_piece(piece3)

    # 添加障碍物
    rock = Obstacle("岩石", blocks_movement=True)
    game.add_obstacle(rock, x=15, y=15)
    tree = Obstacle("树木", blocks_movement=True, blocks_vision=True)
    game.add_obstacle(tree, x=25, y=25)

    print("游戏开始，运行10回合...")
    game.run(max_rounds=10)

    print("\n测试完成。")


if __name__ == "__main__":
    main()