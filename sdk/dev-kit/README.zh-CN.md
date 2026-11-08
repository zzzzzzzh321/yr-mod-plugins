# VERA20K 插件独立开发包

本开发包用于在没有 VERA20K 引擎源码的环境中，修改、测试并编译 `vera20k-ares.wasm`。编译只依赖插件源码、SDK 的类型与接口定义，以及 Rust 工具链。

本说明维护在独立 `plugin-dev` 工程的 `sdk/dev-kit/README.zh-CN.md`，也可作为导出开发包的根 README。下文所有路径和命令均以**插件开发工程或导出开发包的根目录**为起点。

## 开发包内容

0.29 配套引擎的完整存档布局已变化。旧0.28组件可在新引擎中创建对局并保存恢复，旧引擎写出的完整存档仍需原引擎和精确组件。保留旧版本配套；组件状态版本1与引擎完整存档布局分别管理。

```text
开发包根目录/
├── Cargo.toml                   # 仅 SDK 与插件的独立 workspace
├── Cargo.lock                   # 独立开发包的依赖版本
├── LICENSE-GPL
├── sdk/
│   ├── api/                     # 请求、结果、公共数据类型与计时器
│   ├── wit/                     # 插件与引擎之间的接口定义
│   ├── package/                 # 通用组件打包工具源码
│   └── dev-kit/                 # 本说明与 workspace 模板
└── plugins/
    └── vera20k-ares/            # 可修改的功能插件源码与 INI 示例
```

根 workspace 只有 `vera-mod-api`、`vera-mod-pack` 和 `vera20k-ares` 三个 crate，默认构建成员为插件。其他第三方依赖由 Cargo 按锁文件获取。这个工程不包含引擎 crate、Wasmtime 运行时或游戏资源，也不需要 Python 和引擎专用的 `tools.cargo_run`。

后续插件源码和打包工具以本工程为维护位置。`sdk/api`、`sdk/wit` 是从引擎权威接口取得的固定版本开发依赖，来源见 `sdk/source.json`；升级接口时明确同步，日常插件编译只访问本地快照。

SDK 中的 API 和 WIT 类似 C++ 插件开发中的公共头文件与调用契约：它们说明双方交换哪些数据、可以调用哪些接口。具体的 Host 服务由运行插件时的引擎提供，编译插件时无需链接引擎源码或可执行文件。

## 准备环境

安装支持 Rust 2024 edition 的 Rust 工具链，确保 Cargo 和当前平台所需的链接器可用。然后安装插件编译目标：

```sh
rustup target add wasm32-unknown-unknown
```

首次构建通常需要联网下载 Cargo 依赖；已缓存的依赖可以复用。保留开发包附带的 `Cargo.lock`，以使用同一组依赖版本。

## 修改与测试

功能代码位于 `plugins/vera20k-ares/src/features/`，仍按防御、武器等领域组织。公共宿主调用集中在 `src/support/host.rs`，组件入口和功能组合位于 `src/component.rs`。

从开发包根目录运行：

```sh
cargo test --locked -p vera-mod-api -p vera20k-ares -p vera-mod-pack --lib
```

这些测试检查插件决策与打包行为。它们可以在没有 `vera20k.exe` 和原版游戏资源的环境中运行；实际游戏效果、引擎接入、存读档和联机行为仍需放入兼容引擎中验收。

## 编译插件

```sh
cargo build --locked --release --target wasm32-unknown-unknown -p vera20k-ares
```

原始输出为：

```text
target/wasm32-unknown-unknown/release/vera20k_ares.wasm
```

这是包含 WIT 元数据的 core Wasm，接下来需要封装为引擎加载的组件。此处使用 `wasm32-unknown-unknown`，保持当前 SDK 所要求的导入接口。

## 生成发布文件

先在开发包根目录创建 `dist` 文件夹，再执行以下命令。输出子目录 `release-new` 必须尚不存在；下一次发布使用另一个新子目录。

```sh
cargo run --locked --release -p vera-mod-pack -- target/wasm32-unknown-unknown/release/vera20k_ares.wasm dist/release-new --program vera20k-ares.wasm
```

Cargo 会编译本机使用的打包工具并执行，产物为：

