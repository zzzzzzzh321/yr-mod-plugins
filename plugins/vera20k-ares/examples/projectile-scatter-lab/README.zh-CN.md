# 抛射体散布实验地图

[projectile-scatter.map](projectile-scatter.map) 是本工程的自制实验图，沿用旧 SDK 实验图的 48×48 平地、矿石字母 A—F 和房屋配置，重新设置了六组散布武器。地图只引用玩家本机的 YR 原版图像、炮口和命中动画，不附带原版或心灵终结素材，也不修改旧实验图。

使用 **0.32.0／配置22／状态1** 的 `vera20k-ares.wasm`、同目录 `vera-mod.json` 和本批配套引擎。虽然 API 仍为 v1，旧宿主不一定支持新增的可空 Leptons 读取服务和散布声明。

## 启动与对照

复用 [SDK 实验图启动脚本](../sdk-lab/play.py) 和 [spawn.ini](../sdk-lab/spawn.ini)，不需要另一个启动器。从插件工程根目录执行，先修改引擎与原版资源路径：

```powershell
$engine = "C:/path/to/vera20k.exe"
$game = "D:/steam/steamapps/common/Command & Conquer Red Alert II"
$launchArgs = @(
  "--engine", $engine,
  "--game-dir", $game,
  "--component", "dist/release-0.32.0-ballistic-scatter/vera20k-ares.wasm",
  "--map", "plugins/vera20k-ares/examples/projectile-scatter-lab/projectile-scatter.map",
  "--spawn", "plugins/vera20k-ares/examples/sdk-lab/spawn.ini",
  "--mode", "compatible",
  "--windowed"
)
python plugins/vera20k-ares/examples/sdk-lab/play.py @launchArgs
```

先加 `--dry-run` 可只检查启动资源包。Linux、macOS 使用相同的 Python 脚本与参数，换成对应平台的配套引擎路径；本文没有声明已做跨平台运行验收。

| `--mode` | 本图中的用途 |
| --- | --- |
| `compatible` | 加载0.32组件，使用各抛射体的散布设置 |
| `vanilla` | 不提供组件；同一引擎使用全局散布，忽略两个扩展标签 |

将参数数组中的 `compatible` 改为 `vanilla`，重新开局比较。请显式填写模式；脚本默认的 `overload` 不用于本图。本图关闭护盾和过载，强行选择 `overload` 会被组件依赖检查拒绝。

脚本把传入地图作为内存资源 `sdk-feature-lab.map` 提供，因此沿用上述 spawn 文件即可。它还提供匹配的组件与摘要；只复制地图不能证明 Wasm 已加载。每次启动使用独立运行目录，不向原版资源目录写文件。

## 六组观察区

每组一辆可控制的灰熊外观试射车，右上方是一辆不还击的犀牛靶车。用矿石字母与坐标区分，保持车辆在出生点，手动攻击本组靶车。按 `S` 停火，重开本局重置。

六组武器的基础参数相同，仅抛射体的散布声明和 F 组的 `FlakScatter` 不同。各投射物都有 `Inaccurate=yes`、`Arcing=yes`、`Inviso=no`；全局 `[CombatDamage] BallisticScatter=1.0`。

| 组 | 试射车／靶车坐标 `(X,Y)` | 配置 | 0.32组件下的观察目标 |
| --- | --- | --- | --- |
| A | `(38,46)`／`(38,41)` | plain，两项都省略 | 继续使用全局默认幅值0.5—1格，作为基准 |
| B | `(46,38)`／`(46,33)` | plain，Min=0、Max=0 | 散布幅值为0，应比 A 集中；有效0不会当作未配置 |
| C | `(45,53)`／`(45,48)` | plain，只设 Min=0 | Max仍取全局1格，观察0—1格的幅值 |
| D | `(53,45)`／`(53,40)` | plain，只设 Max=3.0 | Min仍取全局的一半，观察0.5—3格的幅值 |
| E | `(52,60)`／`(52,55)` | plain，Min=2.0、Max=3.0 | 观察较大的散布，部分炮弹会明显偏离靶车 |
| F | `(60,52)`／`(60,47)` | flak，Min=2.0、Max=3.0 | 输入幅值与 E 相同，但还按实际目标距离／武器射程缩放 |

上表是幅值抽取规则，不保证画面落点严格等于某个像素圆；F 尤其不能直接把2—3格当作最终落点半径。比较多发的分布，不要求两个独立对局的第 N 发落在同一像素。B 的零幅值仍保留角度抽取，不能据落点集中推断没有消耗随机数。

`vanilla` 模式中，A—E 都回到 plain 的全局默认散布；F 使用原版 flak 默认幅值及距离缩放。先比较 B 与 A，再比较 D、E 与 F，比较容易看出设置是否生效。

## 伤害、资源和配置

靶车生命20,000、速度0且没有武器；试射车不自动索敌、不升级，使用无限弹药，便于连续观察。武器伤害30、ROF45、射程10格，弹头使用 `CellSpread=1.0` 和原版 `S_CLSN16`／`S_CLSN22` 命中动画。`Inaccurate=yes` 保留原本不直接吸附目标的行为，因此不能只用低散布推断必定直接命中；范围伤害让近处爆炸仍可伤到靶车。大幅值时不掉血可能是落点离开了伤害半径。

配置位置：`SCAT_A`—`SCAT_F` 是试射车型；`SCAT_GunA`—`SCAT_GunF` 是武器；`SCAT_ShotA`—`SCAT_ShotF` 是抛射体；`SCAT_WH` 是共用弹头。所有新资源名均短于24字节。车辆使用已存在的 `GTNK` 外观与完整炮塔配置，靶车直接使用原版 `HTNK`；不另配美术包。

字段定义见 [Ares 官方散布说明](https://ares-developers.github.io/Ares-docs/new/projectiles/ballisticscatter.html)。本图不演示 Airburst／Splits、护盾、过载、航程耗尽或 `Inviso + FlakScatter` 的额外散布。

## 验证范围

本图的静态 INI 结构、对象／武器／抛射体引用与启动资源包已检查；配套普通 fast 引擎已用本机 YR 原版资源完成本图的地图／资源加载检查。该加载命令不加载 Wasm，不能代替正式组件下的 GUI 发射与落点观察。散布功能另有真实组件经过普通开火、飞行、伤害、退休和保存恢复的生产回归；本文不声称已完成本图 GUI、非零散布整场景或多人联机玩测，也不代表完整 Ares 兼容。
