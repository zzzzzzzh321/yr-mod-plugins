# 外置 Ares 弹药、装填与武器选择

插件源码独立位于 `plugins/vera20k-ares`，发布文件名为 `vera20k-ares.wasm`；SDK基础设施保留在 `sdk`。统一参考组件0.7.0在新对局安装后默认启用下列三项Ares弹药链功能，直接使用原Ares标签，无须额外开启或传构建参数：

| `[VeraFeatures]`可选关闭项 | 配置位置与标签 | 插件提供的策略 |
| --- | --- | --- |
| `AresAmmo=no` | 武器节`Ammo` | 每次开火的弹药成本 |
| `AresReload=no` | 对象类型节`ReloadAmount`、`EmptyReloadAmount` | 普通和空仓时的装填数量 |
| `AresNoAmmoWeapon=no` | 对象类型节`NoAmmoWeapon`、`NoAmmoAmount` | 当前弹量达到阈值时选用的武器槽 |

三个模块与`AresManualControl`均默认启用，可在`[VeraFeatures]`中分别设为`no`关闭，也可与显式开启的护盾组合。模块开启不改变各标签的原有默认值。引擎保留弹仓、装填时钟与武器选择流程；插件只解释扩展键，返回加载期冻结的静态策略。无组件、缺少可选回调或关闭对应开关时，不发布该项外置策略；引擎内置`ares`构建特性的行为不由这些开关统一关闭。

## 每次开火成本

下面直接配置武器成本，其他Ares模块保持默认启用；完整片段见[examples/ammo.ini](examples/ammo.ini)：

```ini
[MTNK]
Ammo=5
InitialAmmo=5
Reload=120
Primary=SDKAmmoGun
ElitePrimary=SDKAmmoGun

[SDKAmmoGun]
Damage=20
ROF=30
Range=6
Speed=100
Projectile=InvisibleLow
Warhead=SA
Ammo=2
```

单位节的 `Ammo` 是弹仓容量，武器节的 `Ammo` 是一次开火成本。上例五发先变三发，再变一发；剩余一发时等待装填。完整示例见 [examples/ammo.ini](examples/ammo.ini)。