```text
dist/release-new/
├── vera20k-ares.wasm
└── vera-mod.json
```

声明中的文件名与内容摘要由打包工具生成。将这两个文件一起放入作品资源系统，再用作品实际加载的INI配置功能。0.7.0稳定交付的新对局默认启用`AresAmmo`、`AresReload`、`AresNoAmmoWeapon`、`AresManualControl`。当前0.30.0开发候选还默认启用`AresKeepAlive`、`AresAttachEffect`、`AresArmor`、`AresProduction`、`AresEMP`、`AresSpotlight`、`AresTypeConversion`、`AresFoundation`和`AresFaction`，迁移引擎已有的存活判定、附加效果及其隐形/属性依赖、自定义护甲和建筑占地等支持。可以直接使用原Ares标签；`[VeraFeatures]`中对应项设为`no`可分别关闭。标签自身的默认值不变，四个手动限制仍默认`no`。Phobos护盾和自定义过载仍须显式`Shield=yes`与`Overload=yes`，过载依赖护盾。作品的其他美术、音频、地图和规则资源沿用原有分发流程。

如果设置了 `CARGO_TARGET_DIR`，原始 Wasm 的位置随之改变；打包命令中的输入路径也应使用该目标目录。源码修改和编译过程不会自动改变作品当前对局中的组件。

## SDK 与引擎版本

0.30 开发候选通过独立 `features/weapons/projectile_range.rs` 接入普通 FireAt 的武器初始航程。作者仍在武器段配置 `ProjectileRange`（单位为格），在所用抛射体段设置 `Ranged=yes` 才会消耗航程并在耗尽时进入宿主共享引爆路径，无需额外功能启用标签。例如在作品已有武器／抛射体定义中加入：

```ini
[MyWeapon]
Projectile=MyProjectile
ProjectileRange=6

[MyProjectile]
Ranged=yes
```

完整 Rules 物理读取阶段通过 `$weapon-config` 和 `retained_weapon_read("ares-leptons")` 请求宿主已有的 Weapon 专属读取历史，最终绑定不访问物理注册表；资源查询只组合冻结声明。默认值为 100000 leptons（390.625 格），空值或非法值由宿主原生读取器保留此前成功值；有符号结果不在插件内截断或归零。同名 Projectile 后续读取不会替换 Weapon 来源。该配置独立于旧 firing 域：只有新对局及真实 ScenarioClear／TypeReset 重建边界创建 range 域，旧非空配置保持不接管。

组件以 `match_rules.projectile_fuel=Some(true)` 一次选择宿主航程运行时，以可选 `weapon_firing_resource.projectile_range` 提供普通 FireAt 初值。未声明或显式 false 的组件采用原版无燃料模式，旧组件也不被自动启用。bare／shrapnel 创建仍保留 INT_MAX 初值，其他特殊创建器及 Ares 替代空爆扇面不属于本批完成声明。引擎唯一所有者负责实时计数、飞行、引爆、存读档和校验和，不新增逐帧 Wasm 调用或插件计数器。候选为配置20／状态1，API v1 与 WIT 不变，但需提供新字段及读取器的配套宿主；完整存档仍保留原组件与对应引擎，验收状态随配套检查更新。

0.29 新增可选 `weapon_firing_resource`，接入 Ares 多方向开火动画与 `ApplyDamage`，实现位于 `features/weapons/firing.rs`。安装后无需额外启用开关：`Anim` 列表按最大的完整二次幂方向数选择，普通武器默认保留直接伤害，声波／喷火粒子武器默认关闭，`ApplyDamage` 可显式覆盖。配置通过宿主保留的 Weapon 专属物理读取历史调用已有 Ares nullable 布尔读取器，同名 Projectile 后续读取不会替换武器来源。完整 Rules 物理读取结束时更新声明，最终绑定阶段保留这些声明而不重读；资源查询只读冻结配置。已有 AI 伤害估值、动画引用、坐标、伤害缩放与构造仍由原引擎负责，`ApplyDamage` 不扩展到 AI 估值。该版本为 config19／state1／API v1，需支持新 Hook 的配套引擎；旧组件不接收新请求，旧非空配置不自动补入新功能。

