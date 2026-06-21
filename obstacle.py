"""
障碍物 (Obstacle) 类。
表示棋盘上的障碍物，可以阻挡移动或技能。
"""


class Obstacle:
    """障碍物，可以放置在格子上。"""

    def __init__(self, name: str, blocks_movement: bool = True, blocks_vision: bool = False):
        self.name = name
        self.blocks_movement = blocks_movement
        self.blocks_vision = blocks_vision

    def __repr__(self) -> str:
        return f"Obstacle(name={self.name})"