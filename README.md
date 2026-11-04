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

引擎仓库负责 SDK 接口、WIT、组件运行时和通用宿主接入。本工程负责插件算法、配置解释与开发工具；引擎不再包含这份插件源码或打包工具。

`sdk/api` 和 `sdk/wit` 是版本固定的开发依赖快照，记录在 [sdk/source.json](sdk/source.json)。接口的权威源码位于相邻引擎工程，升级 SDK 时从权威版本同步，保持双方接口匹配。日常插件编译只读取本地快照，不访问引擎目录。

0.26 将普通弹头的 House 关系过滤与既有空爆边界接入 `match_rules.warhead_house_filter`／`airburst_tail`。参考组件直接声明启用前者，即使 `AresEMP=no` 也保留关系过滤；缺少有效弹头关系数据的启用声明会被宿主拒绝，不再静默回退。空爆选择 `unsupported_replacement`，仅保留既有未实现边界，不新增替代算法，父弹伤害仍正常执行。未声明沿用宿主模式，显式 false／`retail_fixed_nine` 可选择原版行为。实际组件的伤害、正式发射、保存恢复、原版与旧模式对照及真实地图加载检查通过。配置17／状态1／API v1／WIT不变，仍需配套宿主；没有新增窗口、音频设备、联网或跨平台实跑。

0.25 新增纯启动声明 `prism_support_defaults`，在第一次 Rules 读取前选择既有的棱镜支援倍率默认单位修复。参考组件直接返回 true，无需 INI 开关；未声明沿用宿主模式，显式 false 选择原版。存在 `[General]` 但缺少倍率键时，后续读取保留正确的百分比；数值解析、支援计数及发射仍归原所有者。实际组件的分层读取、保存恢复、延迟发射、旧模式和真实资源加载检查通过。配置17／状态1／API v1／WIT不变，但需要配套宿主；原生规则存档布局已变化，旧存档保留对应旧引擎和组件。本次不新增 PrismForwarding，未进行窗口或跨平台实跑。

0.24 新增独立的纯启动声明 `spawn_identity`，在正式资源接纳后批准 CnCNet `spawn.ini` 的国家编号及战役 Side 范围。参考组件直接声明扩展 Country 与 byte Side，不读取玩法配置或增加 INI 开关；未声明沿用宿主原模式，显式 false 可覆盖旧内置模式。输入先由唯一解析器读取一次，再于原 Ready 流程按固定组件策略批准；国家是否注册、地图、人数和颜色仍由现有所有者校验。战役 Side／CampaignID 仍是元数据，显式 Scenario 仍使用记录0。实际组件的人类／电脑国家、阵营和基地车、旧实现、资源重建及七种真实资源加载检查通过。社区测试覆盖帧解码和同一批准入口，未安装全局快照，也未实际启动窗口／联网；独立网络房间描述的国家范围仍由既有协议校验。配置版本仍为17、状态版本仍为1，API v1/WIT不变，仍需配套宿主。

0.23 新增一次性 `type_sound_resource` 目录，接通 `Promote.VeteranSound`／`Promote.EliteSound`。插件沿实际 Techno 读取点保留继承全局、静音和已注册音效三态，宿主校验固定 SOUNDMD 来源及原生类型身份后一次绑定；军衔、位置音效和 EVA 仍由原机制维护。已注册的字面 `none` 是音效，静音仅关闭音效、不关闭 EVA。未声明保留旧模式，显式空目录在旧构建中也使用全局音效。不增加 INI 开关或逐次晋升 Wasm 调用。实际晋升到派发、存档后继续升级、来源变化拒绝、旧实现、共享 PCX GPU 及七种真实资源模式检查通过；未测试音频设备实际播放，仍需配套新版宿主。 接口见[类型音效目录](sdk/api/src/audio.rs)。

