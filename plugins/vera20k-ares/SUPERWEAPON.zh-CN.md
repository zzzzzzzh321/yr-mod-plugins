# 超级武器通用目标配置

组件 0.37.0（配置版本 26、状态版本 1）把通用目标配置接入普通插件版引擎。需要配套支持 `SuperWeaponDeclaration.common_targeting` 的引擎；API v1 和 WIT 名称没有变化，不能据此认定任意旧引擎都支持新增字段。无需在编译时启用内置 `ares`。

以已有核弹为例，在其超级武器节中添加：

```ini
[NukeSpecial]
SW.RangeMinimum=2
SW.RangeMaximum=20
SW.FireIntoShroud=no
SW.RequiresTarget=buildings
SW.RequiresHouse=enemies
```

玩家选定目标时，宿主使用插件冻结的配置，检查目标、迷雾和可用发射井。实际发射复用引擎原有的充能、建筑顺序、弹道和伤害逻辑。不会每帧重新解析 INI，也不需要再次调用 WASM 来枚举目标。

| 字段 | 默认值与含义 |
| --- | --- |
| `SW.RangeMinimum` / `SW.RangeMaximum` | 默认 -1，以格为单位；负值禁用对应边界，有限有效边界含端点 |
| `SW.FireIntoShroud` | 默认 yes；no 时限制玩家的迷雾目标 |
| `SW.RequiresTarget` / `SW.RequiresHouse` | 默认 none；手动目标的内容和归属筛选 |
| `SW.AIRequiresTarget` / `SW.AIRequiresHouse` | 未设置时使用当前目标模式的默认值，独立于手动筛选 |
| `SW.AITargeting` | 首次初始化时由处理器设定，核弹默认 Nuke，GenericWarhead 默认 Offensive |
| `SW.AITargeting.Preference` | 未显式配置时由模式决定；可设 none、offensive、defensive |
| `SW.UseAITargeting` | 默认 no；yes 时点击使用 AI 选点入口 |
| `SW.AutoFire` / `SW.ManualFire` | 默认 no / yes；AutoFire=yes 时允许玩家所属国家自动发射，ManualFire=no 限制手动点击 |

例如只让核弹自动选择目标、禁止玩家手动点击：

```ini
[NukeSpecial]
SW.AutoFire=yes
SW.ManualFire=no
SW.UseAITargeting=yes
```

自动选择仍要满足目标模式的条件，不保证一充满就能发射。目标内容名称为 `land,water,empty,infantry,units,buildings,all,none`；归属名称为 `owner,allies,enemies,team,trump,others,all,none`。逗号之间不要添加空格。`building` 等别名不属于这套读法。

配置按原有规则、模式和地图的物理读取顺序保留。后层缺失或无效值保留早先有效值；一个文件里请只写一个相同名称的节。百分数沿用 Ares double 读法，例如 `500%` 得到 5；正常配置建议直接填写格数。底层保留 Infinity/NaN 的原始比较语义，不把它们统一当作非法范围。

预览和发射事件属于不同阶段：原 Ares 的已提交事件会进入处理器的 Activate，不重新进行整套鼠标目标检查。核弹在此阶段没有符合射程的发射井时，可能直接投放。不要用手工构造无效发射事件来代替界面的目标选取。

首次到达的类型处理器只初始化一次。后层改写 `Type` 不代表重新建立整套处理器默认值；首次没有可读取的类型节时，也不会在后层补做首次初始化。旧的非空冻结配置缺少新增字段时保持旧行为；读档需要保存原引擎和摘要相同的组件配套。

GenericWarhead 的空弹头不会让通用预览拒绝一个有效目标；它会在 Activate 失败。此时 ClickFire 消耗充能，但不扣费、不记成功发射，也不产生伤害。旧组件缺少新增声明时保留原先的提前拒绝行为。

这仍不是完整 Ares 超级武器支持。Stealth、Self、Base、MultiMissile、EnemyBase、DropPod、LightningRandom 等目标模式尚未实现；次级防御预选坐标、外置 Designators/Inhibitors、升级建筑挂载和核弹 Light/EVA/PreImpact 等后续链仍待完成。GenericWarhead 的越界落点安全检查和 UnitDelivery 的接纳边界也未因本轮通用射程接入而变成完整原版复现。完整范围见[兼容台账](../../compatibility/README.zh-CN.md)。