0.28 已完成本阶段既有 Ares 子集的外置迁移，完整 Ares 兼容仍未完成。接口见[超武声明](../api/src/superweapon.rs)。

0.28 将引擎已有的超武扩展接入 `features/superweapons/`。`configure_super_read` 在原规则读取时点处理首次访问和 `SW.Warhead` 引用；`superweapon_resource` 在配置结束后一次性提供挂载、使用条件、初始/虚拟充能、费用、通用弹头、单位投送及演出声明。作者直接使用既有 Ares INI 标签，无需额外启用标签。Host 共用原数值、列表和固定音频目录读取接口；计时、发射次数、钱包、两阶段选择和投送队列继续由原引擎所有者维护，Wasm 不逐帧改写这些状态。

0.28 候选的配置版本为18、实体状态版本为1，API v1 与 WIT 名称保持不变，但两个新增接口需要配套宿主。旧组件未声明时沿用宿主模式；显式默认声明选择原版行为。存档恢复使用存档中的精确组件与冻结配置，原始组件及配套引擎应与旧存档一起保留。范围限于引擎原有的超武子集；完整 charge-drain、新超武类型和更多 AI 选点方式仍未由本次迁移实现。

0.27 新增纯启动声明 `sound_sample_admission`，在固定 SOUNDMD 目录首次读取前选择未索引样本的接纳方式。参考组件直接声明 true，合法散装 WAV 可沿原有样本选择与解码流程加载；无需额外 INI 开关。未声明沿用宿主模式，显式 false 在存在音频索引时要求样本已登记。声音目录、资源查找和解码器仍由宿主统一维护，资源重新接入继续使用原冻结目录。相同 INI 摘要不代表接纳模式相同，恢复仍依赖精确组件身份与固定资源配方。配置17／状态1／API v1／WIT不变，新声明需要配套宿主；未进行音频设备输出或跨平台实跑。

0.26 将普通弹头的 House 关系过滤与既有空爆边界接入 `match_rules.warhead_house_filter`／`airburst_tail`。参考组件直接声明启用前者，即使 `AresEMP=no` 也保留关系过滤；缺少有效弹头关系数据的启用声明会被宿主拒绝，不再静默回退。空爆选择 `unsupported_replacement`，仅保留既有未实现边界，不新增替代算法，父弹伤害仍正常执行。未声明沿用宿主模式，显式 false／`retail_fixed_nine` 可选择原版行为。实际组件的伤害、正式发射、保存恢复、原版与旧模式对照及真实地图加载检查通过。配置17／状态1／API v1／WIT不变，仍需配套宿主；没有新增窗口、音频设备、联网或跨平台实跑。

0.25 新增纯启动声明 `prism_support_defaults`，在第一次 Rules 读取前选择既有的棱镜支援倍率默认单位修复。参考组件直接返回 true，无需 INI 开关；未声明沿用宿主模式，显式 false 选择原版。存在 `[General]` 但缺少倍率键时，后续读取保留正确的百分比；数值解析、支援计数及发射仍归原所有者。实际组件的分层读取、保存恢复、延迟发射、旧模式和真实资源加载检查通过。配置17／状态1／API v1／WIT不变，但需要配套宿主；原生规则存档布局已变化，旧存档保留对应旧引擎和组件。本次不新增 PrismForwarding，未进行窗口或跨平台实跑。

0.24 新增独立的纯启动声明 `spawn_identity`，在正式资源接纳后批准 CnCNet `spawn.ini` 的国家编号及战役 Side 范围。参考组件直接声明扩展 Country 与 byte Side，不读取玩法配置或增加 INI 开关；未声明沿用宿主原模式，显式 false 可覆盖旧内置模式。输入先由唯一解析器读取一次，再于原 Ready 流程按固定组件策略批准；国家是否注册、地图、人数和颜色仍由现有所有者校验。战役 Side／CampaignID 仍是元数据，显式 Scenario 仍使用记录0。实际组件的人类／电脑国家、阵营和基地车、旧实现、资源重建及七种真实资源加载检查通过。社区测试覆盖帧解码和同一批准入口，未安装全局快照，也未实际启动窗口／联网；独立网络房间描述的国家范围仍由既有协议校验。配置版本仍为17、状态版本仍为1，API v1/WIT不变，仍需配套宿主。 接口见[启动声明](../api/src/startup.rs)，安装级选择直接在组件入口返回，不放入 Shield 或实体配置。

