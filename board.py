"""
棋盘 (Board) 类。
包含 100x100 的格子网格。
"""

from typing import List
from grid import Grid


class Board:
    """表示游戏棋盘。"""

    WIDTH = 100
    HEIGHT = 100

    def __init__(self):
        # 初始化网格，每个元素是一个 Grid 实例
        self.grids: List[List[Grid]] = [
            [Grid(x, y) for y in range(self.HEIGHT)] for x in range(self.WIDTH)
        ]

    def get_grid(self, x: int, y: int) -> Grid:
        """获取指定坐标的格子。"""
        if 0 <= x < self.WIDTH and 0 <= y < self.HEIGHT:
            return self.grids[x][y]
        else:
            raise IndexError(f"坐标 ({x}, {y}) 超出棋盘范围")

    def set_grid(self, x: int, y: int, grid: Grid) -> None:
        """设置指定坐标的格子（替换）。"""
        if 0 <= x < self.WIDTH and 0 <= y < self.HEIGHT:
            self.grids[x][y] = grid
        else:
            raise IndexError(f"坐标 ({x}, {y}) 超出棋盘范围")

    def add_state_to_grid(self, x: int, y: int, state: any) -> None:
        """向指定格子添加状态。"""
        grid = self.get_grid(x, y)
        grid.add_state(state)

    def remove_state_from_grid(self, x: int, y: int, state: any) -> None:
        """从指定格子移除状态。"""
        grid = self.get_grid(x, y)
        grid.remove_state(state)

    def __repr__(self) -> str:
        return f"Board({self.WIDTH}x{self.HEIGHT})"