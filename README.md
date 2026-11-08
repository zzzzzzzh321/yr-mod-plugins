# VERA20K 插件开发工程

本目录是后续外置功能插件的主要维护位置，与 `../engine` 相互独立。当前插件发布文件为 `vera20k-ares.wasm`，已有护盾、过载及 Ares 弹药、装填、低弹量选武器和手动操作限制示例。

从组件 0.7.0 起，新对局安装组件后，`AresAmmo`、`AresReload`、`AresNoAmmoWeapon`、`AresManualControl` 默认启用，已有 Ares 标签可以直接使用；可在 `[VeraFeatures]` 中将对应项设为 `no` 作对照。各标签自身的默认值不变，例如四个手动操作限制仍默认 `no`。Phobos 护盾与自定义过载仍需显式设置 `Shield=yes`、`Overload=yes`，且过载依赖护盾。默认启用只覆盖已外置范围，不表示完整 Ares 兼容。

```text
plugin-dev/
├── Cargo.toml
├── Cargo.lock
├── plugins/vera20k-ares/        # 功能插件源码与 INI 示例
├── sdk/
│   ├── api/                    # 固定版本接口依赖
│   ├── wit/                    # 固定版本通信契约
│   ├── package/                # 组件打包工具源码
│   ├── dev-kit/                # 开发说明与独立工程模板
│   └── source.json             # SDK 快照来源
└── dist/                       # 本地生成的发布文件，不提交
```

编写功能主要修改 `plugins/vera20k-ares/src/features/`；公共宿主调用集中于 `src/support/host.rs`，组合与分发位于 `src/component.rs`。领域与机制的组织方式沿用现有结构，后续超级武器另设领域模块。

0.34配套宿主继续保留 `AirburstWeapon` 的实际Weapon引用，原版／Ares子弹工厂使用对应实例的字段和冻结航程，同名实例不再凭名称重查。Ares子弹自身仍不安装Weapon引用。此增量复用已有实例目录和策略，组件配置保持config23／state1，不增加接口或配置字段；其它资源家族与工厂仍以台账中的限制为准。

0.33 开发候选将武器配置绑定到原生 WeaponType 数组身份，而不是按截短后的名字合并。新增可选 `weapon_instance_resource` 一次返回弹药与开火策略；Ammo、ApplyDamage、ProjectileRange 均从该实例的保留读取来源取得，解析和数值转换仍由宿主统一完成。不同实例即使存储名相同，也可保留不同的读取历史、构造默认值和冻结策略。

新身份域只用于新配置与真实重建边界，历史非空配置缺域时保持原有行为。旧 `weapon_resource`／`weapon_firing_resource` 请求形状保留；旧组件在唯一武器名上继续使用原协议，同名多实例无法凭名字区分时应由配套宿主明确拒绝绑定。组件为 config23／state1／API v1，WIT 通用导入不变，须配套支持实例目录和读取器的新宿主。该增量不代表 Projectile／Warhead 等其他资源家族已经完成实例迁移；实际验收范围以兼容台账为准。

## 构建与发布

安装 Rust 工具链后，在本工程根目录执行：

```sh
rustup target add wasm32-unknown-unknown
cargo test --locked -p vera-mod-api -p vera-mod-pack -p vera20k-ares --lib
cargo build --locked --release --target wasm32-unknown-unknown -p vera20k-ares
```

创建 `dist` 文件夹，然后打包到尚不存在的新子目录：

```sh
cargo run --locked --release -p vera-mod-pack -- target/wasm32-unknown-unknown/release/vera20k_ares.wasm dist/release-new --program vera20k-ares.wasm
```

输出 `vera20k-ares.wasm` 和匹配的 `vera-mod.json`，两者一起作为作品资源交给引擎。设置了 `CARGO_TARGET_DIR` 时，需将打包输入改成该目录中的实际 Wasm 路径。