0.23 新增一次性 `type_sound_resource` 目录，接通 `Promote.VeteranSound`／`Promote.EliteSound`。插件沿实际 Techno 读取点保留继承全局、静音和已注册音效三态，宿主校验固定 SOUNDMD 来源及原生类型身份后一次绑定；军衔、位置音效和 EVA 仍由原机制维护。已注册的字面 `none` 是音效，静音仅关闭音效、不关闭 EVA。未声明保留旧模式，显式空目录在旧构建中也使用全局音效。不增加 INI 开关或逐次晋升 Wasm 调用。实际晋升到派发、存档后继续升级、来源变化拒绝、旧实现、共享 PCX GPU 及七种真实资源模式检查通过；未测试音频设备实际播放，仍需配套新版宿主。 接口见[类型音效目录](../api/src/audio.rs)，实现维护在 `features/presentation/promotion_sounds.rs`，Host 薄适配仍集中在 `support/host.rs`。音效必须先由实际 SOUNDMD 的 `[SoundList]` 注册；未知、缺失、空值保留上次声明，未注册的 `none`／`<none>` 才表示静音。声音目录在首次配置前固定，同一 Arc 贯穿独立投影、类型重置和恢复；保存不序列化目录，正式恢复重绑后校验来源，即使名称与编号相同也不接受不同输入。新增能力使用配置版本17、状态版本1，API v1/WIT不变；旧配套引擎／组件继续保留。

0.22 新增一次性 `cameo_resource` 目录，迁移普通、晋升及超级武器的 PCX 图标。插件按实际 Rules／固定 ART 读取点保留文件名，宿主继续共用原文件名读取器、PCX/SHP 解码和图集；晋升 PCX 缺失时先回退普通 PCX，再走既有 SHP 路径，完全透明的 PCX 仍保持选中。安装即声明，不增加 INI 开关或逐图标 Wasm 调用。未声明保留旧模式，显式空目录选择原版。实际组件 GPU、同组件存档恢复、旧实现、七种真实资源模式及上一版组件加载检查通过。正式图标绑定在场景完成并经过共同 TypeReset 后发生，中间配置暂存不代表原生 Super 扩展跨 ClearScene 存活；Ares 完整文档兼容仍未完成。 声明接口见[图标目录](../api/src/cameo.rs)，实现维护在 `features/presentation/cameos.rs`。Techno 采用 class-local 原生数组身份，Super 使用独立身份空间。固定 ART 哈希和实际物理读取点进入配置阶段；缺失／空字段保持历史，字面 `none`／`<none>` 是文件名。正常与晋升共享 PCX 资源，保留 RGB565 洋红透明键、不透明黑色及 60×48 逻辑画布，不缩放源图。存档采用相同摘要的 Program 和冻结配置，恢复时不重新解释 INI。新增 hook 需要配套宿主；配置版本16、状态版本1，API v1/WIT 不变。旧引擎与组件继续保留，不承诺任意旧完整存档自动迁移。跨 Techno 类同名图标合并、动态 VFS／surface 缓存时机仍是既有边界。

0.21 的 `match_rules.anim_damage` 同样是安装级静态策略：`use_source_house` 优先读取动画已有所属方，没有时读取附着 Techno 的所属方；`prefer_art_warhead` 优先消费原固定 ART 的 Warhead 引用，空引用仍用自身 INVISO/C4Warhead 或 FlameDamage2 回退。原加载器会为未列出的引用创建默认弹头，不应把“未注册列表”当成“查找失败”。伤害累积、计时、扣量和 AreaDamage 仍由原引擎拥有，House 不替代攻击对象，视觉 AnimRemap 不设置伤害所属方。未声明保留原模式，显式 false 覆盖旧内置模式，参考组件安装即声明两项 true，不依赖功能 INI 开关。实际组件对照、保存字段与 Program 接纳、旧构建及针对性资源加载检查通过；不包含 Weapon=／Damage.Delay=，整轮完整组合检查仍待收口。配置版本15、状态版本1、API v1 不变，但需要提供这些策略的新宿主。

