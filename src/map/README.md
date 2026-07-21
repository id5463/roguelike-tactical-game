# 地图模块 (map)

地图模块负责游戏地图的数据结构和渲染。

## 子模块

### `grid` — 地图数据
- `MapGrid` — 地图网格（地形 + 小队 + 相机）
- `Terrain` — 地形枚举（Plain / Water / Mountain / Forest）
- `SquadEntry` — 小队条目（名称、坐标、类型、HP）
- `CELL_SIZE` — 每个格子显示的字符大小（10×10）

### `render` — 地图渲染
- `format_map_view()` — 将地图渲染为字符视图（带轴标签、小队方块、可选颜色）
- `render_minimap()` — 渲染全地图缩略图（降采样 + 小队标记）
- `render_squad_block()` — 渲染单个小队方块（10×10 字符）

## 通讯方式
- `render` 模块通过 `MapGrid` 引用读取地图数据
- 不直接修改 `MapGrid` 状态
