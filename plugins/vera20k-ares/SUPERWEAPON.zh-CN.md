# 超级武器通用目标配置

组件 0.39.0（配置版本 28、状态版本 1）把通用目标配置接入普通插件版引擎。需要配套支持 `SuperWeaponDeclaration.common_targeting`、`influence`、`eva` 及 `SuperWeaponResourcePolicy.techno_influence` 的引擎；API v1 和 WIT 名称没有变化，不能据此认定任意旧引擎都支持新增字段。无需在编译时启用内置 `ares`。

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

已接入的核弹和自定义处理器共用发射完成流程，`UseChargeDrain=yes` 也会在成功激活后依次处理激活提示、费用、公共动画／`SW.Sound` 和配置的 Ready 等待语音清理。停止放电或激活失败不执行此流程。侧栏和 AI 保留各自的余额检查；已排队的放电发射按 Ares 原顺序在成功后结算，余额不足时收取现有现金。此处不代表其他原版重定向处理器或完整 Ares 已完成。

| 字段 | 默认值与含义 |
| --- | --- |
| `SW.RangeMinimum` / `SW.RangeMaximum` | 默认 -1，以格为单位；负值禁用对应边界，有限有效边界含端点 |
| `SW.FireIntoShroud` | 默认 yes；no 时限制玩家的迷雾目标 |
| `SW.RequiresTarget` / `SW.RequiresHouse` | 默认 none；手动目标的内容和归属筛选 |
| `SW.AIRequiresTarget` / `SW.AIRequiresHouse` | 未设置时使用当前目标模式的默认值，独立于手动筛选 |
| `SW.AITargeting` | 首次初始化时由处理器设定，核弹默认 Nuke，GenericWarhead 默认 Offensive |
| `SW.AITargeting.Preference` | 未显式配置时由模式决定；可设 none、offensive、defensive |
| `SW.UseAITargeting` | 默认 no；yes 时点击使用 AI 选点入口 |
| `SW.AutoFire` / `SW.ManualFire` | 默认 no / yes；AutoFire=yes 时允许玩家所属国家自动发射，ManualFire=no 仅在同时启用 AutoFire 时限制手动点击 |

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

这仍不是完整 Ares 超级武器支持。Stealth、Self、Base、MultiMissile、EnemyBase、DropPod、LightningRandom 等目标模式尚未实现；次级防御预选坐标、升级建筑挂载和核弹 Light／PreImpact 以及其他处理器专用 EVA 等后续链仍待完成。GenericWarhead 的越界落点安全检查和 UnitDelivery 的接纳边界也未因本轮通用射程接入而变成完整原版复现。完整范围见[兼容台账](../../compatibility/README.zh-CN.md)。

旧组件保留自身的配置读取和缺省／提前检查路径；共用宿主判定的修正也会作用于旧组件，不承诺跨引擎版本的逐行为一致。配套宿主现已保留逐槽规则、实际构造、生产／计数、电力和同build恢复中的类别＋原生序号，覆盖这些流程的跨类同名及同类存储名截断重名；其它名称寻址资源、升级和完整侧栏保留链接仍需分别处理。指定器／抑制器的共同核弹和 GenericWarhead 目标入口现可使用；其它处理器覆盖和完整目标链仍未完成。

## 指定器与抑制器

例如核弹只能选择己方侦察兵附近的区域，敌方干扰站附近禁止选点：

```ini
[NukeSpecial]
SW.Designators=E1
SW.Inhibitors=GAPILL

[E1]
DesignatorRange=8

[GAPILL]
InhibitorRange=5
```

这些字段写在原有规则、模式或地图类型节里。名称在各次真实读取时解析，未注册的类型不会被补造；后层新分配类型不会追补前层已失败的引用。列表保留顺序和重复项，逗号之间不加空格；缺失或整体空白保留旧列表，`none` 或仅逗号的非空值清空列表。

