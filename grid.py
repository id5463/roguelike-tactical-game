"""
格子 (Grid) 类。
每个格子是棋盘上的一个单元，包含坐标和可变的状态列表。
"""

from typing import List, Any


class Grid:
    """表示棋盘上的一个格子。"""

    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y
        self.states: List[Any] = []  # 可以存放任意状态对象，例如障碍、效果等

    def add_state(self, state: Any) -> None:
        """向格子添加一个状态。"""
        self.states.append(state)

    def remove_state(self, state: Any) -> None:
        """从格子移除一个状态。"""
        if state in self.states:
            self.states.remove(state)

    def has_state(self, state_type: type) -> bool:
        """检查格子是否拥有特定类型的状态。"""
        return any(isinstance(s, state_type) for s in self.states)

    def get_states_of_type(self, state_type: type) -> List[Any]:
        """获取格子中所有指定类型的状态。"""
        return [s for s in self.states if isinstance(s, state_type)]

    def __repr__(self) -> str:
        return f"Grid({self.x}, {self.y}, states={len(self.states)})"