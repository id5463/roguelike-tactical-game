"""
速度 (Speed) 类。
表示一个可变的速率值。
"""


class Speed:
    """速度值，可以变化。"""

    def __init__(self, value: float = 0.0):
        self.value = value

    def set(self, new_value: float) -> None:
        """设置速度值。"""
        self.value = new_value

    def add(self, delta: float) -> None:
        """增加速度值。"""
        self.value += delta

    def multiply(self, factor: float) -> None:
        """乘以因子。"""
        self.value *= factor

    def __repr__(self) -> str:
        return f"Speed({self.value})"