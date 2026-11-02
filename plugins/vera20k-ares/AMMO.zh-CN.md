# 外置 Ares 弹药、装填与武器选择

插件源码独立位于 `plugins/vera20k-ares`，发布文件名为 `vera20k-ares.wasm`；SDK基础设施保留在 `sdk`。统一参考组件0.5.0通过三个独立INI开关选择Ares弹药链功能，无须构建参数：

| `[VeraFeatures]`开关 | 配置位置与标签 | 插件提供的策略 |
| --- | --- | --- |
| `AresAmmo=yes` | 武器节`Ammo` | 每次开火的弹药成本 |
| `AresReload=yes` | 对象类型节`ReloadAmount`、`EmptyReloadAmount` | 普通和空仓时的装填数量 |
| `AresNoAmmoWeapon=yes` | 对象类型节`NoAmmoWeapon`、`NoAmmoAmount` | 当前弹量达到阈值时选用的武器槽 |

三个开关默认`no`，可以独立启用，也可与护盾组合。引擎保留弹仓、装填时钟与武器选择流程；插件只解释扩展键，返回加载期冻结的静态策略。无组件、缺少可选回调或关闭对应开关时，不应用该项扩展。

## 每次开火成本

下面只启用武器成本，完整片段见[examples/ammo.ini](examples/ammo.ini)：

```ini
[VeraFeatures]
AresAmmo=yes

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
- 本例未开启低弹量武器选择与装填数量。0.5.0 的 `AresAmmo` 组合同时提供 `use_weapon_rof` 策略，略过原版建筑当前弹量大于1时的单帧重装分支，使用原有武器ROF、连发、倍率和随机变化计算。它是 Ares 安装补丁的兼容接入，不是武器 `Ammo` 标签新增的原生含义。

组件0.5.0需要同时支持 `use_weapon_rof` 和可选 `input` 策略的新版引擎。新引擎读取旧组件省略的字段时按false/None处理，保留旧组件的原版行为；新组件返回这些字段时，旧引擎会拒绝未知策略。配置版本升至5（新增手动操作配置），状态版本1及WIT导入接口保持不变。保存旧组件与配套引擎，更新组件后开启新对局。

`AresReload` 单独启用仍只选择装填数量，不能据此宣称已经包含 Ares 的全部安装默认修复。完整兼容的剩余范围见[逐项台账](../../compatibility/README.zh-CN.md)。

## 普通与空仓装填数量

开启`AresReload=yes`后，在单位类型节使用：

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

开启`AresNoAmmoWeapon=yes`后，在对象类型节使用：

```ini
[LTNK]
NoAmmoWeapon=1
NoAmmoAmount=1
```

- `NoAmmoWeapon`默认-1，表示关闭；0选主武器，1选副武器。
- `NoAmmoAmount`默认0。类型容量必须`Ammo>=0`，并且当前弹量`<=NoAmmoAmount`才满足条件。它按弹量比较，**不按主武器每发成本判断是否够用**。
- 比较保留有符号语义：类型容量为有限值，但实例当前弹量为负数时，也可能满足阈值。类型`Ammo=-1`则不应用此选择策略。
- 插入共享选择器中Ares对应的阶段，保留DeployFire、敞篷指定武器等更早分支优先级。选定槽后仍走已有武器解析和CanFire检查，不会因为不能开火就自动遍历其他槽。
- 这项选择策略可以用于飞机；它与不支持飞机的武器成本/装填数量策略分开。飞机不能靠武器`Ammo=0`获得这里的免费开火策略，其释放、返航和停靠补弹仍按原机制。
- 若要在空仓时让备用武器继续开火，需同时开启`AresAmmo=yes`，并把备用武器节的`Ammo=0`。仅选择副武器不会免除其弹药要求。
- 本组件严格接纳-1、0、1；其他值在配置阶段明确报错，不默默夹取。上游DLL实际上还允许其他非负槽索引进入后续流程，但官方文档只支持主副两槽；此处是明确的受限配置边界。

官方契约见[NoAmmoWeapon](https://ares-developers.github.io/Ares-docs/new/noammoweapon.html)。对应配置模块为[empty_weapon.rs](src/features/weapons/empty_weapon.rs)。

三个Ares开关及全部四个类型扩展键的组合示例见[resource-cycle.ini](examples/resource-cycle.ini)。示例使用LTNK与两份完整武器定义，普通武器成本2、备用成本0。实际射击可与装填交错，具体弹量序列取决于原装填和开火时钟；示例不另行重置这些时钟。

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

需要护盾或过载时，在同一个`[VeraFeatures]`节增加`Shield=yes`、`Overload=yes`。发布文件名与内部组件标识分别维护。所有组合继续使用同一`vera.reference-gameplay`组件，版本0.4.0，config_version=4，state_version=1。五个开关默认关闭，过载依赖护盾，选择随存档冻结。目录迁移和重新编译会改变组件摘要，旧存档仍需对应的原组件文件。

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