- 默认成本 1，后层缺失、空值或读取失败保留先前有效值，使用引擎已有 Ares 整数读取器。
- 零或负成本允许空弹仓开火，不扣弹，不因射击重启装填；负成本不会补充弹药。
- 成本策略不扣减当前为负的弹量；这不会把有限类型的容量改成无限类型，低弹量选择仍按下文的有符号条件判断。
- 支持车辆、步兵和建筑。建筑和步兵复用通用 Reload；保留建筑 Hospital/Armory 排除条件。
- 按 [官方说明](https://ares-developers.github.io/Ares-docs/new/weapons/ammo.html)，不应用于飞机，保留其独立投弹与返航流程。本版宿主会拒绝插件返回的飞机资源策略，避免只改变开火门槛却未改变独立扣弹流程。
- 本例未配置低弹量武器槽或装填数量，保持索引-1与普通装填量1。自0.5.0起，`AresAmmo`组合同时提供`use_weapon_rof`策略，略过原版建筑当前弹量大于1时的单帧重装分支，使用原有武器ROF、连发、倍率和随机变化计算。它是 Ares 安装补丁的兼容接入，不是武器 `Ammo` 标签新增的原生含义；0.7.0安装组件后无需额外标签即启用该已实现修复。

自0.5.0起，组件需要同时支持`use_weapon_rof`和可选`input`策略的引擎；高槽建筑延迟射击还需要下文所述配套宿主。新引擎读取旧组件省略的字段时按false/None处理，保留旧组件原行为；新组件返回这些字段时，旧引擎会拒绝未知策略。0.7.0保持配置版本5、状态版本1、API v1及WIT接口不变，只改变新对局初始功能选择。保存旧组件与配套引擎，更新组件后开启新对局。

`AresReload` 单独启用仍只选择装填数量，不能据此宣称已经包含 Ares 的全部安装默认修复。完整兼容的剩余范围见[逐项台账](../../compatibility/README.zh-CN.md)。

## 普通与空仓装填数量

安装组件后，直接在单位类型节使用；若曾显式设置`AresReload=no`，需恢复为`yes`：

```ini
[LTNK]
Ammo=6
InitialAmmo=6
Reload=30
EmptyReload=90
ReloadAmount=2
EmptyReloadAmount=3
```

`Ammo`是容量，`InitialAmmo`是初始值，`Reload`和`EmptyReload`是原版装填间隔，新增两项是一次装填恢复的数量。数量与间隔是两组独立判断：即使未设置`EmptyReload`，当前弹量**恰好为0**时仍使用`EmptyReloadAmount`。

- `ReloadAmount`默认1，允许负数。
- `EmptyReloadAmount`未设时使用普通装填量；值为0或负数时，空仓无法由该装填过程恢复。
- 原装填时钟到期、满足原有装填资格后，先按32位有符号整数回绕相加；和为负数则写0，否则再按有符号比较夹到类型容量。不能用普通饱和加法替换，也不能交换这两个分支。
- 继续使用原弹仓所有者、装填时钟、移动/暂停条件与下一次装填调度；不影响武器ROF，也不创建插件自己的弹药计数。
- 支持车辆、步兵与普通建筑；保留Hospital/Armory建筑例外。飞机停靠与无线电补弹不应用这项数量策略。
- 读取使用现有Ares整数读取器；后层缺失、空值或转换失败保留先前有效值，不采用Phobos的显式清空约定。

官方语义见[ReloadAmount / EmptyReloadAmount](https://ares-developers.github.io/Ares-docs/new/reload.html)。原始Ares构造、读取和装填hook地址保留在[reload.rs](src/features/weapons/reload.rs)及引擎消费代码旁。

## 低弹量时选用指定武器

安装组件后，直接在对象类型节使用；若曾显式设置`AresNoAmmoWeapon=no`，需恢复为`yes`：

```ini
[LTNK]
NoAmmoWeapon=1
NoAmmoAmount=1
```

- `NoAmmoWeapon`默认-1；**任意负数**都关闭此覆盖，任意非负32位有符号整数原样交给宿主，不在插件配置阶段夹取或拒绝。0选主武器，1选副武器，2..17复用宿主已有武器数组与精英武器回退；例如索引2对应`Weapon3`/`EliteWeapon3`。
- `NoAmmoAmount`默认0。类型容量必须`Ammo>=0`，并且当前弹量`<=NoAmmoAmount`才满足条件。它按弹量比较，**不按主武器每发成本判断是否够用**。
- 比较保留有符号语义：类型容量为有限值，但实例当前弹量为负数时，也可能满足阈值。类型`Ammo=-1`则不应用此选择策略。
- 插入共享选择器中Ares对应的阶段，保留DeployFire、非Gattling的炮手当前武器、驻军建筑、主副武器缺失、副武器`NeverUse`、空目标和敞篷指定武器等更早分支优先级。选定槽后仍走已有武器解析和CanFire检查；空槽或不能开火都不会自动遍历其他槽。
- `WeaponN`/`EliteWeaponN`只在`TurretCount>0`的规则读取分支填充。非Gattling炮手在进入此覆盖前就返回当前武器槽，因此使用更高槽索引的类型通常还需要`IsGattling=yes`。`WeaponCount`限制配置读取循环，不能在运行选择时重新截掉之前规则层保留的槽。
- 有效数组范围是0..17。索引>=18仍原样发布，但宿主的既有安全检查返回无武器；原DLL会越界读取，可能出错。本工程保留此安全差异，不把它描述成原DLL拒绝了配置。
- 这项选择策略可以用于飞机；它与不支持飞机的武器成本/装填数量策略分开。飞机不能靠武器`Ammo=0`获得这里的免费开火策略，其释放、返航和停靠补弹仍按原机制。飞机接近、扫射和部分ROF/弹道仍按原逻辑读取武器0，不能因本策略选择了其它槽就统一改写。
- 若要在空仓时让备用武器继续开火，需保持`AresAmmo`启用（新对局默认值），并把备用武器节的`Ammo=0`。仅选择副武器不会免除其弹药要求。

0.6.0移除了旧版组件仅允许-1/0/1的人工边界。原始Ares读取器`1003EF01 -> 1000A570`保存整数，选择hook`10047AE0`中的`10047B08..10047B1D`只测试索引符号，再原样返回非负索引。官方说明仍只举主副武器；此处明确区分文档所列常见值和安装DLL的真实接纳行为。

建筑动画延迟开火需要配套新版宿主，在既有延迟状态、保存与恢复中保留完整索引。旧宿主只保留主副两值时，高槽可能错误变成主武器。0.6.0引入的这项支持不扩展SDK字段，0.7.0沿用配置版本5和状态版本1；不能仅凭API版本相同就使用旧宿主。高槽与建筑延迟枪的实际组件/宿主联合验证仍以[兼容台账](../../compatibility/README.zh-CN.md)为准。

官方说明见[NoAmmoWeapon](https://ares-developers.github.io/Ares-docs/new/noammoweapon.html)。对应配置模块为[empty_weapon.rs](src/features/weapons/empty_weapon.rs)。

三项默认Ares弹药机制及全部四个类型扩展键的组合示例见[resource-cycle.ini](examples/resource-cycle.ini)。示例使用LTNK与两份完整武器定义，普通武器成本2、备用成本0。实际射击可与装填交错，具体弹量序列取决于原装填和开火时钟；示例不另行重置这些时钟。

## 代码与接口

源码按领域与机制整理，所有模块共用[support/host.rs](src/support/host.rs)中的Host与配置读取包装：

| 模块 | 责任 |
| --- | --- |
| [features/weapons/ammo_cost.rs](src/features/weapons/ammo_cost.rs) | 解释武器成本及开火后装填请求 |
| [features/weapons/reload.rs](src/features/weapons/reload.rs) | 解释普通/空仓装填数量 |
| [features/weapons/empty_weapon.rs](src/features/weapons/empty_weapon.rs) | 解释低弹量武器槽和阈值 |
| [features/defense/shield](src/features/defense/shield/mod.rs) | 护盾的配置、持续状态、伤害与恢复 |
| [SDK resource.rs](../../sdk/api/src/resource.rs) | 引擎和插件共用的静态资源查询与策略类型 |

实际弹药状态、扣减、装填时钟和最终武器选择继续由引擎已有所有者负责。

可选回调 `weapon_resource` 接收武器名称，返回 `WeaponResourcePolicy`：`cost` 为成本，`reload_after_use` 为射击后装填请求，`users` 为适用类别位掩码。

最终配置阶段通过只读 `registry("$weapons")` 枚举引擎实际分配的武器节（保留精确大小写），使用 `ares-int` 读取器读取成本，并冻结在组件配置中。新增回调只查询冻结配置，不读运行时 INI，不写对象状态。

可选第六回调`type_resource`接收`TypeResourceContext { type_name, user }`，返回`TypeResourcePolicy`。其中`reload`包含`normal_amount/empty_amount`；`depleted_weapon`包含`weapon_index/threshold`。插件按实际类别排除飞机装填，保留飞机武器选择。该查询使用已有`evaluate`消息通道，没有新增Host服务。

Rules接纳时查询各武器与对象类型，绑定只读派生策略，热路径不跨Wasm。AresReload可独立赋予适用类型通用装填数量策略；仅启用武器成本时，装填资格继续按对象实际普通、精英和驻军武器派生，局部策略不会改变无关对象。某类别的全部武器都被策略覆盖时，也允许该类别的无武器对象使用通用装填，以兼容脚本开火。局部成本策略搭配脚本指定未绑定武器时，应同时为对象绑定的武器提供成本策略。

读档从保存的组件配置重建武器与类型投影，不重新解释当前作品INI。这些策略没有独立对象状态，不需要第二套状态容器。原有四/五回调组件不会收到未声明的查询，可保留自身原有行为。

## 构建与测试

从独立的 `plugin-dev` 工程根目录执行：

```powershell
cargo build --locked -p vera20k-ares --target wasm32-unknown-unknown --release
```

按 [中文 SDK README](../../sdk/README.zh-CN.md) 将 `target/wasm32-unknown-unknown/release/vera20k_ares.wasm` 打包到新目录，并传入 `--program vera20k-ares.wasm`。设置外部 `CARGO_TARGET_DIR` 时使用该目录中的实际输入路径。发布内容为 `vera20k-ares.wasm` 加 `vera-mod.json`；通用打包器省略该选项时仍使用默认名 `gameplay.wasm`。

只需要护盾时设置`Shield=yes`；还需要自定义过载时，在同一个`[VeraFeatures]`节增加`Overload=yes`。两项仍默认关闭，过载依赖护盾。发布文件名与内部组件标识分别维护。所有组合继续使用同一`vera.reference-gameplay`组件，版本0.7.0，config_version=5，state_version=1。四个Ares模块只对新对局默认启用，选择随配置冻结。手动操作开关`AresManualControl`及其独立配置见[手动操作说明](MANUAL.zh-CN.md)。

旧的非空冻结配置不补入新默认值，旧组件字节继续解释旧配置。读档还要求当前作品的组件摘要与存档完全一致；即使存档保存了Wasm字节，也不能直接用新版资源接管。目录迁移和重新编译会改变组件摘要，应保留原组件及其声明供旧存档使用。

新引擎验收模式 `ammo` 使用兼容护盾加弹药组件，报告文件必须为新文件，父目录已存在：

```powershell
vera20k.exe --mod-sdk-probe --retail-dir "原版游戏目录" --map XMP03T4.MAP --mode ammo --module "组件目录/vera20k-ares.wasm" --output "报告目录/ammo.json"
```

`ammo`模式通过INI同时开启护盾和武器成本；`ammo-only`只开启武器成本。两者使用同一个组件。该模式检查护盾伤害链、原版地图加载、实际命令与开火产生的5→3→1弹药变化、余量不足停止开火及存档后策略绑定；它本身不代表装填数量和低弹量选择新链已经验收。

```powershell
cargo test --locked -p vera20k-ares --lib
```

以下实际 Wasm 集成测试从相邻的引擎仓库根目录执行，属于运行验收而非插件编译。设置组件的绝对路径和原版资源路径；提取的INI与原版档案资源分别设为必需，避免缺少档案时测试跳过实际对局部分：

```powershell
$env:VERA_SDK_AMMO_WASM = "组件目录/vera20k-ares.wasm"
$env:RA2_DIR = "原版游戏目录"
$env:VERA20K_REQUIRE_RETAIL_INI = "1"
$env:VERA20K_REQUIRE_RETAIL_ASSETS = "1"
python -m tools.cargo_run -- test --locked -p vera20k-core --lib sdk_resources_ -- --include-ignored
python -m tools.cargo_run -- test --locked -p vera20k-core --lib sdk_ammo_ -- --include-ignored
python -m tools.cargo_run -- test --locked -p vera20k-core --lib sdk_resource_policy_ -- --include-ignored
```

`sdk_resources_`检查装填数量、存读档、阈值切换、强制武器仍受弹药成本约束，以及仅启用装填时步兵/建筑开火后的时钟。免费射击与装填扩展同时启用时，也检查原有装填时钟不被射击重置。

示例不含游戏资源，沿用已有 Rust 实现和原始 Ares 指令引用。其他桌面平台、真实联网及大规模性能仍需单独验证。

### 前一版完整存档的验收

0.7 配套引擎另提供 `--mode ares` 验收：临时地图完全不写 `[VeraFeatures]`，使用原有 Ares 弹药、装填、换武器及手动操作标签；同时核对未写标签的普通武器与建筑策略，并经过实际开火和存读档入口。其他隔离模式会明确选择全部模块，避免依赖默认值：

```powershell
vera20k.exe --mod-sdk-probe --retail-dir "原版游戏目录" --map XMP03T4.MAP --mode ares --module "组件目录/vera20k-ares.wasm" --output "新报告.json"
```

配套引擎的 `--mod-sdk-save-probe` 可以读取前一版实际生成的完整存档，检查大厅存档列表的兼容判定、资源载荷、模拟状态恢复及冻结组件保留。报告必须使用新文件；此检查不启动窗口，也不替代原版资源重新绑定或实际游玩验收：

```powershell
vera20k.exe --mod-sdk-save-probe "原版游戏目录" "上一版生成的存档.vsave" "新报告.json"
```

0.7 配套引擎允许在该命令末尾追加当前作品的 Wasm 路径，进一步经过正式的保存程序重绑定入口。相同旧组件可恢复原冻结策略；换成新组件摘要会拒绝接管旧档。这项检查仍不覆盖常驻美术资源重绑定或 GUI。

0.6.0 配套宿主只为已核对的 0.5.0 原版能力构建存档增加格式兼容转换。旧主／副武器的延迟开火数据与新版索引字段字节相同；存档入口和列表共用同一个兼容判定。未知格式仍拒绝，旧存档继续使用保存的原组件与冻结配置，不会自动切换到新版插件。带开发测试字段的构建使用另一种存档结构，不能拿它替代发布构建进行此项测试。
