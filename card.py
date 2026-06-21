"""
卡牌 (Card) 系统基类。
所有卡牌的抽象基类，包括主动技能、被动技能、攻击方式、移动方式。
"""

from abc import ABC, abstractmethod
from typing import Optional
from range_matrix import RangeMatrix


class Card(ABC):
    """卡牌基类。"""

    def __init__(self, name: str, description: str = ""):
        self.name = name
        self.description = description
        self.range_matrix: Optional[RangeMatrix] = None  # 范围矩阵，可选

    def set_range_matrix(self, matrix: RangeMatrix) -> None:
        """设置卡牌的范围矩阵。"""
        self.range_matrix = matrix

    @abstractmethod
    def apply(self, user, target, board, **kwargs) -> any:
        """应用卡牌效果。子类必须实现。"""
        pass

    def __repr__(self) -> str:
        return f"Card(name={self.name})"


class ActiveSkillCard(Card):
    """主动技能卡牌。"""

    def __init__(self, name: str, description: str = ""):
        super().__init__(name, description)
        self.cooldown = 0  # 冷却回合数

    def apply(self, user, target, board, **kwargs) -> any:
        # 默认实现：打印信息
        print(f"{self.name} 被 {user} 对 {target} 使用")
        return None


class PassiveSkillCard(Card):
    """被动技能卡牌。"""

    def __init__(self, name: str, description: str = ""):
        super().__init__(name, description)
        self.is_permanent = True

    def apply(self, user, target, board, **kwargs) -> any:
        # 被动技能通常不直接调用，而是在特定条件下触发
        print(f"{self.name} 被动效果触发")
        return None


class AttackCard(Card):
    """攻击方式卡牌。"""

    def __init__(self, name: str, description: str = "", damage_multiplier: float = 1.0):
        super().__init__(name, description)
        self.damage_multiplier = damage_multiplier

    def apply(self, user, target, board, **kwargs) -> any:
        # 计算伤害
        damage = user.attack_power.value * self.damage_multiplier
        target.take_damage(damage)
        print(f"{self.name} 造成 {damage} 点伤害")
        return damage


class MoveCard(Card):
    """移动方式卡牌。"""

    def __init__(self, name: str, description: str = "", move_bonus: int = 0):
        super().__init__(name, description)
        self.move_bonus = move_bonus

    def apply(self, user, target, board, **kwargs) -> any:
        # 移动逻辑
        print(f"{self.name} 提供移动加成 {self.move_bonus}")
        return None


# ===== 具体主动技能卡牌 =====

class SwapPositionCard(ActiveSkillCard):
    """交换自身与目标的位置。"""

    def apply(self, user, target, board, **kwargs) -> any:
        if user and target:
            user_x, user_y = user.x, user.y
            user.move_to(target.x, target.y)
            target.move_to(user_x, user_y)
            print(f"{self.name}: {user.name} 与 {target.name} 交换位置")
        return None


class PullCard(ActiveSkillCard):
    """将目标拉近至相邻格子。"""

    def apply(self, user, target, board, **kwargs) -> any:
        # 简化：将目标移动到用户相邻的格子（如果可能）
        if user and target:
            # 计算方向向量
            dx = user.x - target.x
            dy = user.y - target.y
            # 归一化到最多一格
            if dx != 0:
                dx = 1 if dx > 0 else -1
            if dy != 0:
                dy = 1 if dy > 0 else -1
            new_x = target.x + dx
            new_y = target.y + dy
            target.move_to(new_x, new_y)
            print(f"{self.name}: 将 {target.name} 拉近到 ({new_x},{new_y})")
        return None


class PushCard(ActiveSkillCard):
    """将目标推远一格。"""

    def apply(self, user, target, board, **kwargs) -> any:
        if user and target:
            dx = target.x - user.x
            dy = target.y - user.y
            if dx != 0:
                dx = 1 if dx > 0 else -1
            if dy != 0:
                dy = 1 if dy > 0 else -1
            new_x = target.x + dx
            new_y = target.y + dy
            target.move_to(new_x, new_y)
            print(f"{self.name}: 将 {target.name} 推远到 ({new_x},{new_y})")
        return None


class GlobalSpeedCard(ActiveSkillCard):
    """使所有棋子速度增加。"""

    def apply(self, user, target, board, **kwargs) -> any:
        pieces = kwargs.get('pieces', [])
        for piece in pieces:
            piece.speed.add(10.0)  # 增加10点速度
        print(f"{self.name}: 全体速度增加10")
        return None


class SelfDamageCard(ActiveSkillCard):
    """减少自身血量。"""

    def apply(self, user, target, board, **kwargs) -> any:
        if user:
            user.take_damage(20.0)
            print(f"{self.name}: {user.name} 损失20点生命")
        return None


# ===== 具体被动技能卡牌 =====

class SpeedToAttackCard(PassiveSkillCard):
    """根据自身速度增加攻击力。"""

    def __init__(self, name: str, description: str = "", factor: float = 0.1):
        super().__init__(name, description)
        self.factor = factor

    def apply(self, user, target, board, **kwargs) -> any:
        # 被动效果通常在计算攻击力时触发，这里仅示范
        if user:
            user.attack_power.add(user.speed.value * self.factor)
            print(f"{self.name}: 基于速度增加攻击力")
        return None


class AttackToSpeedCard(PassiveSkillCard):
    """根据自身攻击力增加速度。"""

    def __init__(self, name: str, description: str = "", factor: float = 0.05):
        super().__init__(name, description)
        self.factor = factor

    def apply(self, user, target, board, **kwargs) -> any:
        if user:
            user.speed.add(user.attack_power.value * self.factor)
            print(f"{self.name}: 基于攻击力增加速度")
        return None


class DamageSixMultiplierCard(PassiveSkillCard):
    """伤害尾数刚好等于6时伤害乘2。"""

    def apply(self, user, target, board, **kwargs) -> any:
        # 这个效果应该在伤害计算时检查，这里仅占位
        print(f"{self.name}: 伤害尾数为6时伤害翻倍")
        return None


class RebirthCard(PassiveSkillCard):
    """被击败时重生一次。"""

    def __init__(self, name: str, description: str = "", revive_health: float = 50.0):
        super().__init__(name, description)
        self.revive_health = revive_health
        self.used = False

    def apply(self, user, target, board, **kwargs) -> any:
        # 当用户死亡时触发重生
        if user and not user.is_alive() and not self.used:
            user.heal(self.revive_health)
            self.used = True
            print(f"{self.name}: {user.name} 重生，恢复{self.revive_health}生命")
        return None