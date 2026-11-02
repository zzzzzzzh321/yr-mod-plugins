# VERA20K 插件开发工程

本目录是后续外置功能插件的主要维护位置，与 `../engine` 相互独立。当前插件发布文件为 `vera20k-ares.wasm`，已有护盾、过载及 Ares 弹药、装填、低弹量选武器和手动操作限制示例。

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

当前内部组件标识仍为 `vera.reference-gameplay`，组件版本 0.5.0、配置版本 5、状态版本 1。本工程使用独立本地 Git 管理，未经授权不上传或公开。