`SW.AnyDesignator=yes`／`SW.AnyInhibitor=yes` 默认均为 no，表示任意符合资格的类型，仍受归属、存活和范围限制。指定器只认自己所属国家；抑制器排除自己及自己单向认定的盟友。建筑抑制器还需实际电力／在线资格，指定器不要求电力。进入 limbo、停用、死亡、占领和电力变化会在下一次目标查询中反映，不维护第二份世界状态。

两个范围默认未设置，回退到当前实际类型的原始 `Sight`，不受视野遍历的 10 格上限限制。单位为格，边界包含端点；显式 0 或负数使该对象不贡献范围。后层无效或缺失值保留前层有效值。不要使用极端整数作为正常配置；底层保持原始 32 位平方溢出语义。

相同存储名称的两个实际类型仍分别保存范围和成员身份；名单中的名称只在读取时按原生构造顺序选中首个匹配。附加超武的建筑提供者也按实际 Building 序号保存，后分配的同名建筑不会覆盖另一类型的列表。

本轮接入共同预览与普通 AI 候选筛选，不改变已经提交的发射事件进入 Activate 的阶段边界；偏好／团队／NoTarget 等既有旁路保留。旧组件及旧非空配置不会被普通 Configure 自动升级，新完整存档仍需原始引擎和组件配套。其它 Ares 超武处理器自有覆写、异常国家索引、完整呈现和所有目标模式仍待分别完成。


## 超级武器 EVA 提示

0.39 把通用 EVA 配置放入独立的 `features/superweapons/eva.rs`；公共导入仍通过 `support/host.rs`。沿用实际规则、模式和地图读取顺序，不在运行中重新读 INI。

```ini
[NukeSpecial]
EVA.Ready=EVA_MyNukeReady
EVA.Activated=EVA_MyNukeLaunched
EVA.Detected=EVA_MyNukeDetected
EVA.Impatient=EVA_MyNukeWait
EVA.InsufficientFunds=EVA_InsufficientFunds
EVA.SelectTarget=EVA_SelectTarget
```

自定义事件要先在作品固定的 `evamd.ini` 的 `[DialogList]` 登记，并配置对应阵营的音频；这里只引用事件 ID，不会从名称创建语音。核弹首次映射时默认 Ready／Activated／Detected 分别为 `EVA_NuclearMissileReady`、`EVA_NuclearMissileLaunched`、`EVA_NuclearSiloDetected`；Impatient 默认静音，另外两项默认 `EVA_InsufficientFunds` 和 `EVA_SelectTarget`。缺少对应事件时默认静音。

后层缺失、空值或未知事件保留先前引用。`none`／`<none>` 可显式静音；固定目录中若恰好登记了同名事件，则优先引用该事件。首次没有可读取节也会消耗初始化，不能在后层补做处理器默认值；改写 Type 不会重做已经执行的初始化。

侧栏在充能或暂停状态不允许发射时使用 Impatient，资金不足使用 InsufficientFunds；AutoFire／ManualFire 的原有拒绝顺序仍保留。成功进入鼠标选点时使用 SelectTarget，立即发射和右击不触发它。

成功发射时播报 Activated，公共呈现结束后清除本地玩家该超武配置的 Ready 等待项；正在播放的语音继续播放。远端发射不会清除本地 Ready，Activate 失败也不会播报 Activated 或取消 Ready。建筑落成及被占领后的发现路径共用首个可用的主／次／附加超武查询，监听者的盟友、战败和游戏模式条件保持原入口所有权。

Ready／Activated／Detected 本批贯通核弹、GenericWarhead 和 UnitDelivery 的现有公共处理器；其他原版重定向处理器仍使用原有消费者，其专用标志、源超武替换和完整生命周期尚待迁移。不要把能读取字段等同于所有处理器都已支持。API v1／WIT 名称不变，新可选声明仍要求配套宿主；存档保留精确组件和对应引擎。
