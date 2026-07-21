# 错误模块 (error)

统一的错误类型定义。

## 子模块

### `types` — 错误类型
- `GameError` — 游戏错误枚举
  - `PositionOutOfBounds` — 坐标越界
  - `TileNotPassable` — 地形不可通行
  - `UnknownCommand` — 未知命令
  - `EmptyInput` — 空输入
  - `SquadNotFound` — 小队不存在
  - `SaveError` — 存档错误
  - `LoadError` — 读档错误
  - `IoError` — I/O 错误
- `GameResult<T>` — 别名 `Result<T, GameError>`

## 通讯方式
- 被所有其他模块引用