编译与插件单元测试只需要本工程、SDK 和 Rust；运行与游戏验收使用兼容的现成引擎及游戏资源。完整说明见[独立开发指南](sdk/dev-kit/README.zh-CN.md)与[弹药机制说明](plugins/vera20k-ares/AMMO.zh-CN.md)。

要实际操作已有功能，使用[六区测试地图与启动说明](plugins/vera20k-ares/examples/sdk-lab/README.zh-CN.md)。同一引擎可切换原版对照、兼容护盾和过载护盾三种模式。

## 与引擎的分工

0.29 配套引擎的完整存档布局已变化。旧0.28组件可在新引擎中运行并保存恢复；此前旧引擎写出的完整存档需要原配套引擎和精确组件。组件状态版本仍为1，旧引擎和测试包继续保留。

引擎仓库负责 SDK 接口、WIT、组件运行时和通用宿主接入。本工程负责插件算法、配置解释与开发工具；引擎不再包含这份插件源码或打包工具。

`sdk/api` 和 `sdk/wit` 是版本固定的开发依赖快照，记录在 [sdk/source.json](sdk/source.json)。接口的权威源码位于相邻引擎工程，升级 SDK 时从权威版本同步，保持双方接口匹配。日常插件编译只读取本地快照，不访问引擎目录。

0.32 增量新增独立 `features/projectiles/ballistic_scatter.rs`，在同一 `projectile_resource` 返回中组合可选 `ballistic_scatter` 与已有 Airburst 声明。新对局和真实配置重建边界拥有该域；旧非空配置缺域时继续省略，不依赖 Ammo／Airburst 开关。物理 Rules 阶段通过 Projectile 专属保留历史，按 Min 后 Max 调用宿主 `ares-nullable-leptons`，读取 `BallisticScatter.Min/Max`；默认均未声明，有效0是显式覆盖，空值或非法输入保留原生读取结果。最终绑定与运行查询只使用冻结配置，不增加逐帧 Wasm 调用。

本批范围是普通 FireAt 的 `Inaccurate=yes`＋`Arcing=yes` 弹道散布及其中的 plain／`FlakScatter` 且非 `Inviso` 分支。未声明上下界时由原发射所有者分别选择全局默认；`Inviso`＋`FlakScatter` 的额外 Bullet::Fire 放置散布不在两键覆盖范围。该增量使用 config22／state1／API v1，WIT 和 Hook 不变，但需提供新字段及可空 Leptons 读取器的配套宿主；数值解析、随机流和发射仍使用原宿主所有者，验收状态随后续配套检查更新。

0.31 新增独立 `features/projectiles/airburst.rs` 与可选 `projectile_resource`，为宿主 Ares Airburst／Splits 替换尾部提供每种 Projectile 的冻结声明。物理 Rules 阶段按实际 Projectile 家族保留历史读取 `AirburstSpread`、`RetargetAccuracy`、`Splits`、`RetargetSelf` 和 `AroundTarget`；double 原始位模式由宿主原生读取器提供，插件不做浮点解析或范围修正。默认分别为 1.5、0、false、true；未指定 `AroundTarget` 时取最终 `Splits`。最终绑定不重读物理注册表，运行中不逐帧调用 Wasm。

只有新对局与真实配置重建边界拥有新 airburst 域，并一次性选择 `ares_replacement`；旧非空配置保留既有 `unsupported_replacement` 行为与省略的新返回字段。新域要求同一配置拥有 range／firing 域和宿主已有的 Warhead 关系完整接纳，子弹航程、目标、随机流、构造、引爆和存读档仍由宿主原有所有者维护。候选为 config21／state1／API v1，WIT 不变，但需要支持新 Hook、字段及 Projectile 专属读取器的配套宿主。实际组件的普通扇面、Splits 过滤、随机顺序、共享 Dummy、子弹字段和冻结保存回归已经通过；完整 AR268 兼容及尚未覆盖的特殊入口仍见[兼容台账](compatibility/README.zh-CN.md)。