0.20 的 `match_rules.building_presentation` 是两项安装级静态策略：`independent_palette_groups` 允许建筑 SHP／MAKE 各自使用成功加载的调色板组，`destroy_anim_house_fallback` 在没有成功自定义组时为 DestroyAnim 使用所属方颜色。参考组件直接声明两项 true，不受功能 INI 开关控制；未声明保留宿主原模式，显式 false 选择原版行为。原 ART 读取、PAL 解析、动画状态和 GPU 渲染继续共用，引擎在正式接纳组件后、对象构造前刷新原资源目录。建筑消失后动画仍保存自己的组和颜色身份；这不会设置动画伤害 owner_house。实际组件、销毁／保存、GPU、旧实现及真实资源加载对照通过。配置版本15、状态版本1、API v1 不变，但仍需配套新版宿主；不包含 bib、活动／超武动画或 voxel 调色板扩展。

0.19 的 `long_list_readers` 是独立的安装级纯声明，在任何冷 Rules 读取前执行。参考组件直接返回 `Extended2048`，仅扩展 House `Allies`、Techno `Prerequisite` 和 `PrerequisiteOverride`；不受 `AresProduction`／`AresFaction` 开关控制。其他组件可明确返回 `Retail128`，不声明则保留宿主原模式。General 六类前置保持128，物理 INI 行限制保持511字节；原解析器、注册表和外交模块继续负责实际结果。配置版本仍为15，状态版本仍为1；完整旧存档保留对应旧引擎与原组件。

接口编号仍为 SDK API v1，WIT 名称为 `vera:mod-sdk@0.1.0`。参考插件保留内部标识`vera.reference-gameplay`和发布文件名`vera20k-ares.wasm`。稳定交付0.7.0使用配置版本5、状态版本1；当前0.30.0开发候选使用配置版本20、状态版本1；探照灯、类型转换、建筑占地、启动资源和阵营共享配置／玩法增量已完成各自的集中验收，Ares 完整文档兼容尚未完成，也未替换保留交付。

开发分支新增`match_rules`、`effect_resource`、`effects`、`armor_resource`、`production_resource`、`configure_boundary`和`emp_resource`可选能力，以及类型隐形许可等消息字段。宿主须提供物理Rules读取点的注册类型查询、效果实例事务、原数学/动画/隐形所有者和相应保存恢复接入；只认识原有回调的宿主无法运行这版组件。组件的护盾字节与宿主效果列表分别保存，效果列表只有一个所有者。宿主接收静态策略时建立派生缓存，每次效果更新只提交经校验的有序动作。护甲增量已通过关联回归及真实资源加载，Ares 完整文档兼容仍未完成。SDK快照的配套引擎来源提交见`sdk/source.json`。

修改现有接口能够表达的算法，可以只重编插件。需要新的引擎能力时，应先扩展 SDK 契约和引擎接入点，再使用匹配的 SDK 构建插件；独立编译并不表示所有未来 SDK 与引擎版本自动兼容。

引擎加载时会检查组件声明、摘要、接口及允许的导入。新默认只用于新对局的初始配置，旧的非空冻结配置不补入新默认；旧存档继续使用保存的原组件字节与配置，不重读当前INI。当前作品资源还必须提供与存档摘要一致的组件，因此更新插件时保留旧组件及声明，供旧存档使用。开发包沿用 GPL 授权，见 `LICENSE-GPL`。

## 维护者如何组装开发包

以本插件开发工程为来源，在新的输出目录保持上述相对布局，复制 `sdk/api`、`sdk/wit`、`sdk/package`、`sdk/dev-kit`、`sdk/source.json`、`plugins/vera20k-ares` 和许可证。复制根 `Cargo.toml` 与 `Cargo.lock`，将本说明复制为导出包的根 README。模板 `sdk/dev-kit/Cargo.toml.template` 也可用于建立新的同结构工程。

