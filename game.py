"""
游戏 (Game) 类。
管理游戏循环、回合逻辑和棋子交互。
"""

from typing import List
from board import Board
from piece import Piece
from action_order import ActionOrder
from obstacle import Obstacle
import random


class Game:
    """游戏主控制器。"""

    def __init__(self, width: int = 100, height: int = 100):
        self.board = Board()
        self.pieces: List[Piece] = []
        self.obstacles: List[Obstacle] = []
        self.round = 0

    def add_piece(self, piece: Piece) -> None:
        """添加一个棋子到游戏。"""
        self.pieces.append(piece)

    def add_obstacle(self, obstacle: Obstacle, x: int, y: int) -> None:
        """在指定位置添加障碍物。"""
        self.board.add_state_to_grid(x, y, obstacle)
        self.obstacles.append(obstacle)

    def remove_piece(self, piece: Piece) -> None:
        """从游戏中移除棋子。"""
        if piece in self.pieces:
            self.pieces.remove(piece)

    def update_speed_bars(self) -> None:
        """更新所有棋子的速度条（每回合开始调用）。"""
        for piece in self.pieces:
            piece.update_speed_bar()

    def determine_action_order(self) -> List[Piece]:
        """确定行动顺序并返回排序后的棋子列表。"""
        order = ActionOrder(self.pieces)
        return order.compute_order()

    def grant_action_points(self) -> None:
        """为所有棋子分配行动点数。"""
        order = ActionOrder(self.pieces)
        order.grant_action_points()

    def play_round(self) -> None:
        """执行一个回合。"""
        self.round += 1
        print(f"\n=== 第 {self.round} 回合开始 ===")

        # 1. 更新速度条
        self.update_speed_bars()

        # 2. 分配行动点数
        self.grant_action_points()

        # 3. 确定行动顺序
        ordered_pieces = self.determine_action_order()
        print(f"行动顺序: {[p.name for p in ordered_pieces]}")

        # 4. 每个棋子依次行动
        for piece in ordered_pieces:
            if not piece.is_alive():
                continue
            print(f"\n{piece.name} 的行动回合，行动点数: {piece.action_points}")
            # 简化：每个棋子执行一次攻击（如果有目标）
            self.execute_piece_turn(piece)

        # 5. 回合结束，清理状态
        self.end_round()

    def execute_piece_turn(self, piece: Piece) -> None:
        """执行单个棋子的行动。"""
        if piece.action_points <= 0:
            print(f"{piece.name} 没有行动点数，跳过")
            return

        # 消耗一点行动点数
        piece.consume_action_point()

        # 如果有AI，则使用AI决策
        if piece.ai is not None:
            from ai import Action
            # 获取其他棋子列表
            other_pieces = [p for p in self.pieces if p != piece and p.is_alive()]
            action = piece.ai.decide(piece, self.board, other_pieces)
            print(f"{piece.name} AI 选择 {action.action_type}")

            if action.action_type == 'move':
                x = action.params.get('x', piece.x)
                y = action.params.get('y', piece.y)
                piece.move_to(x, y)
                print(f"{piece.name} 移动到 ({x},{y})")
            elif action.action_type == 'attack':
                target = action.params.get('target')
                if target and target.is_alive():
                    damage = piece.attack_power.value
                    target.take_damage(damage)
                    print(f"{piece.name} 攻击 {target.name}，造成 {damage} 伤害")
            elif action.action_type == 'skill':
                target = action.params.get('target')
                card = action.params.get('card')
                if card and target:
                    card.apply(piece, target, self.board, pieces=self.pieces)
            elif action.action_type == 'wait':
                print(f"{piece.name} 等待")
            else:
                print(f"{piece.name} 未知动作，跳过")
        else:
            # 玩家控制的棋子（在自动模拟中随机攻击）
            targets = [p for p in self.pieces if p != piece and p.is_alive()]
            if targets:
                target = random.choice(targets)
                # 使用第一个装备的卡牌（如果有）
                if piece.equipped_cards:
                    card = piece.equipped_cards[0]
                    print(f"{piece.name} 使用卡牌 {card.name} 对 {target.name}")
                    card.apply(piece, target, self.board, pieces=self.pieces)
                else:
                    # 普通攻击
                    damage = piece.attack_power.value
                    target.take_damage(damage)
                    print(f"{piece.name} 普通攻击 {target.name}，造成 {damage} 伤害")
            else:
                print(f"{piece.name} 没有可攻击的目标")

    def end_round(self) -> None:
        """回合结束处理。"""
        # 移除死亡的棋子
        dead = [p for p in self.pieces if not p.is_alive()]
        for p in dead:
            print(f"{p.name} 已死亡")
            self.remove_piece(p)

    def is_game_over(self) -> bool:
        """检查游戏是否结束（仅剩一个或零个存活棋子）。"""
        alive = [p for p in self.pieces if p.is_alive()]
        return len(alive) <= 1

    def run(self, max_rounds: int = 20) -> None:
        """运行游戏，直到游戏结束或达到最大回合数。"""
        print("游戏开始！")
        while not self.is_game_over() and self.round < max_rounds:
            self.play_round()
            if self.is_game_over():
                break

        # 游戏结束
        alive = [p for p in self.pieces if p.is_alive()]
        if len(alive) == 1:
            print(f"\n游戏结束！胜利者是 {alive[0].name}")
        else:
            print("\n游戏结束！没有幸存者。")