0.30 新增独立 `features/weapons/projectile_range.rs`，在既有 `weapon_firing_resource` 中组合可选 `projectile_range`，并通过 `match_rules.projectile_fuel` 一次性选择宿主航程运行时。新对局及真实配置重建边界默认拥有该域，无需额外启用标签；非空旧配置即使已有 firing 域，也不补入 range 域。完整 Rules 物理读取阶段通过 Weapon 专属历史和宿主 `ares-leptons` 读取器保留 `ProjectileRange`，最终绑定只冻结声明。默认值为 100000 leptons（390.625 格），成功读取的有符号结果原样保留。

0.30 的范围是普通 FireAt 创建弹体时写入武器航程，以及 `Ranged=yes` 的共享飞行／耗尽引爆接入；bare／shrapnel 创建仍使用 INT_MAX 初值，该版本未接入其他特殊创建器或 Ares 替代空爆扇面。剩余航程、飞行、引爆、保存与校验和归引擎唯一所有者，不逐帧调用 Wasm，也不在插件保存第二份计数器。未声明或显式关闭 `projectile_fuel` 的组件采用原版无燃料模式。该版本为 config20／state1／API v1，WIT 不变，但需要提供新字段和读取器的配套宿主；完整存档仍应保留原组件与对应引擎。

0.29 新增可选 `weapon_firing_resource`，接入 Ares 多方向开火动画与 `ApplyDamage`，实现位于 `features/weapons/firing.rs`。安装后无需额外启用开关：`Anim` 列表按最大的完整二次幂方向数选择，普通武器默认保留直接伤害，声波／喷火粒子武器默认关闭，`ApplyDamage` 可显式覆盖。配置通过宿主保留的 Weapon 专属物理读取历史调用已有 Ares nullable 布尔读取器，同名 Projectile 后续读取不会替换武器来源。完整 Rules 物理读取结束时更新声明，最终绑定阶段保留这些声明而不重读；资源查询只读冻结配置。已有 AI 伤害估值、动画引用、坐标、伤害缩放与构造仍由原引擎负责，`ApplyDamage` 不扩展到 AI 估值。该版本为 config19／state1／API v1，需支持新 Hook 的配套引擎；旧组件不接收新请求，旧非空配置不自动补入新功能。

0.28 将引擎已有的超武扩展接入 `features/superweapons/`。`configure_super_read` 在原规则读取时点处理首次访问和 `SW.Warhead` 引用；`superweapon_resource` 在配置结束后一次性提供挂载、使用条件、初始/虚拟充能、费用、通用弹头、单位投送及演出声明。作者直接使用既有 Ares INI 标签，无需额外启用标签。Host 共用原数值、列表和固定音频目录读取接口；计时、发射次数、钱包、两阶段选择和投送队列继续由原引擎所有者维护，Wasm 不逐帧改写这些状态。

0.28 候选的配置版本为18、实体状态版本为1，API v1 与 WIT 名称保持不变，但两个新增接口需要配套宿主。旧组件未声明时沿用宿主模式；显式默认声明选择原版行为。存档恢复使用存档中的精确组件与冻结配置，原始组件及配套引擎应与旧存档一起保留。范围限于引擎原有的超武子集；完整 charge-drain、新超武类型和更多 AI 选点方式仍未由本次迁移实现。

0.27 新增纯启动声明 `sound_sample_admission`，在固定 SOUNDMD 目录首次读取前选择未索引样本的接纳方式。参考组件直接声明 true，合法散装 WAV 可沿原有样本选择与解码流程加载；无需额外 INI 开关。未声明沿用宿主模式，显式 false 在存在音频索引时要求样本已登记。声音目录、资源查找和解码器仍由宿主统一维护，资源重新接入继续使用原冻结目录。相同 INI 摘要不代表接纳模式相同，恢复仍依赖精确组件身份与固定资源配方。配置17／状态1／API v1／WIT不变，新声明需要配套宿主；未进行音频设备输出或跨平台实跑。

