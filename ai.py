"""
AI 系统基类。
为棋子提供自动决策能力，支持扩展不同的AI策略。
"""

from abc import ABC, abstractmethod
from typing import Optional, Tuple, List
from piece import Piece
from board import Board
import random


class Action:
    """表示一个动作。"""

    def __init__(self, action_type: str, **kwargs):
        self.action_type = action_type  # 'move', 'attack', 'skill', 'wait'
        self.params = kwargs

    def __repr__(self):
        return f"Action({self.action_type}, {self.params})"


class AI(ABC):
    """AI 基类。"""

    def __init__(self, aggressiveness: float = 0.8):
        self.aggressiveness = aggressiveness  # 攻击性系数

    @abstractmethod
    def decide(self, piece: Piece, board: Board, other_pieces: List[Piece]) -> Action:
        """根据当前状态决定下一步动作。"""
        pass


class SimpleMonsterAI(AI):
    """简单的怪物AI：移动到最近的敌人并攻击。"""

    def decide(self, piece: Piece, board: Board, other_pieces: List[Piece]) -> Action:
        # 找出所有存活的敌方棋子
        enemies = [p for p in other_pieces if p.is_alive() and p != piece]
        if not enemies:
            return Action('wait')

        # 找到最近的敌人
        nearest = min(enemies, key=lambda e: self.distance(piece, e))
        # 如果相邻，则攻击
        if self.is_adjacent(piece, nearest):
            # 如果有装备攻击卡牌，则使用；否则普通攻击
            if piece.equipped_cards:
                for card in piece.equipped_cards:
                    if card.__class__.__name__ == 'AttackCard':
                        return Action('skill', target=nearest, card=card)
            return Action('attack', target=nearest)
        else:
            # 尝试向敌人移动一步
            dx = nearest.x - piece.x
            dy = nearest.y - piece.y
            step_x = 1 if dx > 0 else -1 if dx < 0 else 0
            step_y = 1 if dy > 0 else -1 if dy < 0 else 0
            # 确保移动不超出棋盘边界
            new_x = piece.x + step_x
            new_y = piece.y + step_y
            if 0 <= new_x < 100 and 0 <= new_y < 100:
                return Action('move', x=new_x, y=new_y)
            else:
                # 无法移动，等待
                return Action('wait')

    @staticmethod
    def distance(p1: Piece, p2: Piece) -> float:
        """计算两个棋子之间的欧几里得距离。"""
        return ((p1.x - p2.x) ** 2 + (p1.y - p2.y) ** 2) ** 0.5

    @staticmethod
    def is_adjacent(p1: Piece, p2: Piece) -> bool:
        """判断两个棋子是否相邻（八方向）。"""
        return max(abs(p1.x - p2.x), abs(p1.y - p2.y)) <= 1


class PassiveAI(AI):
    """被动AI：只防御，不主动攻击。"""

    def decide(self, piece: Piece, board: Board, other_pieces: List[Piece]) -> Action:
        # 如果有敌人相邻，则攻击；否则随机移动或等待
        enemies = [p for p in other_pieces if p.is_alive() and p != piece]
        adjacent_enemies = [e for e in enemies if SimpleMonsterAI.is_adjacent(piece, e)]
        if adjacent_enemies:
            target = random.choice(adjacent_enemies)
            return Action('attack', target=target)
        else:
            # 随机移动一格（包括可能不动）
            moves = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)]
            dx, dy = random.choice(moves)
            new_x = piece.x + dx
            new_y = piece.y + dy
            if 0 <= new_x < 100 and 0 <= new_y < 100:
                return Action('move', x=new_x, y=new_y)
            else:
                return Action('wait')