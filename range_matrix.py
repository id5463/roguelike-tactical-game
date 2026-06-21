"""
范围矩阵 (RangeMatrix) 类。
表示一个20x20的矩阵，用于存储技能、攻击、移动的范围。
矩阵以列表形式存储，每个元素是一个整数值（默认为0）。
"""

from typing import List


class RangeMatrix:
    """20x20的范围矩阵，共400个单元格。"""

    WIDTH = 20
    HEIGHT = 20
    SIZE = WIDTH * HEIGHT

    def __init__(self, default_value: int = 0):
        # 使用一维列表存储，按行优先顺序
        self.data: List[int] = [default_value] * self.SIZE

    def set_cell(self, x: int, y: int, value: int) -> None:
        """设置指定坐标的单元格值。"""
        if 0 <= x < self.WIDTH and 0 <= y < self.HEIGHT:
            index = y * self.WIDTH + x
            self.data[index] = value
        else:
            raise IndexError(f"坐标 ({x}, {y}) 超出矩阵范围")

    def get_cell(self, x: int, y: int) -> int:
        """获取指定坐标的单元格值。"""
        if 0 <= x < self.WIDTH and 0 <= y < self.HEIGHT:
            index = y * self.WIDTH + x
            return self.data[index]
        else:
            raise IndexError(f"坐标 ({x}, {y}) 超出矩阵范围")

    def set_all(self, value: int) -> None:
        """将所有单元格设置为指定值。"""
        self.data = [value] * self.SIZE

    def set_pattern(self, pattern: List[int]) -> None:
        """用给定列表（长度必须为400）设置整个矩阵。"""
        if len(pattern) != self.SIZE:
            raise ValueError(f"模式列表长度必须为 {self.SIZE}")
        self.data = pattern.copy()

    def get_row(self, y: int) -> List[int]:
        """返回第y行的列表。"""
        if 0 <= y < self.HEIGHT:
            start = y * self.WIDTH
            return self.data[start:start + self.WIDTH]
        else:
            raise IndexError(f"行号 {y} 超出范围")

    def get_column(self, x: int) -> List[int]:
        """返回第x列的列表。"""
        if 0 <= x < self.WIDTH:
            return [self.data[i * self.WIDTH + x] for i in range(self.HEIGHT)]
        else:
            raise IndexError(f"列号 {x} 超出范围")

    def __repr__(self) -> str:
        return f"RangeMatrix({self.WIDTH}x{self.HEIGHT})"