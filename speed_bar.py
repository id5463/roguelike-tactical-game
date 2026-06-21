"""
速度条 (SpeedBar) 类。
管理速度条的值，支持溢出、变化因素等。
"""

from speed import Speed


class SpeedBar:
    """速度条，用于决定回合顺序。"""

    def __init__(self, max_speed: float = 100.0, change_factor: float = 1.0):
        self.current = Speed(0.0)  # 当前速度条值
        self.max_speed = Speed(max_speed)  # 最大速度（可变）
        self.change_factor = Speed(change_factor)  # 变化因素（可变）

    def add(self, amount: float) -> None:
        """增加速度条值，可以溢出。"""
        self.current.add(amount)

    def subtract(self, amount: float) -> None:
        """减少速度条值，可以为负。"""
        self.current.add(-amount)

    def update_by_factor(self) -> None:
        """根据变化因素更新当前值（例如每回合自动变化）。"""
        self.current.multiply(self.change_factor.value)

    def get_action_count(self) -> int:
        """
        计算行动值：当前值除以最大速度的整数部分。
        返回整数行动次数。
        """
        if self.max_speed.value == 0:
            return 0
        return int(self.current.value // self.max_speed.value)

    def get_remainder(self) -> float:
        """计算剩余值（余数）。"""
        if self.max_speed.value == 0:
            return 0.0
        return self.current.value % self.max_speed.value

    def consume_action(self) -> None:
        """消耗一次行动（减去一个最大速度值）。"""
        self.current.add(-self.max_speed.value)

    def overflow(self) -> float:
        """返回溢出值（如果当前值超过最大速度的整数倍）。"""
        return self.current.value - self.get_action_count() * self.max_speed.value

    def __repr__(self) -> str:
        return f"SpeedBar(current={self.current.value}, max={self.max_speed.value}, factor={self.change_factor.value})"