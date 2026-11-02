# 手动操作限制

在规则或地图的 `[VeraFeatures]` 节设置 `AresManualControl=yes`，再在对象类型节使用下列 Ares 标签。默认值均为 `no`。代码集中于 [units/manual_control.rs](src/features/units/manual_control.rs)，所有功能共用 Host。

| 标签 | 作用范围 | 效果 |
| --- | --- | --- |
| `NoManualFire=yes` | 作战对象 | 禁止玩家直接指定攻击目标；自动索敌与反击保留 |
| `NoManualUnload=yes` | 车辆、飞机 | 隐藏自点卸载操作，并限制车辆的卸载热键；不改变摧毁时的乘客处理 |
| `NoManualEnter=yes` | 作为目标的运输工具 | 隐藏普通进入操作；脚本、AI与模拟层运输命令仍可使用 |
| `NoSelfGuardArea=yes` | 步兵 | 禁止对自身点击产生的治疗类区域警戒操作 |

这批功能只返回加载时冻结的输入策略。弹药计数、运输状态、任务队列依然由引擎原有模块管理。它们不会通过模拟层的统一命令过滤器误伤 AI、脚本或联机回放。

按安装版 Ares 的实际控制流，飞机的 D 键绕过这项限制，本插件保留该差异。车辆的转建筑、简单部署与坦克碉堡逃出按各自原有分支判断。`NoManualEnter` 跳过普通乘客操作后仍保留建筑入口后继，例如回收厂；尚未外置 Ares 隧道入口。治疗步兵的单选自点已接入，混合/多选治疗步兵的完整逐对象操作仍待补齐，因此兼容台账仍记 `partial`。

布尔值使用宿主已有的 Ares 读取器：后续规则层缺失、空值或无效值保留之前的有效值，明确 `no` 则关闭。存档保存组件与冻结配置，读取存档后重新绑定该配置，不能用当前磁盘 INI 覆盖旧存档的策略。

参见 [可复制配置](examples/manual-control.ini) 和 [Ares 官方说明](https://ares-developers.github.io/Ares-docs/new/manual.html)。本页列出的是本批接入范围；整体兼容进度见[台账](../../compatibility/README.zh-CN.md)。
