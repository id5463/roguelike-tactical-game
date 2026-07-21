# 命令模块 (command)

负责解析用户输入的命令。

## 子模块

### `parser` — 命令解析
- `Command` — 命令枚举（Teleport / Camera / CameraSquad / Help / Save / Load / Position / Quit）
- `CommandParser::parse()` — 将字符串解析为 `Command`

## 通讯方式
- 被 `app::game` 调用，返回 `Command` 枚举供执行