0.22 新增一次性 `cameo_resource` 目录，迁移普通、晋升及超级武器的 PCX 图标。插件按实际 Rules／固定 ART 读取点保留文件名，宿主继续共用原文件名读取器、PCX/SHP 解码和图集；晋升 PCX 缺失时先回退普通 PCX，再走既有 SHP 路径，完全透明的 PCX 仍保持选中。安装即声明，不增加 INI 开关或逐图标 Wasm 调用。未声明保留旧模式，显式空目录选择原版。实际组件 GPU、同组件存档恢复、旧实现、七种真实资源模式及上一版组件加载检查通过。正式图标绑定在场景完成并经过共同 TypeReset 后发生，中间配置暂存不代表原生 Super 扩展跨 ClearScene 存活；整体迁移仍未完成。 接口见[图标目录](sdk/api/src/cameo.rs)。

0.21 通过 `match_rules.anim_damage` 接通既有动画所属方来源与 ART 弹头优先级。安装即声明，无新 INI 开关；原引擎保留累积、计时、引用分配和伤害接收。实际动画、无组件／旧组件／显式关闭、旧构建和针对性真实资源加载检查通过，整轮组合验收仍待收口；不包含 Weapon=／Damage.Delay= 新功能。

0.20 通过 `match_rules.building_presentation` 声明建筑 SHP／MAKE 独立调色板和销毁动画颜色回退，安装即生效，不增加 INI 开关。原引擎继续维护 ART 读取、调色板解析和动画状态；对象构造前刷新同一资源目录，建筑消失后的动画仍保留原颜色。实际组件、保存恢复、GPU、旧实现及真实资源加载检查通过；不包含新的建筑动画或 voxel 调色板扩展。

## 来源与许可

本工程迁自 VERA20K 本地源码提交 `314dfcbb5202a171d17edbd79fc1fab21d77e3f0`；原实现历史保留在引擎 Git 仓库。迁移保留文件内容、原有代码注释、贡献者说明和 GPL 许可，见 [LICENSE-GPL](LICENSE-GPL)。

Ares 全量兼容以[逐项兼容台账](compatibility/README.zh-CN.md)持续跟踪，包含官方功能、修复和参考入口；文件名不表示全量兼容已完成。

`NoAmmoWeapon`按安装版Ares的有符号整数语义处理：任意负数关闭，非负索引原样发布；有效0..17槽复用宿主现有选择器和精英回退。`WeaponN`读取、炮手提前返回及>=18安全无武器的边界见[弹药机制说明](plugins/vera20k-ares/AMMO.zh-CN.md)。建筑动画延迟枪需使用保存完整索引的配套新版宿主。0.6.0不扩展SDK或配置/状态结构。

内部组件标识仍为 `vera.reference-gameplay`。保留交付是组件 0.7.0、配置版本 5、状态版本 1；当前开发版本是 0.26.0、配置版本 17、状态版本 1，API v1 与 WIT 不变。开发分支已接入生产、护甲、附加效果、EMP、探照灯、类型转换、建筑占地、启动资源及阵营共享配置／玩法等既有子集，具体范围和配置见[独立开发指南](sdk/dev-kit/README.zh-CN.md)，整体迁移仍在进行。新默认只用于新对局；旧组件字节和已冻结配置保留原行为。读档要求当前作品提供与存档摘要相同的原组件，完整存档布局变化时也需保留相应旧引擎。本工程使用独立本地 Git 管理，未经授权不上传或公开。

0.17 接通国家菜单、旗帜、早晚加载画面与加载音乐，使用同一实时组件声明和原界面快照。场景全局清空保留尚存国家的父国继承历史，真正类型重建才清除；音乐仍由原播放模块负责。该链已通过实际组件、关联回归与旧实现对照，七种真实资源模式及上一版组件加载也已通过。0.18 接入侧栏、EVA SetSide、消息颜色、胜负文字及结算资源，仍使用原有界面、音频与资源所有者；保留交付不变。

启动资源迁移新增可选 `startup_resources` 能力，代码位于 `plugins/vera20k-ares/src/features/resources/startup.rs`。组件在原有加载时点选择 `[#include]`、附加音频目录和 CSF 文本；引擎仍负责文件查找、解析与实际资源目录。该增量已通过实际组件、应用加载与恢复、局域网地图工作线程、旧实现对照、七种零售资源模式和上一版组件加载检查。原保留交付不变，整体迁移尚未完成。
