# 模块接口文档（面向AI）

本文档描述已实现的5个核心类的公共API，供AI理解与调用。

## 1. Grid (grid.py)

表示棋盘上的一个格子。

### 属性
- `x: int` - 横坐标，范围 [0, 棋盘宽度-1]
- `y: int` - 纵坐标，范围 [0, 棋盘高度-1]
- `states: List[Any]` - 可变的状态列表，可存放任意对象

### 类型约束
- `x`, `y` 为整数，无内置上下限，但通常在棋盘边界内
- `states` 列表长度不限，元素类型不限

### 方法
- `add_state(state: Any) -> None` - 添加一个状态对象（无返回值）
- `remove_state(state: Any) -> None` - 移除指定状态对象（如果存在）
- `has_state(state_type: type) -> bool` - 检查是否存在指定类型的状态，返回布尔值
- `get_states_of_type(state_type: type) -> List[Any]` - 返回所有该类型的状态列表
- `__repr__() -> str` - 返回可读表示，格式如 `Grid(x, y, states=数量)`

## 2. Board (board.py)

表示100x100的棋盘，由Grid组成。

### 类常量
- `WIDTH = 100`
- `HEIGHT = 100`

### 属性
- `grids: List[List[Grid]]` - 二维Grid数组

### 方法
- `get_grid(x: int, y: int) -> Grid` - 返回坐标处的Grid（越界抛出IndexError）
- `set_grid(x: int, y: int, grid: Grid) -> None` - 替换坐标处的Grid
- `add_state_to_grid(x: int, y: int, state: any) -> None` - 向指定格子添加状态
- `remove_state_from_grid(x: int, y: int, state: any) -> None` - 从指定格子移除状态
- `__repr__() -> str` - 返回棋盘尺寸信息

## 3. Speed (speed.py)

表示一个可变的速度值。

### 属性
- `value: float` - 当前速度值

### 方法
- `set(new_value: float) -> None` - 直接设置值
- `add(delta: float) -> None` - 增加值
- `multiply(factor: float) -> None` - 乘以因子
- `__repr__() -> str` - 返回速度值

## 4. SpeedBar (speed_bar.py)

管理速度条，支持溢出、变化因素。

### 属性
- `current: Speed` - 当前速度条值
- `max_speed: Speed` - 最大速度（可变）
- `change_factor: Speed` - 变化因素（可变）

### 方法
- `add(amount: float) -> None` - 增加速度条值（可溢出）
- `subtract(amount: float) -> None` - 减少速度条值（可为负）
- `update_by_factor() -> None` - 根据变化因素更新当前值
- `get_action_count() -> int` - 计算行动值（整数部分）
- `get_remainder() -> float` - 计算剩余值（余数）
- `consume_action() -> None` - 消耗一次行动（减去一个最大速度值）
- `overflow() -> float` - 返回溢出值（当前值减去整数倍最大速度）
- `__repr__() -> str` - 返回当前状态

## 5. Piece (piece.py)

表示游戏中的一个棋子（角色）。

### 属性
- `name: str` - 名称
- `x, y: int` - 位置坐标
- `speed: Speed` - 速度值
- `speed_bar: SpeedBar` - 单独的速度条
- `attack_power: Speed` - 攻击力（可变）
- `move_power: Speed` - 移动力（可变）
- `health: Speed` - 生命值（可变）
- `equipped_cards: List[any]` - 装备的卡牌（最多4张）
- `action_points: int` - 当前行动点数

### 方法
- `move_to(new_x: int, new_y: int) -> None` - 移动棋子
- `take_damage(amount: float) -> None` - 受到伤害
- `heal(amount: float) -> None` - 恢复生命
- `is_alive() -> bool` - 是否存活
- `equip_card(card: any) -> bool` - 装备卡牌（成功返回True）
- `unequip_card(card: any) -> bool` - 卸下卡牌（成功返回True）
- `update_speed_bar() -> None` - 根据速度更新速度条（每回合调用）
- `get_action_count() -> int` - 从速度条计算行动值
- `get_remainder() -> float` - 获取剩余速度条值
- `consume_action_point() -> bool` - 消耗一个行动点数（有点数则成功）
- `grant_action_points_from_speed() -> None` - 根据行动值兑换行动点数
- `__repr__() -> str` - 返回名称、位置和生命值

---

**注意**：所有类均设计为可扩展，避免硬编码。状态、卡牌等均通过泛型容器存储，方便后续添加新类型。