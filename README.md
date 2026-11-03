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

## 来源与许可

本工程迁自 VERA20K 本地源码提交 `314dfcbb5202a171d17edbd79fc1fab21d77e3f0`；原实现历史保留在引擎 Git 仓库。迁移保留文件内容、原有代码注释、贡献者说明和 GPL 许可，见 [LICENSE-GPL](LICENSE-GPL)。

Ares 全量兼容以[逐项兼容台账](compatibility/README.zh-CN.md)持续跟踪，包含官方功能、修复和参考入口；文件名不表示全量兼容已完成。

`NoAmmoWeapon`按安装版Ares的有符号整数语义处理：任意负数关闭，非负索引原样发布；有效0..17槽复用宿主现有选择器和精英回退。`WeaponN`读取、炮手提前返回及>=18安全无武器的边界见[弹药机制说明](plugins/vera20k-ares/AMMO.zh-CN.md)。建筑动画延迟枪需使用保存完整索引的配套新版宿主。0.6.0不扩展SDK或配置/状态结构。

内部组件标识仍为 `vera.reference-gameplay`。保留交付是组件 0.7.0、配置版本 5、状态版本 1；当前开发版本是 0.15.0、配置版本 14、状态版本 1，API v1 与 WIT 不变。开发分支已接入生产、护甲、附加效果、EMP、探照灯、类型转换及建筑占地等既有子集，具体范围和配置见[独立开发指南](sdk/dev-kit/README.zh-CN.md)，整体迁移仍在进行。新默认只用于新对局；旧组件字节和已冻结配置保留原行为。读档要求当前作品提供与存档摘要相同的原组件，完整存档布局变化时也需保留相应旧引擎。本工程使用独立本地 Git 管理，未经授权不上传或公开。

启动资源迁移新增可选 `startup_resources` 能力，代码位于 `plugins/vera20k-ares/src/features/resources/startup.rs`。组件在原有加载时点选择 `[#include]`、附加音频目录和 CSF 文本；引擎仍负责文件查找、解析与实际资源目录。该增量已通过实际组件、应用加载与恢复、局域网地图工作线程、旧实现对照、七种零售资源模式和上一版组件加载检查。原保留交付不变，整体迁移尚未完成。
