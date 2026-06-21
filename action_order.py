"""
行动顺序 (ActionOrder) 类。
根据速度条计算棋子的行动顺序。
"""

import random
from typing import List
from piece import Piece


class ActionOrder:
    """管理一組棋子的行动顺序。"""

    def __init__(self, pieces: List[Piece]):
        self.pieces = pieces

    def compute_order(self) -> List[Piece]:
        """
        计算行动顺序。
        规则：
        1. 每个棋子根据 speed_bar 计算行动值（整数部分）和剩余值。
        2. 先按行动值降序排列（行动值高的先行动）。
        3. 行动值相同的，按剩余值降序排列。
        4. 如果都相同，随机排序。
        """
        # 为每个棋子计算行动值和剩余值
        piece_data = []
        for piece in self.pieces:
            action_count = piece.get_action_count()
            remainder = piece.get_remainder()
            piece_data.append((piece, action_count, remainder))

        # 排序
        piece_data.sort(key=lambda x: (x[1], x[2]), reverse=True)

        # 处理随机性：对于行动值和剩余值都相同的组，随机打乱
        i = 0
        while i < len(piece_data):
            j = i
            while j < len(piece_data) and piece_data[j][1] == piece_data[i][1] and piece_data[j][2] == piece_data[i][2]:
                j += 1
            if j - i > 1:
                # 随机打乱这个子列表
                sublist = piece_data[i:j]
                random.shuffle(sublist)
                piece_data[i:j] = sublist
            i = j

        return [item[0] for item in piece_data]

    def grant_action_points(self) -> None:
        """为所有棋子根据行动值兑换行动点数。"""
        for piece in self.pieces:
            piece.grant_action_points_from_speed()

    def next_round(self) -> None:
        """进入下一轮：更新所有棋子的速度条。"""
        for piece in self.pieces:
            piece.update_speed_bar()