本工程的锁文件已经只包含 SDK、插件与打包工具的依赖。在依赖已缓存的维护环境中，可执行以下命令检查导出工程仍能按该版本解析：

```sh
cargo metadata --locked --offline --format-version 1
```

检查依赖图只包含开发包成员及其依赖，再使用前述带 `--locked` 的测试、构建和打包命令验证。不要将引擎 workspace 或引擎锁文件作为本开发包的根配置。

源码以 `plugin-dev` 为准，导出包是交付快照。一般作者直接使用包内锁文件；只有维护者明确升级依赖时才重新解析版本。

高于主/副武器的 NoAmmoWeapon 索引还需要能完整保存建筑延迟武器索引的配套宿主。API编号或WIT包名相同，不代表旧宿主具有新回调、消息字段或处理修复；按来源提交与配套验收包核对。新增宿主保存字段会改变完整存档布局，保留旧引擎和旧组件供旧存档使用，不能把护盾状态版本仍为1理解成完整存档自动兼容。

生产扩展在 `features/production` 按组、工厂资格与公共目录索引组织；没有第二份 House 计数、工厂队列或永久计划。配置边界由宿主真实流程提供：普通场景清空，类型重建保留组件自身的全局控制，读取存档直接采用保存配置。跨场景种子必须匹配完全相同的组件内容与清单，更新组件后应开启新对局。0.10.0 普通引擎新增国家来源与计划字段，完整存档布局随之变化；旧存档保留相应旧引擎和组件，不自动转换。生产增量已通过关联回归、旧实现对照、正式读档和七种真实资源加载检查；Ares 完整文档兼容尚未完成。

EMP 扩展位于 `features/units/emp`。新配置默认启用 `AresEMP`，历史冻结配置缺少该开关时保持关闭；`[VeraFeatures] AresEMP=no` 明确停用。插件只发布静态声明，计数、停用状态和动画引用仍只有原宿主所有者。任务名称、阈值和注册动画由共享 Host 调用原读取器，类型动画的继承／清空／指定三态不会合并。TypeReset 保留 EMP 全局任务配置；普通场景清空，读档使用保存配置。EMP 增量已通过关联行为、旧实现对照、正式读档和七种真实资源加载检查。完整存档新增触发器冻结策略，旧存档需保留对应旧引擎和组件；整体迁移继续进行。

探照灯通过现有 `type_resource.spotlight` 发布静态声明，没有新增逐帧回调或灯光状态。新对局默认启用 `AresSpotlight`；关闭时恢复原版建筑灯规则。两套开关沿各自的物理读取时点累积，不能从最终合并文本重新猜测。原版 General 参数继续由宿主保留，颜色与几何交给原绘制链。本增量已通过关联行为、旧实现对照、应用资源加载、现有 GPU 像素对照和七种真实资源加载检查；原有绘制残差继续保留。

类型转换通过现有 `type_resource.conversion` 与 `match_rules.script_type_conversion` 发布冻结声明。`AresTypeConversion` 默认启用，关闭时恢复原版 Script68 不执行行为。`Convert.Script` 使用真实读取点的共同 Techno 顺序，保留类别和编号；`TurretROT` 保留完整有符号输入，只在转换时交原 Facing 所有者处理。单位状态、队伍游标和运动控制器仍由原引擎拥有。本增量已通过关联行为、旧实现对照、正式资源恢复和七种真实资源加载检查。普通引擎接纳停用矿车枚举分支后会改变完整存档布局；旧存档保留对应旧引擎和组件。

建筑占地使用独立的 `foundation_resource` 声明，源码在 `features/buildings/foundation.rs`。插件按真实 Building 读取点保存原版占地编号和第一次 `Foundation=Custom` 的固定 ART section 引用，关闭期间也保留这段历史；`AresFoundation=no` 只改变最终选择。插件不保存或逐格读取几何数组。宿主校验来源、内容散列与 section 后，由原 Foundation 模块生成完整占地和轮廓，摆放、占格、寻路与存档沿用原有所有者。读档复用保存的配置和匹配的固定 ART，不重新执行配置。普通引擎通过最终绑定检查自定义几何；旧内置 Ares 编译模式中，原始非法 Custom 仍会在更早的规则校验处失败，即使组件随后关闭该功能。

