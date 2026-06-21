"""
棋子 (Piece) 类。
代表游戏中的角色，拥有位置、速度、卡牌等属性。
"""

from typing import List, Optional, Any
from speed import Speed
from speed_bar import SpeedBar


class Piece:
    """游戏棋子。"""

    def __init__(
        self,
        name: str,
        x: int,
        y: int,
        speed: float = 50.0,
        max_speed: float = 100.0,
        attack_power: float = 10.0,
        move_power: float = 5.0,
        ai: Any = None,
    ):
        self.name = name
        self.x = x
        self.y = y
        self.speed = Speed(speed)  # 速度值
        self.speed_bar = SpeedBar(max_speed)  # 单独的速度条
        self.attack_power = Speed(attack_power)  # 攻击力（可变）
        self.move_power = Speed(move_power)  # 移动力（可变）
        self.health = Speed(100.0)  # 生命值（可变）
        self.equipped_cards: List[Any] = []  # 装备的卡牌，最多4张
        self.action_points = 0  # 当前行动点数（由行动值换取）
        self.ai = ai  # AI控制器，None表示玩家控制

    def move_to(self, new_x: int, new_y: int) -> None:
        """移动棋子到新位置。"""
        self.x = new_x
        self.y = new_y

    def take_damage(self, amount: float) -> None:
        """受到伤害。"""
        self.health.add(-amount)

    def heal(self, amount: float) -> None:
        """恢复生命值。"""
        self.health.add(amount)

    def is_alive(self) -> bool:
        """是否存活。"""
        return self.health.value > 0

    def equip_card(self, card: any) -> bool:
        """装备一张卡牌，如果已有4张则失败。"""
        if len(self.equipped_cards) >= 4:
            return False
        self.equipped_cards.append(card)
        return True

    def unequip_card(self, card: any) -> bool:
        """卸下一张卡牌。"""
        if card in self.equipped_cards:
            self.equipped_cards.remove(card)
            return True
        return False

    def update_speed_bar(self) -> None:
        """根据速度更新速度条（每回合调用）。"""
        self.speed_bar.add(self.speed.value)

    def get_action_count(self) -> int:
        """从速度条计算行动值。"""
        return self.speed_bar.get_action_count()

    def get_remainder(self) -> float:
        """获取剩余速度条值。"""
        return self.speed_bar.get_remainder()

    def consume_action_point(self) -> bool:
        """消耗一个行动点数，如果有点数则成功。"""
        if self.action_points > 0:
            self.action_points -= 1
            return True
        return False

    def grant_action_points_from_speed(self) -> None:
        """根据行动值兑换行动点数。"""
        actions = self.get_action_count()
        self.action_points += actions
        # 消耗掉整份速度条值
        for _ in range(actions):
            self.speed_bar.consume_action()

    def __repr__(self) -> str:
        return f"Piece(name={self.name}, pos=({self.x},{self.y}), health={self.health.value})"