0.26 将普通弹头的 House 关系过滤与既有空爆边界接入 `match_rules.warhead_house_filter`／`airburst_tail`。参考组件直接声明启用前者，即使 `AresEMP=no` 也保留关系过滤；缺少有效弹头关系数据的启用声明会被宿主拒绝，不再静默回退。空爆选择 `unsupported_replacement`，仅保留既有未实现边界，不新增替代算法，父弹伤害仍正常执行。未声明沿用宿主模式，显式 false／`retail_fixed_nine` 可选择原版行为。实际组件的伤害、正式发射、保存恢复、原版与旧模式对照及真实地图加载检查通过。配置17／状态1／API v1／WIT不变，仍需配套宿主；没有新增窗口、音频设备、联网或跨平台实跑。

0.25 新增纯启动声明 `prism_support_defaults`，在第一次 Rules 读取前选择既有的棱镜支援倍率默认单位修复。参考组件直接返回 true，无需 INI 开关；未声明沿用宿主模式，显式 false 选择原版。存在 `[General]` 但缺少倍率键时，后续读取保留正确的百分比；数值解析、支援计数及发射仍归原所有者。实际组件的分层读取、保存恢复、延迟发射、旧模式和真实资源加载检查通过。配置17／状态1／API v1／WIT不变，但需要配套宿主；原生规则存档布局已变化，旧存档保留对应旧引擎和组件。本次不新增 PrismForwarding，未进行窗口或跨平台实跑。

0.24 新增独立的纯启动声明 `spawn_identity`，在正式资源接纳后批准 CnCNet `spawn.ini` 的国家编号及战役 Side 范围。参考组件直接声明扩展 Country 与 byte Side，不读取玩法配置或增加 INI 开关；未声明沿用宿主原模式，显式 false 可覆盖旧内置模式。输入先由唯一解析器读取一次，再于原 Ready 流程按固定组件策略批准；国家是否注册、地图、人数和颜色仍由现有所有者校验。战役 Side／CampaignID 仍是元数据，显式 Scenario 仍使用记录0。实际组件的人类／电脑国家、阵营和基地车、旧实现、资源重建及七种真实资源加载检查通过。社区测试覆盖帧解码和同一批准入口，未安装全局快照，也未实际启动窗口／联网；独立网络房间描述的国家范围仍由既有协议校验。配置版本仍为17、状态版本仍为1，API v1/WIT不变，仍需配套宿主。

0.23 新增一次性 `type_sound_resource` 目录，接通 `Promote.VeteranSound`／`Promote.EliteSound`。插件沿实际 Techno 读取点保留继承全局、静音和已注册音效三态，宿主校验固定 SOUNDMD 来源及原生类型身份后一次绑定；军衔、位置音效和 EVA 仍由原机制维护。已注册的字面 `none` 是音效，静音仅关闭音效、不关闭 EVA。未声明保留旧模式，显式空目录在旧构建中也使用全局音效。不增加 INI 开关或逐次晋升 Wasm 调用。实际晋升到派发、存档后继续升级、来源变化拒绝、旧实现、共享 PCX GPU 及七种真实资源模式检查通过；未测试音频设备实际播放，仍需配套新版宿主。 接口见[类型音效目录](sdk/api/src/audio.rs)。

0.22 新增一次性 `cameo_resource` 目录，迁移普通、晋升及超级武器的 PCX 图标。插件按实际 Rules／固定 ART 读取点保留文件名，宿主继续共用原文件名读取器、PCX/SHP 解码和图集；晋升 PCX 缺失时先回退普通 PCX，再走既有 SHP 路径，完全透明的 PCX 仍保持选中。安装即声明，不增加 INI 开关或逐图标 Wasm 调用。未声明保留旧模式，显式空目录选择原版。实际组件 GPU、同组件存档恢复、旧实现、七种真实资源模式及上一版组件加载检查通过。正式图标绑定在场景完成并经过共同 TypeReset 后发生，中间配置暂存不代表原生 Super 扩展跨 ClearScene 存活；Ares 完整文档兼容仍未完成。 接口见[图标目录](sdk/api/src/cameo.rs)。