本占地增量已通过真实组件的空间与存档测试、关联回归、旧实现对照、正式客户端及现有摆放 GPU 检查、七种真实资源加载模式和上一版组件加载检查。这不表示所有自定义占地绘制路径都已与原生逐像素对照。普通构建新增 Custom 类型后，完整存档布局和配置摘要会变化；旧存档保留对应旧引擎与组件。

## 启动资源

`features/resources/startup.rs` 使用独立的可选 `startup_resources` 回调，在原有资源加载前沿分别返回 INI 引用语义、音频 IDX/BAG 配对顺序和 CSF 候选顺序。Ares 参考组件按原规则选择 `audio`、`ares`、`audio01` 至 `audio99`，以及 `ares.csf`、当前语言的 `ares_*.csf` 和 `stringtable00.csf` 至 `stringtable99.csf`。不存在的资源仍由原加载器处理；顺序、同名覆盖和语言检查沿用原模块，插件不复制解码器或目录。

此回调早于游戏规则配置，不读 `[VeraFeatures]`，也不能读文件、时钟、网络或数值服务；宿主只传入查询阶段和必要的基础 CSF 语言。不要给资源加载增加一个尚依赖这些资源才能读取的 INI 开关。该启动资源增量保留了当时的配置版本14；阵营增量当时使用配置版本15；当前候选使用配置版本17，状态版本仍是1。这不代表任意 API v1 旧引擎都支持新回调。

`vera-mod.json` 和需要启动能力的 Wasm 必须在音频选择前已经能被作品资源系统查到。只有声明较早可见、组件却藏在后续挂载的 MIX 时，宿主先保留待定身份：旧无启动回调组件仍可在原绑定处加载；需要新启动回调的组件会明确报错，不能补跑已经错过的加载步骤。已接纳的组件字节和声明身份在本进程固定，资源管理器恢复继续使用它们；读档仍按相同组件摘要接纳已冻结的游戏配置。

此增量已通过实际 Wasm 的源文件选择、音频样本解码、CSF 文本和保存恢复检查，以及正式应用资源管理、局域网下载与缓存工作线程、现有网络控制、旧 Ares 模式、七种零售资源加载模式和上一版组件检查。检查范围不包含完整联网对局、实际扬声器／窗口输出和性能对比。游戏引擎的原状态与解析所有者继续保留，整体 Ares 迁移尚未完成。

## 阵营共享配置与玩法

0.16.0 将既有 Side／Country 扩展放入 `plugins/vera20k-ares/src/features/factions/`：`mod.rs` 处理真实读取顺序，`side.rs` 与 `country.rs` 分别读取声明，`defaults.rs` 保存构造默认，`tests.rs` 验证共享机制。各模块继续使用同一个 `support/host.rs`，不复制 INI 解析器、类型查找、House 或随机数状态。

本批接入乘员／默认伪装、AI 基地防御与扩展阵营条件、Country→Side 空降／降落伞优先级，以及 VeteranBuildings 的实际构造军衔与既有图标资格。原 Ares 标签可直接写在作品规则或地图的对应节中；新对局默认启用 `[VeraFeatures] AresFaction`，显式 `no` 选择原版策略。原有空降等支持边界继续保留。

| 可选能力 | 发生时点与职责 |
| --- | --- |
| `eva_catalog` | 规则读取前选择固定 EVA 目录布局，使用空配置和纯启动请求。 |
| `configure_country_read` | 当前物理 Country body 之前读取开关并选择 128／2048 容量的原有读取器。 |
| `configure_faction_pass` | 只消费真实 Country-only 后缀，不重跑其他功能的完整配置。 |
| `faction_resource` | 按实际 Side／Country 编号和 stored ID 查询冻结声明，宿主校验引用后绑定。 |

配置保留第一次初始化与父国尝试；失败的父国查找也会消耗尝试。后层缺少或写错布尔开关时保留前值，先前发生的类型分配继续保留。安装期 EVA 目录与规则中的功能选择分别确定，开关不重建目录。完整 Process 和独立 Country 读取使用不同的真实记录；独立启动投影保留自己的记录，不改变真实进程历史。读档沿用相同摘要的原组件及冻结配置，保留旧引擎／组件配套版本。

