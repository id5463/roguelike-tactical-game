# 存档模块 (save)

负责游戏存档的序列化和文件管理。

## 子模块

### `data` — 数据结构
- `SaveData` — 存档顶层结构（版本、时间戳、种子、回合、金币、地图数据）
- `MapData` — 地图数据（宽高、小队列表、地形编码）
- `SquadData` — 小队存档数据

### `manager` — 存档管理
- `SaveManager` — 存档文件读写（JSON 格式）
- `save()` — 写入存档文件
- `load()` — 读取存档文件
- `list_saves()` — 列出所有存档

## 通讯方式
- 被 `app::game` 调用
- 使用 `crate::error` 中的 `GameError` / `GameResult`
