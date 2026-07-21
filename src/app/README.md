# 应用模块 (app)

应用模块负责游戏循环、帧渲染和终端交互。

## 子模块

### `game` — 游戏状态与逻辑
- `Game` — 游戏主状态（地图、消息、存档路径）
- 命令执行（teleport / camera / save / load / quit）
- 存档/读档的数据转换
- 主游戏循环 `run()`

### `frame` — 帧渲染
- `format_frame()` — 构建圣杯式布局的完整帧
- `colorize_output()` — 给 `[P]`/`[E]` 标记添加 ANSI 颜色
- `build_left_panel()` — 左侧小队信息面板
- `build_right_panel()` — 右侧小地图 + 图例面板
- 辅助函数 `pad_or_trim()` / `fit_in()`

### `terminal` — 终端检测
- `detect_terminal_size()` — 获取终端列数和行数
- Windows 下使用 WinAPI `GetConsoleScreenBufferInfo`

## 通讯方式
- `game` 调用 `frame::format_frame()` 和 `frame::colorize_output()`
- `game` 调用 `terminal::detect_terminal_size()`
- `frame` 调用 `map::render::format_map_view()` 和 `map::render::render_minimap()`