本增量已通过真实组件、存档恢复、关联消费者、正式应用加载、旧实现对照、七种零售资源模式和上一版组件加载检查。0.17 接入菜单／旗帜、早晚加载画面和加载音乐，0.18 继续接入侧栏、EVA SetSide、消息颜色、胜负文字和结算资源；不宣称完整 Ares、完整窗口／设备输出、完整联网或性能等价。组件身份及完整不透明配置仍参与 SDK 规则摘要，其中也包含表现声明。

## 国家菜单与加载

0.17 的菜单、旗帜、状态提示和随机权重按实际国家编号进入原有界面快照，重复的截断 ID 不会合并。未分配、显式原版和扩展声明分别处理；查询错误通过既有房间／加载失败入口返回，不静默使用原版值。加载先冻结阵营、文字色与 PAL（包括缺失），实际 E_multi 后才读取背景和文字；取消票据保留已发生的读取，并归还资源管理器。

Country／Side 记录只在真实类型重建时清除；ScenarioClear 仅清 Rules 全局，尚存国家的父国尝试继续保留，避免重复继承覆盖第二次主规则读取。配置版本仍为15、对象状态版本仍为1；旧组件及配套引擎继续保留。

独立可选 `loading_theme_resource` 定义在 `sdk/api/src/loading.rs`，实现位于 `features/resources/loading_theme.rs`。它在 Selected→Prepared 原时点读取 root Side→Country 或已解析战役 Basic 的 `LoadingTheme`，不使用 ParentCountry 或后续合并规则。仅通过共享 `ConfigView::native_only` 调用宿主原 `name:128` 读取器，物理来源不暴露 raw 文本／目录／数值服务；名称绑定、缺名默认和播放仍归原 Theme 模块。该能力要求冷 Rules 前可见并声明 `configure_boundary`，可以独立于 `faction_resource` 和 `configure_country_read` 使用。Ares 参考组件的显式 `AresFaction=no` 选择原版加载音乐。

### Side 界面与结算资源（0.18）

普通引擎安装当前组件后，可在 Side 节使用现有的 `Sidebar.MixFileIndex`、`Sidebar.YuriFileNames`、`EVA.Tag`、`MessageTextColor`、`GraphicalText.*`、`CampaignScore.*` 和 `MultiplayerScore.*`。初始化、字段读取与声明仍在 `features/factions`；不在绘制时反复调用 Wasm，也不建立第二套语音队列、纹理解析器或音乐播放器。

`match_rules.registered_side_ui` 是可选的全局选择：缺失保留旧组件／构建模式，`false` 使用原版，`true` 使用已注册 Side 的现有界面入口。它与某个 Side 的原版资源声明分别处理，不能根据是否缺少图片或是否只有一张图集猜测模式。参考组件复用 `AresFaction`，没有增加新的 INI 开关。切回原版侧栏时退役上一组资源；失败的读档候选沿原资源事务回滚。

结算模型在当前 Side 仍有效时保存一次资源快照，后续画面与音乐使用该快照；合作任务的最终结算也保留本局快照。SkipScore、战败提示与合作分支继续按原时点选择流程，不提前执行不会显示的结算查询。组件查询失败会显示错误，保留原 Continue／战役续关清理顺序，不伪装成成功的原版结算。

本轮保留固定基线的既有边界：不新增第四种侧栏几何、DialogBackground 或观察者画面。另有已存在的适配差异：比分文件名显式 `none` 当前会走原版文件回退，而原 Ares 会尝试空文件名；本次迁移保留该行为并单列残留。API v1、配置版本15和状态版本1不变，新消息仍须使用配套新版引擎；旧组件及其引擎继续保留。实际界面截图只验证资源接入，不等于原版像素、设备音频、完整联网或性能认证。

本链已通过实际组件、关联规则／菜单／加载／恢复和旧 Ares 模式检查，七种真实资源模式及上一版组件加载也已通过；独立 Theme-only 的实际夹具覆盖战役路径，非战役前置由现调用链核对。未宣称完整 GPU 界面、实际扬声器、完整联机或性能等价。