0.21 通过 `match_rules.anim_damage` 接通既有动画所属方来源与 ART 弹头优先级。安装即声明，无新 INI 开关；原引擎保留累积、计时、引用分配和伤害接收。实际动画、无组件／旧组件／显式关闭、旧构建和针对性真实资源加载检查通过，整轮组合验收仍待收口；不包含 Weapon=／Damage.Delay= 新功能。

0.20 通过 `match_rules.building_presentation` 声明建筑 SHP／MAKE 独立调色板和销毁动画颜色回退，安装即生效，不增加 INI 开关。原引擎继续维护 ART 读取、调色板解析和动画状态；对象构造前刷新同一资源目录，建筑消失后的动画仍保留原颜色。实际组件、保存恢复、GPU、旧实现及真实资源加载检查通过；不包含新的建筑动画或 voxel 调色板扩展。

## 来源与许可

本工程迁自 VERA20K 本地源码提交 `314dfcbb5202a171d17edbd79fc1fab21d77e3f0`；原实现历史保留在引擎 Git 仓库。迁移保留文件内容、原有代码注释、贡献者说明和 GPL 许可，见 [LICENSE-GPL](LICENSE-GPL)。

Ares 全量兼容以[逐项兼容台账](compatibility/README.zh-CN.md)持续跟踪，包含官方功能、修复和参考入口；文件名不表示全量兼容已完成。

`NoAmmoWeapon`按安装版Ares的有符号整数语义处理：任意负数关闭，非负索引原样发布；有效0..17槽复用宿主现有选择器和精英回退。`WeaponN`读取、炮手提前返回及>=18安全无武器的边界见[弹药机制说明](plugins/vera20k-ares/AMMO.zh-CN.md)。建筑动画延迟枪需使用保存完整索引的配套新版宿主。0.6.0不扩展SDK或配置/状态结构。

内部组件标识仍为 `vera.reference-gameplay`。保留交付是组件 0.7.0、配置版本 5、状态版本 1；当前开发候选是 0.34.0、配置版本 23、状态版本 1，API v1 与 WIT 不变。开发分支已接入生产、护甲、附加效果、EMP、探照灯、类型转换、建筑占地、启动资源及阵营共享配置／玩法等既有子集，具体范围和配置见[独立开发指南](sdk/dev-kit/README.zh-CN.md)，Ares 完整文档兼容仍未完成。新默认只用于新对局；旧组件字节和已冻结配置保留原行为。读档要求当前作品提供与存档摘要相同的原组件，完整存档布局变化时也需保留相应旧引擎。本工程使用独立本地 Git 管理，未经授权不上传或公开。

0.17 接通国家菜单、旗帜、早晚加载画面与加载音乐，使用同一实时组件声明和原界面快照。场景全局清空保留尚存国家的父国继承历史，真正类型重建才清除；音乐仍由原播放模块负责。该链已通过实际组件、关联回归与旧实现对照，七种真实资源模式及上一版组件加载也已通过。0.18 接入侧栏、EVA SetSide、消息颜色、胜负文字及结算资源，仍使用原有界面、音频与资源所有者；保留交付不变。

启动资源迁移新增可选 `startup_resources` 能力，代码位于 `plugins/vera20k-ares/src/features/resources/startup.rs`。组件在原有加载时点选择 `[#include]`、附加音频目录和 CSF 文本；引擎仍负责文件查找、解析与实际资源目录。该增量已通过实际组件、应用加载与恢复、局域网地图工作线程、旧实现对照、七种零售资源模式和上一版组件加载检查。原保留交付不变，Ares 完整文档兼容尚未完成。
